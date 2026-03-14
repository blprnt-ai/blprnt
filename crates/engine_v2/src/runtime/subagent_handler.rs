use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use common::agent::ToolId;
use common::blprnt_dispatch::BlprntDispatch;
use common::errors::EngineError;
use common::session_dispatch::prelude::*;
use common::shared::prelude::*;
use common::tools::SubAgentArgs;
use common::tools::SubAgentPayload;
use common::tools::ToolResult;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::ToolUseResponseError;
use persistence::prelude::SessionModelV2;
use persistence::prelude::SessionRepositoryV2;
use serde_json::Value;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::task::JoinSet;

use crate::prelude::Controller;
use crate::prelude::ControllerConfig;
use crate::runtime::context::RuntimeContext;

struct ToolRequestParams {
  tool_use_id:      String,
  turn_id:          Uuid,
  step_id:          Uuid,
  args:             Value,
  subagent_details: SubagentDetails,
}

#[derive(Debug)]
pub struct SubagentHandler;

impl SubagentHandler {
  #[allow(clippy::too_many_arguments)]
  pub async fn handle(
    config: ControllerConfig,
    turn_id: Uuid,
    step_id: Uuid,
    tool_use_id: String,
    healed_args: Value,
    runtime_context: Arc<RuntimeContext>,
    function_calls: &mut JoinSet<ToolResult>,
  ) -> Result<()> {
    let parent_id = config.parent_id.clone().unwrap_or(runtime_context.session_id.clone());
    let args: SubAgentArgs = serde_json::from_value(healed_args.clone())
      .map_err(|e| EngineError::FailedToParseToolArgs(format!("SubAgent: {}", e)))?;

    let session_id = Self::get_session_id(args.clone(), parent_id.clone()).await?;
    let sandbox_key = config.sandbox_key.clone();

    let subagent_details = SubagentDetails {
      session_id:        session_id.clone().to_string(),
      parent_session_id: Some(parent_id.clone().to_string()),
    };

    let tool_request_params = ToolRequestParams {
      tool_use_id:      tool_use_id.clone(),
      turn_id:          turn_id,
      step_id:          step_id,
      args:             healed_args.clone(),
      subagent_details: subagent_details.clone(),
    };
    let id = Self::init_tool_request(runtime_context.clone(), tool_request_params).await?;

    function_calls.spawn(async move {
      let (tx, rx) = oneshot::channel();
      let (activity_tx, activity_rx) = watch::channel(Instant::now());
      Self::setup_subagent_listener(session_id.clone(), tx, activity_tx.clone());

      let config = Self::controller_config(sandbox_key, session_id.clone(), parent_id, runtime_context.clone()).await;

      let _ = activity_tx.send(Instant::now());
      Self::push_prompt(config, runtime_context.clone(), args.prompt).await;

      let tool_result = Self::run_subagent(rx, session_id, runtime_context.clone(), activity_rx).await;

      ToolResult { history_id: id.clone(), tool_use_id, result: tool_result }
    });

    Ok(())
  }

  async fn get_session_id(args: SubAgentArgs, parent_id: SurrealId) -> Result<SurrealId> {
    let session_id = if let Some(session_id) = args.subagent_id.clone() {
      let session_id = parse_session_id(session_id)?;

      let session_model = SessionRepositoryV2::get(session_id).await?;

      session_model.id.clone()
    } else {
      let parent_session_model = SessionRepositoryV2::get(parent_id.clone()).await?.clone();
      let parent_session_id = parent_session_model.id.clone();
      let parent_session_model_override = parent_session_model.model_override.clone();

      let model_override = if let Some(model_override) = args.model_override
        && model_override != "null"
      {
        model_override
      } else {
        parent_session_model_override
      };

      let project_id = parent_session_model.project.clone();
      let mut new_session_model: SessionModelV2 = parent_session_model.into();
      new_session_model.name = args.name;
      new_session_model.agent_kind = args.agent_kind;
      new_session_model.model_override = model_override;

      let new_session_model = SessionRepositoryV2::create(new_session_model, project_id).await?;
      SessionRepositoryV2::relate_parent(parent_session_id, new_session_model.id.clone()).await?;

      new_session_model.id.clone()
    };

    Ok(session_id)
  }

  async fn init_tool_request(runtime_context: Arc<RuntimeContext>, params: ToolRequestParams) -> Result<SurrealId> {
    let id = Session::init_tool_request(
      &runtime_context.session_id,
      params.tool_use_id.clone(),
      params.turn_id,
      params.step_id,
      ToolId::SubAgent,
      params.args.clone(),
      None,
      None,
      Some(params.subagent_details.clone()),
    )
    .await?;

    runtime_context
      .session_dispatch
      .send(
        ToolCallStarted {
          id:               id.to_string(),
          turn_id:          params.turn_id,
          step_id:          params.step_id,
          tool_id:          ToolId::SubAgent,
          args:             params.args.clone(),
          question_id:      None,
          subagent_details: Some(params.subagent_details.clone()),
        }
        .into(),
      )
      .await?;

    Ok(id)
  }

  fn setup_subagent_listener(
    session_id: SurrealId,
    tx: oneshot::Sender<Result<String, String>>,
    activity_tx: watch::Sender<Instant>,
  ) {
    tokio::spawn(async move {
      let mut llm_response_buffer: Vec<String> = Vec::new();

      loop {
        let event = BlprntDispatch::recv(session_id.clone()).await;
        let _ = activity_tx.send(Instant::now());
        match event.event_data {
          SessionDispatchEvent::Llm(LlmEvent::Response(Response { content, .. })) => llm_response_buffer.push(content),
          SessionDispatchEvent::Control(ControlEvent::TurnStop) => {
            tracing::info!("[TURNSTOP] Subagent response: {:?}", llm_response_buffer);

            let content = match llm_response_buffer.last() {
              Some(content) => content.clone(),
              None => "No response from subagent".to_string(),
            };
            let _ = tx.send(Ok(content));

            break;
          }
          SessionDispatchEvent::Signal(SignalEvent::Error(SignalPayload { message, .. })) => {
            tracing::info!("[SIGNAL] Subagent response: {:?}", llm_response_buffer);
            tracing::info!("[SIGNAL] Subagent error: {:?}", message);

            let _ = match llm_response_buffer.last() {
              Some(content) if !content.is_empty() => tx.send(Ok(content.clone())),
              _ => tx.send(Err(message)),
            };

            break;
          }
          _ => continue,
        }
      }
    });
  }

  async fn controller_config(
    sandbox_key: String,
    session_id: SurrealId,
    parent_id: SurrealId,
    runtime_context: Arc<RuntimeContext>,
  ) -> ControllerConfig {
    ControllerConfig {
      sandbox_key:          sandbox_key.clone(),
      is_subagent:          true,
      session_id:           session_id.clone(),
      parent_id:            Some(parent_id.clone()),
      mcp_runtime:          runtime_context.mcp_runtime.clone(),
      memory_tools_enabled: runtime_context.memory_tools_enabled,
    }
  }

  async fn push_prompt(config: ControllerConfig, runtime_context: Arc<RuntimeContext>, prompt: String) {
    let controller = Controller::new_with_cancel_token(config, runtime_context.cancel_token.child_token()).await;
    let controller = controller.clone();
    let controller = controller.read().await;

    let _ = controller.push_prompt(prompt, None).await;
  }

  async fn run_subagent(
    mut rx: oneshot::Receiver<Result<String, String>>,
    session_id: SurrealId,
    runtime_context: Arc<RuntimeContext>,
    mut activity_rx: watch::Receiver<Instant>,
  ) -> ToolUseResponse {
    let timeout = Duration::from_secs(500);
    let mut last_activity = *activity_rx.borrow();

    loop {
      let deadline = last_activity + timeout;
      let idle_timer = tokio::time::sleep_until(deadline.into());
      tokio::pin!(idle_timer);
      tokio::select! {
        result = &mut rx => {
          let outcome = match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(message)) => Err(message),
            Err(_) => Err("Subagent response channel dropped".to_string()),
          };

          let payload = outcome.map(|result| SubAgentPayload {
            result,
            subagent_id: Some(session_id.clone().key().to_string()),
          });

          return match payload {
            Ok(payload) => ToolUseResponseData::success(payload.into()),
            Err(message) => ToolUseResponseError::error_with_subagent_status(
              ToolId::SubAgent,
              message,
              Some(session_id.clone().key().to_string()),
              SubAgentStatus::Failure,
            ),
          };
        }
        _ = runtime_context.cancel_token.cancelled() => {
          return ToolUseResponseError::error_with_subagent_status(
            ToolId::SubAgent,
            "Subagent canceled by user",
            Some(session_id.clone().key().to_string()),
            SubAgentStatus::Failure,
          );
        }
        _ = &mut idle_timer => {
          // Check if last message from subagent is a response
          if let Some(content) = Self::check_for_subagent_response(&session_id).await {
            return ToolUseResponseData::success(SubAgentPayload {
              result: content,
              subagent_id: Some(session_id.clone().key().to_string()),
            }.into());
          }

          return ToolUseResponseError::error_with_subagent_status(
            ToolId::SubAgent,
            "Subagent timed out due to inactivity",
            Some(session_id.clone().key().to_string()),
            SubAgentStatus::Timeout,
          );
        }
        changed = activity_rx.changed() => {
          if changed.is_ok() {
            last_activity = *activity_rx.borrow();
          } else {
            return ToolUseResponseError::error_with_subagent_status(
              ToolId::SubAgent,
              "Subagent activity channel closed unexpectedly",
              Some(session_id.clone().key().to_string()),
              SubAgentStatus::Failure,
            );
          }
        }
      }
    }
  }

  async fn check_for_subagent_response(session_id: &SurrealId) -> Option<String> {
    if let Some(last_message) = Session::get_last_message(session_id).await
      && last_message.role == MessageRole::Assistant
      && last_message.content.is_text()
    {
      let content = last_message.content().as_text().unwrap_or_default();
      Some(content)
    } else {
      None
    }
  }
}

fn parse_session_id(session_id: String) -> Result<SurrealId, anyhow::Error> {
  let session_id = if SurrealId::looks_like(session_id.clone()) {
    session_id.clone().try_into()?
  } else if Uuid::from_str(&session_id).is_ok() {
    SurrealId::from((String::from("sessions"), Uuid::from_str(&session_id).unwrap()))
  } else if SurrealId::get_id_from_string(session_id.clone()).is_some() {
    SurrealId::get_id_from_string(session_id.clone()).unwrap()
  } else if SurrealId::get_uuid_from_string(session_id.clone()).is_some() {
    SurrealId::from((String::from("sessions"), SurrealId::get_uuid_from_string(session_id.clone()).unwrap()))
  } else {
    tracing::error!("Invalid subagent ID: {}", session_id);
    return Err(EngineError::InvalidSubagentId(session_id).into());
  };

  Ok(session_id)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_session_id() {
    let session_id = parse_session_id("019cde8e-fa2c-7ee2-abb3-0a60e1d538fd".to_string()).unwrap();
    println!("Session ID: {:?}", session_id);

    assert_eq!(
      session_id,
      SurrealId::from((String::from("sessions"), Uuid::from_str("019cde8e-fa2c-7ee2-abb3-0a60e1d538fd").unwrap()))
    );
  }
}
