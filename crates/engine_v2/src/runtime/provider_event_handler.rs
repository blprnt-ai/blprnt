use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use common::agent::ToolId;
use common::errors::ProviderError;
use common::provider_dispatch::ProviderEvent;
use common::session_dispatch::prelude::*;
use common::shared::prelude::*;
use common::tools::ToolResult;
use persistence::prelude::MessagePatchV2;
use persistence::prelude::MessageRepositoryV2;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

use crate::prelude::ControllerConfig;
use crate::runtime::ControlFlow;
use crate::runtime::TerminalManagers;
use crate::runtime::UserInteraction;
use crate::runtime::context::RuntimeContext;
use crate::runtime::tool_call_handler::ToolCallHandler;
use crate::runtime::tool_call_handler::ToolCallHandlerParams;

pub struct ProviderEventHandler {
  config:                    ControllerConfig,
  user_interaction_requests: Arc<Mutex<HashMap<String, oneshot::Sender<UserInteraction>>>>,
  terminal_managers:         TerminalManagers,
}

impl ProviderEventHandler {
  pub fn new(
    config: ControllerConfig,
    user_interaction_requests: Arc<Mutex<HashMap<String, oneshot::Sender<UserInteraction>>>>,
    terminal_managers: TerminalManagers,
  ) -> Arc<Self> {
    Arc::new(Self { config, user_interaction_requests, terminal_managers })
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn process_provider_events(
    self: Arc<Self>,
    turn_id: Uuid,
    step_id: Uuid,
    runtime_context: Arc<RuntimeContext>,
    event: ProviderEvent,
    function_calls: &mut JoinSet<ToolResult>,
    last_reasoning_id: &mut Option<SurrealId>,
    last_response_id: &mut Option<SurrealId>,
    source_provider: Provider,
  ) -> Result<ControlFlow> {
    match event {
      ProviderEvent::Start(_) => {
        return Ok(ControlFlow::Continue);
      }
      ProviderEvent::Stop(_) => {
        return Ok(ControlFlow::EndStep);
      }

      ProviderEvent::Ping => return Ok(ControlFlow::Continue),

      ProviderEvent::ReasoningStarted { rel_id } => {
        let parent_id = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id);
        let id = Session::append_assistant_reasoning(
          &runtime_context.session_id,
          rel_id.clone(),
          turn_id,
          step_id,
          "".to_string(),
          None,
          parent_id,
          source_provider,
        )
        .await?;

        *last_reasoning_id = Some(id.clone());
        runtime_context.session_dispatch.send(ReasoningStarted { id, turn_id, step_id }.into()).await
      }
      ProviderEvent::Reasoning { rel_id, reasoning, signature } => {
        let id = Session::update_assistant_reasoning(
          rel_id.to_string(),
          reasoning.clone(),
          signature.unwrap_or_default(),
          source_provider,
        )
        .await?;

        *last_reasoning_id = None;
        runtime_context.session_dispatch.send(ReasoningFinal { id, reasoning }.into()).await
      }
      ProviderEvent::ReasoningDelta { rel_id, delta } => {
        let id = if let Some(id) = last_reasoning_id {
          id.clone()
        } else {
          let id = Session::get_assistant_reasoning_id(rel_id.to_string()).await?;
          *last_reasoning_id = Some(id.clone());
          id
        };
        runtime_context.session_dispatch.send(ReasoningTextDelta { id, delta }.into()).await
      }
      ProviderEvent::ReasoningDone { rel_id } => {
        let id = Session::complete_assistant_reasoning(rel_id.to_string()).await?;

        *last_reasoning_id = None;
        runtime_context.session_dispatch.send(ReasoningDone { id }.into()).await
      }

      ProviderEvent::ResponseStarted { rel_id } => {
        let parent_id = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id);
        let id = Session::append_assistant_message(
          &runtime_context.session_id,
          rel_id.clone(),
          turn_id,
          step_id,
          "".to_string(),
          None,
          parent_id,
        )
        .await?;

        *last_response_id = Some(id.clone());
        runtime_context.session_dispatch.send(ResponseStarted { id, turn_id, step_id }.into()).await
      }
      ProviderEvent::Response { rel_id, content, signature } => {
        let id = Session::update_assistant_message(rel_id.to_string(), content.clone(), signature).await?;

        *last_response_id = None;
        runtime_context.session_dispatch.send(Response { id, content }.into()).await
      }
      ProviderEvent::ResponseDelta { rel_id, delta } => {
        let id = if let Some(id) = last_response_id {
          id.clone()
        } else {
          let id = Session::get_assistant_message_id(rel_id.to_string()).await?;
          *last_response_id = Some(id.clone());
          id
        };
        runtime_context.session_dispatch.send(ResponseDelta { id, delta }.into()).await
      }
      ProviderEvent::ResponseDone { rel_id } => {
        let id = Session::get_assistant_message_id(rel_id.to_string()).await?;

        *last_response_id = None;
        runtime_context.session_dispatch.send(ResponseDone { id }.into()).await
      }

      ProviderEvent::Status(status) => runtime_context.session_dispatch.send(Status { status }.into()).await,
      ProviderEvent::TokenUsage(token_usage) => {
        if let Some(parent_id) = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id) {
          let patch = MessagePatchV2 { token_usage: Some(token_usage), ..Default::default() };
          let _ = MessageRepositoryV2::update(parent_id, patch).await;
        };

        runtime_context.session_dispatch.send(TokenUsage { input_tokens: token_usage, output_tokens: 0 }.into()).await
      }
      ProviderEvent::Error(error) => {
        match error {
          ProviderError::InvalidToolId { call_id, tool_id, arguments, message, .. } => {
            ToolCallHandler::handle_tool_call_error(
              turn_id,
              step_id,
              ToolId::Unknown(tool_id),
              call_id,
              arguments,
              None,
              runtime_context.clone(),
              function_calls,
              message,
            )
            .await;
          }
          ProviderError::LlmError { context, message } => {
            let signal = SignalPayload::error_from(&ProviderError::LlmError { context, message: message.clone() });
            let id = Session::append_signal_to_user(&runtime_context.session_id, signal.clone().into()).await?;
            let _ = runtime_context.session_dispatch.send(signal.with_id(id).into()).await;

            let message = format!(
              "<system-message>\nAn error occurred while processing the request.\nIf it is recoverable, continue the conversation. If not, let the user know and end the turn.\n</system-message>\n\nError: {message}"
            );
            let signal = SignalPayload::error(message);
            let _ = Session::append_signal_to_assistant(&runtime_context.session_id, signal.clone().into()).await;

            return Ok(ControlFlow::Continue);
          }
          error => {
            let signal = SignalPayload::error_from(&error);
            let id = Session::append_signal_to_user(&runtime_context.session_id, signal.clone().into()).await?;
            let _ = runtime_context.session_dispatch.send(signal.with_id(id).into()).await;

            return Ok(ControlFlow::EndTurn);
          }
        }

        Ok(())
      }

      ProviderEvent::ToolCall { tool_id, tool_use_id, args, signature } => {
        let params = ToolCallHandlerParams {
          config:                    self.config.clone(),
          turn_id:                   turn_id,
          step_id:                   step_id,
          tool_id:                   tool_id,
          tool_use_id:               tool_use_id,
          args:                      args,
          signature:                 signature,
          runtime_context:           runtime_context,
          terminal_manager:          self.terminal_managers.clone(),
          user_interaction_requests: self.user_interaction_requests.clone(),
        };

        ToolCallHandler::handle(params, function_calls).await;

        Ok(())
      }
    }?;

    Ok(ControlFlow::Continue)
  }
}
