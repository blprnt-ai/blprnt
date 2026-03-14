use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use common::agent::ToolId;
use common::session_dispatch::prelude::*;
use common::tools::ToolResult;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseError;
use common::tools::ToolUseResponseSuccess;
use common::tools::question::AskQuestionPayload;
use serde_json::Value;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::runtime::UserInteraction;
use crate::runtime::context::RuntimeContext;

#[derive(Debug)]
pub struct AskQuestionHandler;

impl AskQuestionHandler {
  #[allow(clippy::too_many_arguments)]
  pub async fn handle(
    turn_id: Uuid,
    step_id: Uuid,
    tool_use_id: String,
    healed_args: Value,
    signature: Option<String>,
    runtime_context: Arc<RuntimeContext>,
    function_calls: &mut JoinSet<ToolResult>,
    user_interaction_requests: Arc<Mutex<HashMap<String, oneshot::Sender<UserInteraction>>>>,
  ) -> Result<()> {
    let id = Session::init_tool_request(
      &runtime_context.session_id,
      tool_use_id.clone(),
      turn_id,
      step_id,
      ToolId::AskQuestion,
      healed_args.clone(),
      signature.clone(),
      None,
      None,
    )
    .await?;

    let (tx, rx) = oneshot::channel();
    user_interaction_requests.lock().await.insert(tool_use_id.clone(), tx);

    runtime_context
      .session_dispatch
      .send(
        ToolCallStarted {
          id:               id.to_string(),
          turn_id:          turn_id,
          step_id:          step_id,
          tool_id:          ToolId::AskQuestion,
          args:             healed_args.clone(),
          question_id:      Some(tool_use_id.clone()),
          subagent_details: None,
        }
        .into(),
      )
      .await?;

    function_calls.spawn(async move {
      let result = rx.await;

      let result = match result {
        Ok(UserInteraction::QuestionAnswer(answer)) => ToolUseResponse::Success(ToolUseResponseSuccess {
          success: true,
          data:    AskQuestionPayload { answer }.into(),
          message: None,
        }),
        Err(e) => ToolUseResponseError::error(ToolId::AskQuestion, e),
      };

      ToolResult { history_id: id.clone(), tool_use_id: tool_use_id.clone(), result: result }
    });

    Ok(())
  }
}
