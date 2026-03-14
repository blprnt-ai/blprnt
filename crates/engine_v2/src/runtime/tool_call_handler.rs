use std::collections::HashMap;
use std::sync::Arc;

use common::agent::ToolId;
use common::tools::ToolResult;
use common::tools::ToolUseResponseError;
use persistence::prelude::MessageRepositoryV2;
use serde_json::Value;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::prelude::ControllerConfig;
use crate::runtime::UserInteraction;
use crate::runtime::ask_question_handler::AskQuestionHandler;
use crate::runtime::basic_tool_handler::BasicToolHandler;
use crate::runtime::context::RuntimeContext;
use crate::runtime::subagent_handler::SubagentHandler;
use crate::runtime::terminal_handler::TerminalHandler;
use crate::terminal::TerminalManager;

type SharedTerminal = Arc<Mutex<TerminalManager>>;
type TerminalManagers = Arc<Mutex<HashMap<Uuid, SharedTerminal>>>;

pub struct ToolCallHandlerParams {
  pub config:                    ControllerConfig,
  pub turn_id:                   Uuid,
  pub step_id:                   Uuid,
  pub tool_id:                   ToolId,
  pub tool_use_id:               String,
  pub args:                      String,
  pub signature:                 Option<String>,
  pub runtime_context:           Arc<RuntimeContext>,
  pub user_interaction_requests: Arc<Mutex<HashMap<String, oneshot::Sender<UserInteraction>>>>,
  pub terminal_manager:          TerminalManagers,
}

#[derive(Debug)]
pub struct ToolCallHandler;

impl ToolCallHandler {
  pub async fn handle(params: ToolCallHandlerParams, function_calls: &mut JoinSet<ToolResult>) {
    let args = if params.args.is_empty() { "{}".to_string() } else { params.args.clone() };

    let healed_args = match json_repair::repair_json_string(&args) {
      Ok(healed_args) => healed_args,
      Err(e) => {
        return Self::handle_tool_call_error(
          params.turn_id,
          params.step_id,
          params.tool_id,
          params.tool_use_id,
          params.args,
          params.signature,
          params.runtime_context,
          function_calls,
          e.to_string(),
        )
        .await;
      }
    };

    let result = match params.tool_id {
      ToolId::SubAgent => {
        SubagentHandler::handle(
          params.config.clone(),
          params.turn_id,
          params.step_id,
          params.tool_use_id.clone(),
          healed_args,
          params.runtime_context.clone(),
          function_calls,
        )
        .await
      }
      ToolId::AskQuestion => {
        AskQuestionHandler::handle(
          params.turn_id,
          params.step_id,
          params.tool_use_id.clone(),
          healed_args,
          params.signature.clone(),
          params.runtime_context.clone(),
          function_calls,
          params.user_interaction_requests.clone(),
        )
        .await
      }
      ToolId::Terminal => {
        TerminalHandler::handle(
          params.turn_id,
          params.step_id,
          params.tool_use_id.clone(),
          healed_args,
          params.runtime_context.clone(),
          params.terminal_manager.clone(),
          function_calls,
        )
        .await
      }
      _ => {
        BasicToolHandler::handle(
          params.turn_id,
          params.step_id,
          params.tool_id.clone(),
          params.tool_use_id.clone(),
          healed_args,
          params.signature.clone(),
          params.runtime_context.clone(),
          params.config.sandbox_key.clone(),
          function_calls,
        )
        .await
      }
    };

    match result {
      Ok(result) => result,
      Err(e) => {
        Self::handle_tool_call_error(
          params.turn_id,
          params.step_id,
          params.tool_id,
          params.tool_use_id,
          params.args,
          params.signature,
          params.runtime_context,
          function_calls,
          e.to_string(),
        )
        .await
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn handle_tool_call_error(
    turn_id: Uuid,
    step_id: Uuid,
    tool_id: ToolId,
    tool_use_id: String,
    args: String,
    signature: Option<String>,
    runtime_context: Arc<RuntimeContext>,
    function_calls: &mut JoinSet<ToolResult>,
    error: String,
  ) {
    let parent_id = match MessageRepositoryV2::first_by_step_id(step_id).await {
      Ok(Some(h)) => Some(h.id),
      _ => None,
    };

    let args = match serde_json::from_str::<Value>(&args) {
      Ok(args) => args,
      Err(_) => serde_json::json!({ "__MALFORMED_JSON__": args }),
    };

    if let Ok(id) = Session::init_tool_request(
      &runtime_context.session_id,
      tool_use_id.clone(),
      turn_id,
      step_id,
      tool_id.clone(),
      args,
      signature.clone(),
      parent_id,
      None,
    )
    .await
    {
      function_calls.spawn(async move {
        ToolResult { history_id: id.clone(), tool_use_id, result: ToolUseResponseError::error(tool_id, error) }
      });
    };
  }
}
