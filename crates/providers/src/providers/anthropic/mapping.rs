use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::LazyLock;

use common::OrderedFloat;
use common::errors::ProviderError;
use common::models::ReasoningEffort;
use common::shared::prelude::*;
use persistence::prelude::MessageRepositoryV2;

use crate::providers::anthropic::types::AnthropicEffort;
use crate::providers::anthropic::types::AnthropicThinking;
use crate::providers::anthropic::types::CacheControl;
use crate::providers::anthropic::types::CacheControlTtl;
use crate::providers::anthropic::types::ClaudeMesssage;
use crate::providers::anthropic::types::ContentPartImage;
use crate::providers::anthropic::types::ContentPartImageKind;
use crate::providers::anthropic::types::ContentPartText;
use crate::providers::anthropic::types::ContentPartThinking;
use crate::providers::anthropic::types::ContentPartToolResult;
use crate::providers::anthropic::types::ContentPartToolUse;
use crate::providers::anthropic::types::ContextManagement;
use crate::providers::anthropic::types::ContextManagementClear;
use crate::providers::anthropic::types::ContextManagementEdits;
use crate::providers::anthropic::types::ContextManagementKeep;
use crate::providers::anthropic::types::ContextManagementTrigger;
use crate::providers::anthropic::types::MessageRequestBody;
use crate::providers::anthropic::types::SystemRequestBody;
use crate::util::get_oauth_slug;

static MODEL_TO_REASONING_TOKENS: LazyLock<HashMap<&str, u32>> =
  LazyLock::new(|| HashMap::from([("opus", 32_000), ("sonnet", 32_000), ("haiku", 64_000)]));

#[derive(Clone, Debug, Default)]
pub struct AnthropicMapping;

impl AnthropicMapping {
  pub async fn to_messages(
    session_id: SurrealId,
    llm_model: LlmModel,
  ) -> std::result::Result<Vec<ClaudeMesssage>, ProviderError> {
    let mut tool_response_ids = HashSet::new();

    let history =
      MessageRepositoryV2::list(session_id.clone()).await.map_err(|e| ProviderError::Internal(e.to_string()))?;

    let history = history.iter().filter(|h| h.visibility.for_assistant()).cloned().collect::<Vec<_>>();

    let history = crate::util::history_pruning::apply_pruning(
      history,
      llm_model.clone(),
      crate::util::history_pruning::prune_history,
    )
    .await;

    let messages = history.iter().fold(Vec::new(), |mut acc, message| {
      let is_tool_result = message.content().is_tool_result();
      let current_role = if is_tool_result { MessageRole::User } else { message.role().clone() };

      let content = match (&current_role, message.content()) {
        (MessageRole::User | MessageRole::Assistant, MessageContent::Text(text)) => {
          if text.text.trim().is_empty() {
            None
          } else {
            Some(ContentPartText { text: text.text.clone() }.into())
          }
        }
        (MessageRole::User, MessageContent::Image64(image)) => {
          let data = image.image_64.split_once(",").map(|(_, data)| data.to_string()).unwrap_or_default();

          Some(
            ContentPartImage {
              source: ContentPartImageKind::Base64 { data: data, media_type: image.media_type.clone() },
            }
            .into(),
          )
        }
        (MessageRole::User, MessageContent::ToolResult(result)) => {
          let content = result.content.clone().into_llm_payload().to_string();

          let id = result.tool_use_id.trim_matches('"');
          if tool_response_ids.contains(&id) {
            None
          } else {
            tool_response_ids.insert(id);
            Some(ContentPartToolResult { tool_use_id: result.tool_use_id.clone(), content: content }.into())
          }
        }
        (MessageRole::Assistant, MessageContent::Thinking(thinking)) => {
          if thinking.thinking.trim().is_empty()
            || thinking.source_provider.is_none()
            || thinking.source_provider.unwrap() != Provider::Anthropic
          {
            None
          } else {
            Some(
              ContentPartThinking { thinking: thinking.thinking.clone(), signature: thinking.signature.clone() }.into(),
            )
          }
        }
        (MessageRole::Assistant, MessageContent::ToolUse(tool_use)) if message.visibility.for_assistant() => Some(
          ContentPartToolUse {
            id:    tool_use.id.clone(),
            name:  tool_use.tool_id.clone(),
            input: tool_use.input.clone(),
          }
          .into(),
        ),
        (MessageRole::Assistant | MessageRole::System, MessageContent::Error(error))
          if message.visibility.for_assistant() =>
        {
          let text = if error.error.is_some() {
            format!("Error: {}\n\n{}", serde_json::to_string(&error.error).unwrap_or_default(), error.message)
          } else {
            error.message.clone()
          };

          Some(ContentPartText { text: text }.into())
        }
        (
          MessageRole::Assistant | MessageRole::System,
          MessageContent::Warning(Signal { message: text, .. }) | MessageContent::Info(Signal { message: text, .. }),
        ) if message.visibility.for_assistant() => Some(ContentPartText { text: text.clone() }.into()),
        _ => None,
      };

      let Some(content) = content else {
        return acc;
      };

      let last: Option<&ClaudeMesssage> = acc.last();
      let prev_role = last.map(|m| m.role.clone()).unwrap_or_default();
      let is_same_role = prev_role == current_role;

      if acc.is_empty() || !is_same_role {
        acc.push(ClaudeMesssage { role: current_role, content: vec![content] });
      } else if let Some(last) = acc.last_mut() {
        last.content.push(content);
      } else {
        acc.push(ClaudeMesssage { role: current_role, content: vec![content] });
      }

      acc
    });

    Ok(messages)
  }

  pub async fn build_body(
    req: ChatRequest,
    stream: bool,
    tools: serde_json::Value,
  ) -> std::result::Result<MessageRequestBody, ProviderError> {
    let system = if let Some(instructions) = &req.instructions {
      vec![SystemRequestBody { kind: "text".into(), text: instructions.clone() }]
    } else {
      Self::build_system_prompt(req.clone().into())
    };

    let model = get_oauth_slug(&req.llm_model);
    let keys = MODEL_TO_REASONING_TOKENS.keys().cloned().collect::<Vec<_>>();
    let key = keys.iter().find(|k| k.contains(&model)).unwrap_or(&"haiku");
    let max_tokens = MODEL_TO_REASONING_TOKENS.get(key).unwrap_or(&32_000);

    let thinking = match req.reasoning_effort {
      ReasoningEffort::XHigh | ReasoningEffort::High => {
        Some(AnthropicThinking { kind: "enabled".into(), budget_tokens: max_tokens.saturating_sub(1) })
      }
      ReasoningEffort::Medium => Some(AnthropicThinking {
        kind:          "enabled".into(),
        budget_tokens: max_tokens.saturating_div(2).max(1024),
      }),
      ReasoningEffort::Low => Some(AnthropicThinking { kind: "enabled".into(), budget_tokens: 2048 }),
      ReasoningEffort::Minimal => Some(AnthropicThinking { kind: "enabled".into(), budget_tokens: 1024 }),
      ReasoningEffort::None => None,
    };

    let output_config = if model.contains("opus") && model.contains("4.5") {
      match req.reasoning_effort {
        ReasoningEffort::XHigh => Some(AnthropicEffort::Max.into()),
        ReasoningEffort::High => Some(AnthropicEffort::High.into()),
        ReasoningEffort::Medium => Some(AnthropicEffort::Medium.into()),
        ReasoningEffort::Low | ReasoningEffort::Minimal | ReasoningEffort::None => Some(AnthropicEffort::Low.into()),
      }
    } else {
      None
    };

    Ok(MessageRequestBody {
      model:              model,
      system:             system,
      messages:           Self::to_messages(req.session_id.clone(), req.llm_model.clone()).await?,
      stream:             Some(stream),
      max_tokens:         Some(*max_tokens),
      tools:              Some(tools),
      thinking:           thinking,
      output_config:      output_config,
      temperature:        Some(OrderedFloat(1.0)),
      context_management: Some(ContextManagement {
        edits: [ContextManagementEdits {
          kind:           "clear_tool_uses_20250919".into(),
          trigger:        ContextManagementTrigger { kind: "input_tokens".into(), value: 50_000 },
          keep:           ContextManagementKeep { kind: "tool_uses".into(), value: 3 },
          clear_at_least: ContextManagementClear { kind: "input_tokens".into(), value: 10_000 },
          exclude_tools:  vec![],
        }],
      }),
      cache_control:      Some(CacheControl { kind: "ephemeral".into(), ttl: Some(CacheControlTtl::OneHour) }),
    })
  }

  pub fn build_body_basic(prompt: String, system: String, model: Option<String>) -> MessageRequestBody {
    let system = vec![SystemRequestBody { kind: "text".into(), text: system }];

    let messages =
      vec![ClaudeMesssage { role: MessageRole::User, content: vec![ContentPartText { text: prompt }.into()] }];

    let model = model.unwrap_or_else(|| "claude-4.5-haiku".to_string());

    MessageRequestBody {
      model:              model,
      system:             system,
      messages:           messages,
      max_tokens:         Some(512),
      temperature:        Some(OrderedFloat(0.0)),
      output_config:      None,
      stream:             Some(false),
      tools:              None,
      thinking:           None,
      context_management: None,
      cache_control:      None,
    }
  }

  pub fn build_system_prompt(params: PromptParams) -> Vec<SystemRequestBody> {
    vec![SystemRequestBody { kind: "text".into(), text: "system_prompt".to_string() }]
  }
}
