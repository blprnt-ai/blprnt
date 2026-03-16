use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use common::agent::ToolId;
use common::errors::ProviderError;
use common::errors::SerdeError;
use common::provider_dispatch::ProviderDispatch;
use common::provider_dispatch::ProviderEvent;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::MessageContent;
use common::shared::prelude::MessageImage64;
use common::shared::prelude::MessageRole;
use common::shared::prelude::MessageText;
use common::shared::prelude::MessageThinking;
use common::shared::prelude::MessageToolResult;
use common::shared::prelude::MessageToolUse;
use common::shared::prelude::PromptParams;
use common::shared::prelude::Provider;
use common::shared::prelude::Signal;
use persistence::prelude::MessageRecord;
use persistence::prelude::MessageRepositoryV2;
use surrealdb::types::Uuid;

use super::request::ContentItem;
use super::request::InputItem;
use super::request::ResponsesChatRequestBody;
use super::response::OpenAiStreamEvent;
use crate::providers::openai::responses::request::ChatRequestBodyReasoning;
use crate::providers::openai::responses::request::CodexReasoningEffort;
use crate::providers::openai::responses::request::FunctionCallStatus;
use crate::providers::openai::responses::request::InputImageDetail;
use crate::providers::openai::responses::request::OutputText;
use crate::providers::openai::responses::request::ReasoningItemReasoningSummary;
use crate::providers::openai::responses::response::OutputItem;
use crate::providers::openai::responses::response::ReasoningContentPart;
use crate::providers::openai::responses::response::ResponseCompleted;
use crate::providers::openai::responses::response::ResponseFailed;
use crate::providers::openai::responses::response::ResponseSummary;
use crate::types::ParsedContentBlock;
use crate::util::get_oauth_slug;
use crate::util::sse::SseItem;

#[derive(Clone, Debug)]
pub struct SystemPrompt {
  pub instructions: String,
  pub dev_message:  Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct OpenAiResponsesMapping;

impl OpenAiResponsesMapping {
  fn to_openai_messages(provider: Provider, raw_messages: &[MessageRecord]) -> Vec<InputItem> {
    let mut tool_response_ids = HashSet::new();
    let mut messages = Vec::new();

    let mut iter = raw_messages.iter().peekable();

    while let Some(history) = iter.next() {
      match (history.role(), history.content()) {
        (MessageRole::User, MessageContent::Text(MessageText { text, .. })) => {
          let mut content = vec![ContentItem::InputText { text: text.clone() }];
          while let Some(next) = iter.peek()
            && next.is_user()
          {
            let next = iter.next().unwrap();
            match &next.content() {
              MessageContent::Text(MessageText { text, .. }) => {
                content.push(ContentItem::InputText { text: text.clone() });
              }
              MessageContent::Image64(MessageImage64 { image_64, .. }) => {
                content.push(ContentItem::InputImage { image_url: image_64.clone(), detail: InputImageDetail::Auto })
              }
              _ => break,
            }
          }

          messages.push(InputItem::Message {
            role:    MessageRole::User.to_string(),
            content: content,
            id:      None,
            status:  None,
          });
        }
        (MessageRole::Assistant, MessageContent::Text(MessageText { text, .. })) => {
          let id = if provider == Provider::OpenAi { None } else { Some(format!("msg_{}", history.id.key())) };
          if !text.is_empty() {
            messages.push(InputItem::Message {
              role:    MessageRole::Assistant.to_string(),
              content: vec![OutputText::from_text(text.clone())],
              id:      id,
              status:  Some("completed".to_string()),
            });
          }
        }
        (MessageRole::Assistant, MessageContent::Thinking(MessageThinking { thinking, .. })) => {
          let id = if provider == Provider::OpenAi { None } else { Some(format!("rs_{}", history.id.key())) };
          if !thinking.is_empty() {
            messages.push(InputItem::Reasoning {
              id:                id,
              summary:           vec![ReasoningItemReasoningSummary::SummaryText { text: thinking.clone() }],
              // content:           Some(vec![ReasoningItemContent::ReasoningText { text: thinking.clone() }]),
              content:           Some(vec![]),
              encrypted_content: None,
            });
          }
        }
        (MessageRole::Assistant, MessageContent::ToolUse(MessageToolUse { id: call_id, tool_id, input, .. })) => {
          let id = if provider == Provider::OpenAi { None } else { Some(format!("fc_{}", history.id.key())) };
          let call_id = call_id.trim_matches('"');
          let arguments = match input.get("__MALFORMED_JSON__") {
            Some(arguments) => serde_plain::to_string(arguments).unwrap_or_else(|_| arguments.to_string()),
            _ => input.to_string(),
          };

          let tool_id = match tool_id {
            ToolId::Mcp(name) => name.to_string(),
            _ => tool_id.to_string(),
          };

          messages.push(InputItem::FunctionCall {
            id:        id,
            call_id:   call_id.to_string(),
            name:      tool_id,
            arguments: arguments,
          });

          if history.is_user_visible() {
            let call_id = call_id.trim_matches('"');
            if tool_response_ids.contains(&call_id) {
              continue;
            }

            tool_response_ids.insert(call_id);
            let id = if provider == Provider::OpenAi { None } else { Some(format!("fc_out_{}", history.id.key())) };
            messages.push(InputItem::FunctionCallOutput {
              id:      id,
              call_id: call_id.to_string(),
              output:  "".to_string(),
              status:  FunctionCallStatus::Incomplete,
            });
          }
        }
        (
          MessageRole::Assistant,
          MessageContent::ToolResult(MessageToolResult { tool_use_id: call_id, content, .. }),
        ) => {
          let call_id = call_id.trim_matches('"');
          if tool_response_ids.contains(&call_id) {
            continue;
          }

          tool_response_ids.insert(call_id);
          let id = if provider == Provider::OpenAi { None } else { Some(format!("fc_out_{}", history.id.key())) };

          messages.push(InputItem::FunctionCallOutput {
            id:      id,
            call_id: call_id.to_string(),
            output:  serde_json::to_string(&content.clone().into_llm_payload()).unwrap_or_default(),
            status:  FunctionCallStatus::Completed,
          });
        }
        (MessageRole::Assistant | MessageRole::System, MessageContent::Error(error))
          if history.visibility.for_assistant() =>
        {
          let text = if error.error.is_some() {
            format!("Error: {}\n\n{}", serde_json::to_string(&error.error).unwrap_or_default(), error.message)
          } else {
            error.message.clone()
          };

          messages.push(InputItem::Message {
            role:    MessageRole::User.to_string(),
            content: vec![ContentItem::InputText { text: text }],
            id:      None,
            status:  None,
          });
        }
        (
          MessageRole::Assistant | MessageRole::System,
          MessageContent::Warning(Signal { message: text, .. }) | MessageContent::Info(Signal { message: text, .. }),
        ) if history.visibility.for_assistant() => messages.push(InputItem::Message {
          role:    MessageRole::User.to_string(),
          content: vec![ContentItem::InputText { text: text.clone() }],
          id:      None,
          status:  None,
        }),
        _ => {}
      }
    }

    messages
  }

  pub async fn build_body(
    provider: Provider,
    req: ChatRequest,
    stream: bool,
    tools: Option<serde_json::Value>,
  ) -> std::result::Result<ResponsesChatRequestBody, ProviderError> {
    let history =
      MessageRepositoryV2::list(req.session_id.clone()).await.map_err(|e| ProviderError::Internal(e.to_string()))?;

    let history = history.iter().filter(|h| h.visibility.for_assistant()).cloned().collect::<Vec<_>>();

    let history = crate::util::history_pruning::apply_pruning(
      history,
      req.llm_model.clone(),
      crate::util::history_pruning::prune_history,
    )
    .await;
    let mut input = Self::to_openai_messages(provider, &history);

    let instructions = req.instructions.clone().or_else(|| {
      let instructions = Self::build_system_prompt(req.clone().into());

      if let Some(dev_message) = instructions.dev_message {
        input.insert(
          0,
          InputItem::Message {
            role:    "developer".into(),
            content: vec![ContentItem::InputText { text: dev_message }],
            id:      None,
            status:  None,
          },
        );
      }

      Some(instructions.instructions)
    });

    let supports_reasoning = req.llm_model.supports_reasoning;
    let supports_reasoning =
      supports_reasoning && req.llm_model.provider_slug.as_ref().unwrap_or(&"".to_string()) != "gpt-5.3-codex-spark";

    let reasoning = if supports_reasoning { Some(ChatRequestBodyReasoning::from(req.reasoning_effort)) } else { None };
    let model = if provider == Provider::OpenAi { get_oauth_slug(&req.llm_model) } else { req.llm_model.slug.clone() };

    Ok(ResponsesChatRequestBody {
      model:               model,
      instructions:        instructions,
      input:               input,
      parallel_tool_calls: true,
      reasoning:           reasoning,
      store:               Some(false),
      include:             vec![],
      stream:              Some(stream),
      tools:               tools,
    })
  }

  pub fn build_body_basic(
    model: String,
    supports_reasoning: bool,
    prompt: String,
    system: String,
  ) -> ResponsesChatRequestBody {
    Self::build_body_basic_base(
      model,
      supports_reasoning,
      system,
      vec![InputItem::Message {
        role:    "user".into(),
        content: vec![ContentItem::InputText { text: prompt }],
        id:      None,
        status:  None,
      }],
    )
  }

  fn build_body_basic_base(
    model: String,
    supports_reasoning: bool,
    instructions: String,
    input: Vec<InputItem>,
  ) -> ResponsesChatRequestBody {
    let reasoning =
      if supports_reasoning { Some(ChatRequestBodyReasoning::from(CodexReasoningEffort::Low)) } else { None };

    ResponsesChatRequestBody {
      model:               model,
      instructions:        Some(instructions),
      input:               input,
      parallel_tool_calls: false,
      reasoning:           reasoning,
      store:               Some(false),
      include:             vec![],
      stream:              Some(true),
      tools:               None,
    }
  }

  pub fn stream_event_from_sse(
    provider: Provider,
    value: &SseItem,
    provider_dispatch: Arc<ProviderDispatch>,
    content_blocks: &mut HashMap<u32, ParsedContentBlock>,
  ) -> Result<Option<bool>, ProviderError> {
    let event = match serde_json::from_value::<OpenAiStreamEvent>(value.clone()) {
      Ok(event) => event,
      Err(e) => {
        tracing::error!("error: {}", serde_json::to_string_pretty(&value).unwrap_or_default());
        tracing::error!("error: {}", e.to_string());
        return Ok(None);
      }
    };

    match &event {
      OpenAiStreamEvent::Created { response: ResponseSummary { id, .. }, .. } => {
        provider_dispatch
          .send(ProviderEvent::Start(id.clone().unwrap_or_default()))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;
        Ok(None)
      }
      OpenAiStreamEvent::Completed { response: ResponseCompleted { id, usage, .. }, .. } => {
        // Ok(Some(true))
        if let Some(usage) = usage
          && let Some(input_tokens) = usage.input_tokens
        {
          let _ = provider_dispatch.send(ProviderEvent::TokenUsage(input_tokens));
        }

        provider_dispatch
          .send(ProviderEvent::Stop(id.clone().unwrap_or_default()))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      // Response
      OpenAiStreamEvent::OutputItemAdded { output_index, item: OutputItem::Message { .. }, .. } => {
        let id = Uuid::new_v7().to_string();
        let content_block = ParsedContentBlock::new_text(*output_index, id.clone(), None);
        content_blocks.insert(*output_index, content_block);

        provider_dispatch
          .send(ProviderEvent::ResponseStarted { rel_id: id })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;
        Ok(None)
      }
      OpenAiStreamEvent::OutputTextDelta { output_index, delta, .. } if content_blocks.get(output_index).is_some() => {
        let Some(content_block) = content_blocks.get_mut(output_index) else {
          return Err(ProviderError::Internal(format!("missing content block for output_index={}", output_index)));
        };
        let id = content_block.get_id();
        content_block.append_text(delta.clone());

        provider_dispatch
          .send(ProviderEvent::ResponseDelta { rel_id: id.clone(), delta: delta.clone() })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      OpenAiStreamEvent::OutputItemDone { output_index, item: OutputItem::Message { .. }, .. }
        if content_blocks.get(output_index).is_some() =>
      {
        let Some(content_block) = content_blocks.remove(output_index) else {
          return Err(ProviderError::Internal(format!("missing content block for output_index={output_index}")));
        };
        let id = content_block.get_id();
        let content = content_block.get_text();

        provider_dispatch
          .send(ProviderEvent::Response { rel_id: id.clone(), content, signature: None })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;
        provider_dispatch
          .send(ProviderEvent::ResponseDone { rel_id: id })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      // Reasoning
      OpenAiStreamEvent::OutputItemAdded { output_index, item: OutputItem::Reasoning { .. }, .. }
      | OpenAiStreamEvent::ReasoningSummaryPartAdded { output_index, .. }
        if content_blocks.get(output_index).is_none() =>
      {
        let id = Uuid::new_v7().to_string();
        let content_block = ParsedContentBlock::new_thinking(*output_index, id.clone(), None);
        content_blocks.insert(*output_index, content_block);

        provider_dispatch
          .send(ProviderEvent::ReasoningStarted { rel_id: id })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;
        Ok(None)
      }
      OpenAiStreamEvent::ReasoningTextDelta { output_index, delta, .. }
      | OpenAiStreamEvent::ReasoningSummaryTextDelta { output_index, delta, .. }
        if content_blocks.get(output_index).is_some() =>
      {
        let content_block = content_blocks.get_mut(output_index).unwrap();

        content_block.append_thinking(delta.to_string());
        let id = content_block.get_id();
        provider_dispatch
          .send(ProviderEvent::ReasoningDelta { rel_id: id, delta: delta.to_string() })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      OpenAiStreamEvent::ReasoningSummaryTextDone { output_index, text, .. }
      | OpenAiStreamEvent::ReasoningSummaryPartDone { output_index, part: ReasoningContentPart { text, .. }, .. }
      | OpenAiStreamEvent::ReasoningTextDone { output_index, text, .. }
        if content_blocks.get(output_index).is_some() =>
      {
        let Some(content_block) = content_blocks.remove(output_index) else {
          return Err(ProviderError::Internal(format!("missing content block for output_index={output_index}")));
        };
        let id = content_block.get_id();
        let mut reasoning_text = content_block.get_thinking();
        if reasoning_text.trim().is_empty() {
          reasoning_text = text.clone();
        }

        let trimmed = reasoning_text.trim();
        let is_meaningful = !trimmed.is_empty() && !trimmed.trim_matches('*').trim().is_empty();

        if is_meaningful {
          provider_dispatch
            .send(ProviderEvent::Reasoning { rel_id: id.clone(), reasoning: reasoning_text, signature: None })
            .map_err(|e| ProviderError::Internal(e.to_string()))?;
          provider_dispatch
            .send(ProviderEvent::ReasoningDone { rel_id: id })
            .map_err(|e| ProviderError::Internal(e.to_string()))?;
        }

        Ok(None)
      }

      // Function call
      OpenAiStreamEvent::OutputItemAdded {
        item: OutputItem::FunctionCall { name, call_id, .. }, output_index, ..
      } if provider == Provider::OpenRouter => {
        let content_block = ParsedContentBlock::new_function_call(*output_index, call_id.clone(), name.clone());
        content_blocks.insert(*output_index, content_block);

        Ok(None)
      }
      OpenAiStreamEvent::FunctionCallArgumentsDelta { output_index, delta, .. }
        if provider == Provider::OpenRouter && content_blocks.get(output_index).is_some() =>
      {
        if let Some(content_block) = content_blocks.get_mut(output_index) {
          content_block.append_input(delta.clone());
        };

        Ok(None)
      }
      OpenAiStreamEvent::FunctionCallArgumentsDone { output_index, .. }
        if provider == Provider::OpenRouter && content_blocks.get(output_index).is_some() =>
      {
        let Some(content_block) = content_blocks.remove(output_index) else {
          return Err(ProviderError::Internal(format!("missing content block for output_index={output_index}")));
        };
        let id = content_block.get_id();
        let input = content_block.get_input();
        let name = content_block.get_name();

        let tool_id = ToolId::try_from(name.clone()).map_err(|e| ProviderError::LlmMistake {
          context: "openai::responses::mapping::stream_event_from_sse".into(),
          message: e.to_string(),
        })?;

        provider_dispatch
          .send(ProviderEvent::ToolCall {
            tool_id:     tool_id,
            tool_use_id: id.clone(),
            args:        input,
            signature:   None,
          })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      OpenAiStreamEvent::OutputItemDone { item: OutputItem::FunctionCall { name, call_id, arguments, .. }, .. }
        if provider != Provider::OpenRouter =>
      {
        let tool_id = ToolId::try_from(name.clone()).map_err(|e| match e {
          SerdeError::FailedToDeserializeFromPlain(e) => ProviderError::InvalidToolId {
            call_id:   call_id.clone(),
            tool_id:   name.clone(),
            arguments: arguments.clone(),
            context:   "openai::responses::mapping::stream_event_from_sse".into(),
            message:   e,
          },
          _ => unreachable!(),
        });

        // Silently try to fix the llm mistake by stripping the "functions." prefix
        // If that still fails, return the original error
        let tool_id = match tool_id {
          Ok(tool_id) => tool_id,
          Err(e) => {
            let name_fixed = name.strip_prefix("functions.").map(|name| name.to_string()).unwrap_or(name.clone());
            ToolId::try_from(name_fixed.clone()).map_err(|_| e)?
          }
        };

        provider_dispatch
          .send(ProviderEvent::ToolCall {
            tool_id,
            tool_use_id: call_id.clone(),
            args: arguments.clone(),
            signature: None,
          })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      OpenAiStreamEvent::Failed { response: ResponseFailed { error, .. }, .. } => {
        provider_dispatch
          .send(ProviderEvent::Error(ProviderError::LlmError {
            context: "responses::mapping::stream_event_from_sse".into(),
            message: format!("LLM error: {:#?}", error.message),
          }))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;
        Ok(None)
      }

      _ => Ok(None),
    }
  }

  pub fn build_system_prompt(params: PromptParams) -> SystemPrompt {
    SystemPrompt { instructions: prompt::render_prompt(params), dev_message: None }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use persistence::prelude::SurrealId;
  use surrealdb::types::Uuid;

  use super::*;

  #[tokio::test]
  async fn test_openrouter_trailing_json_tool_call_emits_tool_call() {
    let (tx, mut rx) = tokio::sync::broadcast::channel(16);
    let provider_dispatch = ProviderDispatch::new(tx);
    let mut content_blocks = HashMap::new();

    let created = OpenAiStreamEvent::Created {
      response: ResponseSummary { id: Some("resp-1".into()), model: Some("test".into()), ..Default::default() },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(created).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let added = OpenAiStreamEvent::OutputItemAdded {
      output_index: 0,
      item:         OutputItem::Reasoning { id: Some("reason-1".into()), content: None, summary: None },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(added).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let trailing_json = r#"{"name":"file_read","arguments":{"path":"/tmp/a"},"id":"call-123"}"#;
    let done = OpenAiStreamEvent::ReasoningTextDone {
      item_id:         "reason-1".into(),
      output_index:    0,
      content_index:   None,
      sequence_number: None,
      text:            format!("Thinking step {}", trailing_json),
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(done).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let completed = OpenAiStreamEvent::Completed {
      response: ResponseCompleted { id: Some("resp-1".into()), model: Some("test".into()), ..Default::default() },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(completed).unwrap(),
      provider_dispatch,
      &mut content_blocks,
    )
    .unwrap();

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
      events.push(event);
    }

    let reasoning = events.iter().find_map(|event| match event {
      ProviderEvent::Reasoning { reasoning, .. } => Some(reasoning.clone()),
      _ => None,
    });
    assert_eq!(reasoning.unwrap_or_default(), "Thinking step");

    let tool_calls = events
      .iter()
      .filter_map(|event| match event {
        ProviderEvent::ToolCall { tool_id, tool_use_id, args, .. } => {
          Some((tool_id.clone(), tool_use_id.clone(), args.clone()))
        }
        _ => None,
      })
      .collect::<Vec<_>>();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].0, ToolId::FilesRead);
    assert_eq!(tool_calls[0].1, "call-123");
    assert_eq!(tool_calls[0].2, r#"{"path":"/tmp/a"}"#);
  }

  #[tokio::test]
  async fn test_openrouter_trailing_json_without_id_generates_tool_use_id() {
    let (tx, mut rx) = tokio::sync::broadcast::channel(16);
    let provider_dispatch = ProviderDispatch::new(tx);
    let mut content_blocks = HashMap::new();

    let created = OpenAiStreamEvent::Created {
      response: ResponseSummary { id: Some("resp-2".into()), model: Some("test".into()), ..Default::default() },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(created).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let added = OpenAiStreamEvent::OutputItemAdded {
      output_index: 0,
      item:         OutputItem::Reasoning { id: Some("reason-2".into()), content: None, summary: None },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(added).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let trailing_json = r#"{"name":"file_read","arguments":{"path":"/tmp/b"}}"#;
    let done = OpenAiStreamEvent::ReasoningTextDone {
      item_id:         "reason-2".into(),
      output_index:    0,
      content_index:   None,
      sequence_number: None,
      text:            format!("Thinking two {}", trailing_json),
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(done).unwrap(),
      provider_dispatch,
      &mut content_blocks,
    )
    .unwrap();

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
      events.push(event);
    }

    let tool_use_id = events.iter().find_map(|event| match event {
      ProviderEvent::ToolCall { tool_use_id, .. } => Some(tool_use_id.clone()),
      _ => None,
    });
    assert!(tool_use_id.unwrap_or_default().starts_with("openrouter-trailing-"));
  }

  #[tokio::test]
  async fn test_openrouter_trailing_text_without_json_keeps_reasoning() {
    let (tx, mut rx) = tokio::sync::broadcast::channel(16);
    let provider_dispatch = ProviderDispatch::new(tx);
    let mut content_blocks = HashMap::new();

    let created = OpenAiStreamEvent::Created {
      response: ResponseSummary { id: Some("resp-3".into()), model: Some("test".into()), ..Default::default() },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(created).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let added = OpenAiStreamEvent::OutputItemAdded {
      output_index: 0,
      item:         OutputItem::Reasoning { id: Some("reason-3".into()), content: None, summary: None },
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(added).unwrap(),
      provider_dispatch.clone(),
      &mut content_blocks,
    )
    .unwrap();

    let done = OpenAiStreamEvent::ReasoningTextDone {
      item_id:         "reason-3".into(),
      output_index:    0,
      content_index:   None,
      sequence_number: None,
      text:            "Just thinking".to_string(),
    };
    OpenAiResponsesMapping::stream_event_from_sse(
      Provider::OpenRouter,
      &serde_json::to_value(done).unwrap(),
      provider_dispatch,
      &mut content_blocks,
    )
    .unwrap();

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
      events.push(event);
    }

    let reasoning = events.iter().find_map(|event| match event {
      ProviderEvent::Reasoning { reasoning, .. } => Some(reasoning.clone()),
      _ => None,
    });
    assert_eq!(reasoning.unwrap_or_default(), "Just thinking");
    assert!(events.iter().all(|event| !matches!(event, ProviderEvent::ToolCall { .. })));
  }

  #[tokio::test]
  async fn test_conversation() {
    let uuid = Uuid::from_str("019caf2a-19fe-7f10-b37c-286836582980").unwrap();
    let session_id: SurrealId = ("sessions".to_string(), uuid).into();

    println!("Session ID: {}", session_id);

    let messages = MessageRepositoryV2::list(session_id.clone())
      .await
      .unwrap()
      .iter()
      .filter(|h| h.visibility.for_assistant())
      .cloned()
      .collect::<Vec<_>>();
    println!("Messages: {}", serde_json::to_string_pretty(&messages).unwrap());

    let messages = OpenAiResponsesMapping::to_openai_messages(Provider::OpenAi, &messages);

    println!("Messages: {}", serde_json::to_string_pretty(&messages).unwrap());
  }
}
