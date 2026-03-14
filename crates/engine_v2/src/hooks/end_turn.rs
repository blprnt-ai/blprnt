use std::sync::Arc;

use anyhow::Result;
use common::session_dispatch::prelude::*;
use common::shared::prelude::SubAgentStatus;
use common::tools::ToolResult;
use common::tools::ToolUseResponseError;
use persistence::prelude::MessageRepositoryV2;
use session::Session;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;

#[derive(Clone, Debug)]
pub struct EndTurn;

#[async_trait::async_trait]
impl Hook for EndTurn {
  fn name(&self) -> String {
    "EndTurn".to_string()
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    if runtime_context.cancel_token.is_cancelled() {
      tokio::spawn({
        let runtime_context = runtime_context.clone();
        async move {
          let pending_requests = MessageRepositoryV2::get_pending_tool_requests(runtime_context.session_id.clone())
            .await
            .unwrap_or_default()
            .into_iter();

          for (history_id, tool_use_id, tool_id, turn_id, step_id, subagent_details) in pending_requests {
            let completed = Session::complete_tool_request(tool_use_id.clone()).await;
            if !completed {
              continue;
            }

            let parent_id =
              MessageRepositoryV2::first_by_step_id(step_id).await.unwrap_or_default().map(|history| history.id);
            let subagent_id = subagent_details.map(|details| details.session_id);
            let result = ToolUseResponseError::error_with_subagent_status(
              tool_id.clone(),
              "User canceled",
              subagent_id,
              SubAgentStatus::Failure,
            );
            let tool_result = ToolResult { history_id: history_id.clone(), tool_use_id: tool_use_id.clone(), result };

            let tool_result_id = Session::append_cancelled_tool_response(
              &runtime_context.session_id,
              turn_id,
              step_id,
              tool_result.clone(),
              parent_id,
            )
            .await
            .unwrap_or_default();

            runtime_context
              .session_dispatch
              .send(
                ToolCallCompleted {
                  id:      tool_result_id.to_string(),
                  item_id: tool_use_id.clone(),
                  content: tool_result.result,
                }
                .into(),
              )
              .await
              .unwrap_or_default();
          }
        }
      });
    }

    let event = ControlEvent::TurnStop;
    tracing::info!("EndTurn: sending turn stop event: {:?}", runtime_context.session_id);
    let _ = runtime_context.session_dispatch.send(event.into()).await?;

    Ok(())
  }
}
