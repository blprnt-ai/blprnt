#![allow(clippy::redundant_field_names)]
#![warn(unused, unused_crate_dependencies)]

use std::path::PathBuf;

use anyhow::Result;
use common::agent::ToolId;
use common::consts::OS_LINE_ENDING;
use common::errors::ErrorEvent;
use common::models::ReasoningEffort;
use common::session_dispatch::prelude::SignalPayload;
use common::session_dispatch::prelude::SubagentDetails;
use common::shared::prelude::*;
use common::tools::ToolResult;
use persistence::prelude::MessageModelV2;
use persistence::prelude::MessagePatchV2;
use persistence::prelude::MessageRecord;
use persistence::prelude::MessageRepositoryV2;
use persistence::prelude::ProjectRepositoryV2;
use persistence::prelude::SessionPatchV2;
use persistence::prelude::SessionRepositoryV2;
use serde_json::Value;
use surrealdb::types::Uuid;

pub struct Session;

impl Session {
  fn memory_dirty_message(model: MessageModelV2) -> MessageModelV2 {
    model.with_memory_dirty(true)
  }

  pub async fn working_directories(project_id: &SurrealId) -> Result<Vec<PathBuf>> {
    let project = ProjectRepositoryV2::get(project_id.clone()).await?;

    Ok(project.working_directories().into_vec())
  }

  pub async fn agent_primer(project_id: &SurrealId) -> Result<Option<String>> {
    let project = ProjectRepositoryV2::get(project_id.clone()).await?;

    Ok(project.agent_primer().clone())
  }

  pub async fn token_usage(session_id: &SurrealId) -> Result<u32> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    Ok(session.token_usage().to_owned())
  }

  pub async fn update_token_usage(session_id: &SurrealId, token_usage: u32) -> Result<()> {
    let session_patch = SessionPatchV2 { token_usage: Some(token_usage), ..Default::default() };
    SessionRepositoryV2::update(session_id.clone(), session_patch).await?;
    Ok(())
  }

  // History

  pub async fn get_last_message(session_id: &SurrealId) -> Option<MessageRecord> {
    let messages = MessageRepositoryV2::list(session_id.clone()).await.ok()?;
    messages.last().cloned()
  }

  pub async fn append_user_input(
    session_id: &SurrealId,
    contents: Vec<MessageContent>,
    turn_id: Uuid,
    step_id: Uuid,
    reasoning_effort: ReasoningEffort,
    visibility: Option<HistoryVisibility>,
  ) -> Result<SurrealId> {
    let mut parent_id: Option<SurrealId> = None;

    for content in contents {
      let content = match content {
        MessageContent::Text(text) => {
          let text = OS_LINE_ENDING.apply(&text.text);
          MessageContent::Text(MessageText { text, signature: None })
        }
        _ => content,
      };

      let mut history_model = Self::memory_dirty_message(MessageModelV2::user_full_message(content, reasoning_effort));
      history_model.turn_id = turn_id;
      history_model.step_id = step_id;
      if let Some(visibility) = visibility.clone() {
        history_model.visibility = visibility;
      }

      let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

      if let Some(parent_id) = &parent_id {
        MessageRepositoryV2::relate_parent(parent_id.clone(), history_model.id.clone()).await?;
      }

      if parent_id.is_none() {
        parent_id = Some(history_model.id);
      }
    }

    Ok(parent_id.unwrap())
  }

  pub async fn set_message_reasoning_effort(message_id: &SurrealId, reasoning_effort: ReasoningEffort) -> Result<()> {
    let history_patch = MessagePatchV2 { reasoning_effort: Some(reasoning_effort), ..Default::default() };
    MessageRepositoryV2::update(message_id.clone(), history_patch).await?;

    Ok(())
  }

  pub async fn append_user_input_system(
    session_id: &SurrealId,
    content: String,
    turn_id: Uuid,
    step_id: Uuid,
    reasoning_effort: ReasoningEffort,
  ) -> Result<SurrealId> {
    let content = MessageContent::Text(MessageText { text: OS_LINE_ENDING.apply(&content), signature: None });

    let mut history_model =
      Self::memory_dirty_message(MessageModelV2::user_assistant_message(content, reasoning_effort));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    Ok(history_model.id)
  }

  pub async fn append_signal_to_user(session_id: &SurrealId, content: MessageContent) -> Result<SurrealId> {
    let history_model = Self::memory_dirty_message(MessageModelV2::system_user_message(content));
    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    Ok(history_model.id)
  }

  pub async fn append_signal_to_assistant(session_id: &SurrealId, content: MessageContent) -> Result<SurrealId> {
    let history_model = Self::memory_dirty_message(MessageModelV2::system_assistant_message(content));
    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    Ok(history_model.id)
  }

  pub async fn append_assistant_message(
    session_id: &SurrealId,
    rel_id: String,
    turn_id: Uuid,
    step_id: Uuid,
    text: String,
    signature: Option<String>,
    parent_id: Option<SurrealId>,
  ) -> Result<SurrealId> {
    let text = OS_LINE_ENDING.apply(&text);
    let content = MessageContent::Text(MessageText { text, signature });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_full_message(content));
    history_model.rel_id = rel_id;
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id)
  }

  pub async fn get_assistant_message_id(rel_id: String) -> Result<SurrealId> {
    MessageRepositoryV2::get_by_rel_id(rel_id).await.map(|h| h.id)
  }

  pub async fn update_assistant_message(rel_id: String, text: String, signature: Option<String>) -> Result<SurrealId> {
    let text = OS_LINE_ENDING.apply(&text);
    let content = MessageContent::Text(MessageText { text, signature });

    if let Ok(history_model) = MessageRepositoryV2::get_by_rel_id(rel_id).await {
      let history_patch = MessagePatchV2 { content: Some(content), ..Default::default() };
      MessageRepositoryV2::update(history_model.id.clone(), history_patch).await?;

      Ok(history_model.id)
    } else {
      Ok(SurrealId::default())
    }
  }

  pub async fn append_tool_request(
    session_id: &SurrealId,
    turn_id: Uuid,
    step_id: Uuid,
    tool_request: MessageToolUse,
    parent_id: Option<SurrealId>,
  ) -> Result<SurrealId> {
    let content = MessageContent::ToolUse(tool_request);
    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_user_message(content));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id)
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn init_tool_request(
    session_id: &SurrealId,
    tool_use_id: String,
    turn_id: Uuid,
    step_id: Uuid,
    tool_id: ToolId,
    args: Value,
    signature: Option<String>,
    parent_id: Option<SurrealId>,
    subagent_details: Option<SubagentDetails>,
  ) -> Result<SurrealId> {
    if let Ok(history_model) = MessageRepositoryV2::get_by_rel_id(tool_use_id.clone()).await {
      return Ok(history_model.id);
    }

    let content = MessageContent::ToolUse(MessageToolUse {
      id:               tool_use_id.clone(),
      tool_id:          tool_id,
      input:            args,
      signature:        signature,
      subagent_details: subagent_details,
    });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_user_message(content));
    history_model.rel_id = tool_use_id.clone();
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id)
  }

  pub async fn complete_tool_request(rel_id: String) -> bool {
    if let Ok(history_model) = MessageRepositoryV2::get_by_rel_id(rel_id).await {
      if history_model.is_tool_request() {
        return false;
      }

      let history_patch =
        MessagePatchV2 { visibility: Some(PartialVisibility::ToolRequest.into()), ..Default::default() };
      MessageRepositoryV2::update(history_model.id, history_patch).await.is_ok()
    } else {
      false
    }
  }

  pub async fn append_tool_response(
    session_id: &SurrealId,
    turn_id: Uuid,
    step_id: Uuid,
    tool_result: ToolResult,
    parent_id: Option<SurrealId>,
  ) -> Result<SurrealId> {
    let content = MessageContent::ToolResult(MessageToolResult {
      tool_use_id: tool_result.tool_use_id.clone(),
      content:     tool_result.result.clone(),
    });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_tool_response_message(content));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id)
  }

  pub async fn append_cancelled_tool_response(
    session_id: &SurrealId,
    turn_id: Uuid,
    step_id: Uuid,
    tool_result: ToolResult,
    parent_id: Option<SurrealId>,
  ) -> Result<SurrealId> {
    let content = MessageContent::ToolResult(MessageToolResult {
      tool_use_id: tool_result.tool_use_id.clone(),
      content:     tool_result.result.clone(),
    });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_assistant_message(content));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id.clone())
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn append_assistant_reasoning(
    session_id: &SurrealId,
    rel_id: String,
    turn_id: Uuid,
    step_id: Uuid,
    reasoning: String,
    signature: Option<String>,
    parent_id: Option<SurrealId>,
    source_provider: Provider,
  ) -> Result<SurrealId> {
    let reasoning = OS_LINE_ENDING.apply(&reasoning);
    let content = MessageContent::Thinking(MessageThinking {
      thinking:        reasoning,
      signature:       signature.unwrap_or_default(),
      source_provider: Some(source_provider),
    });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_none_message(content));
    history_model.rel_id = rel_id;
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    if let Some(parent_id) = parent_id {
      MessageRepositoryV2::relate_parent(parent_id, history_model.id.clone()).await?;
    }

    Ok(history_model.id)
  }

  pub async fn get_assistant_reasoning_id(rel_id: String) -> Result<SurrealId> {
    MessageRepositoryV2::get_by_rel_id(rel_id).await.map(|h| h.id)
  }

  pub async fn update_assistant_reasoning(
    rel_id: String,
    reasoning: String,
    signature: String,
    source_provider: Provider,
  ) -> Result<SurrealId> {
    let reasoning = OS_LINE_ENDING.apply(&reasoning);
    let content = MessageContent::Thinking(MessageThinking {
      thinking: reasoning,
      signature,
      source_provider: Some(source_provider),
    });

    if let Ok(history_model) = MessageRepositoryV2::get_by_rel_id(rel_id).await {
      let history_patch =
        MessagePatchV2 { content: Some(content), visibility: Some(HistoryVisibility::User), ..Default::default() };
      MessageRepositoryV2::update(history_model.id.clone(), history_patch).await?;
      Ok(history_model.id)
    } else {
      Ok(SurrealId::default())
    }
  }

  pub async fn complete_assistant_reasoning(rel_id: String) -> Result<SurrealId> {
    if let Ok(history_model) = MessageRepositoryV2::get_by_rel_id(rel_id).await {
      let history_patch = MessagePatchV2 { visibility: Some(HistoryVisibility::Full), ..Default::default() };
      MessageRepositoryV2::update(history_model.id.clone(), history_patch).await?;
      Ok(history_model.id)
    } else {
      Ok(SurrealId::default())
    }
  }

  pub async fn append_assistant_system_error(
    session_id: &SurrealId,
    turn_id: Uuid,
    step_id: Uuid,
    error: ErrorEvent,
  ) -> Result<SurrealId> {
    let message = r#"<system-message>
An error occurred while processing the request.
If it is recoverable, continue the conversation. If not, let the user know and end the turn.
</system-message>"#;

    let signal = SignalPayload::error_from_with_message(message.to_string(), error);
    let content = signal.into();

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_assistant_message(content));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    Ok(history_model.id)
  }

  pub async fn append_assistant_reasoning_from_system(
    session_id: &SurrealId,
    turn_id: Uuid,
    step_id: Uuid,
    message: String,
  ) -> Result<SurrealId> {
    let content = MessageContent::Text(MessageText { text: OS_LINE_ENDING.apply(&message), signature: None });

    let mut history_model = Self::memory_dirty_message(MessageModelV2::assistant_assistant_message(content));
    history_model.turn_id = turn_id;
    history_model.step_id = step_id;

    let history_model = MessageRepositoryV2::create(history_model, session_id.clone()).await?;

    Ok(history_model.id)
  }

  pub async fn heal_orphans(session_id: &SurrealId) -> Result<()> {
    let history_models = MessageRepositoryV2::list(session_id.clone()).await?;

    // Collect all tool calls
    let tool_calls = history_models
      .iter()
      .filter_map(|h| match h.content() {
        MessageContent::ToolUse(tool_use) => Some((h.clone(), tool_use)),
        _ => None,
      })
      .collect::<Vec<_>>();
    let tool_results = history_models
      .iter()
      .filter_map(|h| match h.content() {
        MessageContent::ToolResult(tool_result) => Some(tool_result),
        _ => None,
      })
      .collect::<Vec<_>>();

    let tool_result_ids = tool_results.iter().map(|h| h.tool_use_id.clone()).collect::<Vec<_>>();

    // intersect tool call and tool result ids to find missing results
    let missing_results =
      tool_calls.iter().filter(|(_, tool_use)| !tool_result_ids.contains(&tool_use.id)).collect::<Vec<_>>();

    for missing in missing_results {
      if !missing.0.is_user_visible() {
        let history_patch = MessagePatchV2 { visibility: Some(HistoryVisibility::User), ..Default::default() };
        MessageRepositoryV2::update(missing.0.id.clone(), history_patch).await?;
      }
    }

    Ok(())
  }

  pub async fn set_message_memory_dirty(message_id: &SurrealId, memory_dirty: bool) -> Result<SurrealId> {
    let history_patch = MessagePatchV2 { memory_dirty: Some(memory_dirty), ..Default::default() };
    MessageRepositoryV2::update(message_id.clone(), history_patch).await?;
    Ok(message_id.clone())
  }
}
