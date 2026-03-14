use std::sync::Arc;

use anyhow::Result;
use common::agent::AgentKind;
use common::agent::ToolId;
use common::sandbox_flags::BoolValueSource;
use common::sandbox_flags::SandboxFlags;
use common::session_dispatch::prelude::*;
use common::shared::prelude::parse_mcp_tool_runtime_name;
use common::shared::prelude::*;
use common::tools::McpToolPayload;
use common::tools::ToolResult;
use common::tools::ToolUseResponseData;
use common::tools::ToolUseResponseError;
use persistence::prelude::MessageRepositoryV2;
use persistence::prelude::SessionRepositoryV2;
use serde_json::Value;
use session::Session;
use surrealdb::types::Uuid;
use tokio::task::JoinSet;
use tools::Tool;
use tools::Tools;
use tools::tool_use::ToolUseContext;

use crate::runtime::context::RuntimeContext;

#[derive(Debug)]
pub struct BasicToolHandler;

impl BasicToolHandler {
  fn subagent_plan_tool_rejection(tool_id: &ToolId, agent_kind: AgentKind, is_subagent: bool) -> Option<String> {
    if !is_subagent {
      return None;
    }

    match tool_id {
      ToolId::PlanCreate | ToolId::PlanUpdate if agent_kind != AgentKind::Planner => Some(format!(
        "Subagent '{}' is read-only for plan mutations. Use a planning subagent for {}.",
        agent_kind, tool_id
      )),
      ToolId::PlanDelete => Some(
        "Subagents cannot delete plans. Detach from subagent workflow and delete from parent session tooling."
          .to_string(),
      ),
      _ => None,
    }
  }

  fn mcp_target(tool_id: &ToolId) -> Option<(String, String, String)> {
    let ToolId::Mcp(runtime_name) = tool_id else {
      return None;
    };

    let (server_id, tool_name) = parse_mcp_tool_runtime_name(runtime_name)?;
    Some((runtime_name.clone(), server_id, tool_name))
  }

  async fn dispatch_mcp_tool_call(
    tool_id: ToolId,
    runtime_name: String,
    server_id: String,
    tool_name: String,
    healed_args: Value,
    mcp_runtime: Option<McpRuntimeBridgeRef>,
  ) -> common::tools::ToolUseResponse {
    match mcp_runtime {
      Some(mcp_runtime) => {
        let call_result = mcp_runtime.call_tool(server_id.clone(), tool_name, healed_args).await;
        match call_result {
          Ok(payload) => ToolUseResponseData::success(ToolUseResponseData::McpTool(McpToolPayload {
            server_id,
            name: runtime_name,
            result: payload,
          })),
          Err(error) => ToolUseResponseError::error(tool_id, error),
        }
      }
      None => ToolUseResponseError::error(tool_id, "MCP runtime is unavailable for this session"),
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn handle(
    turn_id: Uuid,
    step_id: Uuid,
    tool_id: ToolId,
    tool_use_id: String,
    healed_args: Value,
    signature: Option<String>,
    runtime_context: Arc<RuntimeContext>,
    sandbox_key: String,
    function_calls: &mut JoinSet<ToolResult>,
  ) -> Result<()> {
    let parent_id = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id);

    let id = Session::init_tool_request(
      &runtime_context.session_id,
      tool_use_id.clone(),
      turn_id,
      step_id,
      tool_id.clone(),
      healed_args.clone(),
      signature.clone(),
      parent_id,
      None,
    )
    .await?;

    runtime_context
      .session_dispatch
      .send(
        ToolCallStarted {
          id: id.clone().to_string(),
          turn_id,
          step_id,
          tool_id: tool_id.clone(),
          args: healed_args.clone(),
          question_id: None,
          subagent_details: None,
        }
        .into(),
      )
      .await?;

    function_calls.spawn(async move {
      if let Some((runtime_name, server_id, tool_name)) = Self::mcp_target(&tool_id) {
        let result = Self::dispatch_mcp_tool_call(
          tool_id.clone(),
          runtime_name,
          server_id,
          tool_name,
          healed_args.clone(),
          runtime_context.mcp_runtime.clone(),
        )
        .await;

        return ToolResult { history_id: id.clone(), tool_use_id, result };
      }

      let tool = match Tools::try_from((&tool_id, healed_args.to_string().as_str())) {
        Ok(tool) => tool,
        Err(e) => {
          let message = format!("{}: {}", tool_id, e);

          return ToolResult {
            history_id: id.clone(),
            tool_use_id,
            result: ToolUseResponseError::error(tool_id, message),
          };
        }
      };

      let working_directories = match Session::working_directories(&runtime_context.project_id).await {
        Ok(working_directories) => working_directories,
        Err(e) => {
          let message = format!("Failed to get working directories: {}", e);

          return ToolResult {
            history_id: id.clone(),
            tool_use_id,
            result: ToolUseResponseError::error(tool_id, message),
          };
        }
      };

      let session_model = match SessionRepositoryV2::get(runtime_context.session_id.clone()).await {
        Ok(session_model) => session_model,
        Err(e) => {
          let message = format!("Failed to get session settings: {}", e);
          return ToolResult {
            history_id: id.clone(),
            tool_use_id,
            result: ToolUseResponseError::error(tool_id, message),
          };
        }
      };

      let mut sandbox_flags = SandboxFlags::default();
      sandbox_flags.set_network_access(*session_model.network_access(), BoolValueSource::Engine);
      sandbox_flags.set_read_only(*session_model.read_only(), BoolValueSource::Engine);
      sandbox_flags.set_yolo(*session_model.yolo(), BoolValueSource::Engine);

      let project_id = session_model.project.clone();

      let tool_use_context = ToolUseContext::new_with_memory_tools_enabled(
        runtime_context.session_id.clone(),
        session_model.parent_id.clone(),
        project_id,
        *session_model.agent_kind(),
        working_directories,
        runtime_context.current_skills().await.unwrap_or_default(),
        sandbox_flags,
        sandbox_key.clone(),
        runtime_context.is_subagent,
        runtime_context.memory_tools_enabled,
      );

      if let Some(error) =
        Self::subagent_plan_tool_rejection(&tool_id, *session_model.agent_kind(), runtime_context.is_subagent)
      {
        return ToolResult { history_id: id.clone(), tool_use_id, result: ToolUseResponseError::error(tool_id, error) };
      }

      let result = tool.maybe_invoke(tool_use_context).await;

      ToolResult { history_id: id.clone(), tool_use_id, result }
    });

    Ok(())
  }
}
