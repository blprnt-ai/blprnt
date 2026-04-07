use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::McpServerId;
use persistence::prelude::McpServerRepository;
use persistence::prelude::RunEnabledMcpServerRepository;
use shared::agent::ToolId;
use shared::tools::EnableMcpServerArgs;
use shared::tools::EnableMcpServerPayload;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct EnableMcpServerTool {
  pub args: EnableMcpServerArgs,
}

#[async_trait]
impl Tool for EnableMcpServerTool {
  fn tool_id(&self) -> ToolId {
    ToolId::EnableMcpServer
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let run_id = context
      .runtime_config
      .run_id
      .as_deref()
      .context("enable_mcp_server requires a run-scoped runtime context")?;
    let run_id = Uuid::parse_str(run_id).context("invalid runtime run id")?;
    let server_id = Uuid::parse_str(&self.args.server_id).context("invalid server id")?;

    let run_id = run_id.into();
    let server_id: McpServerId = server_id.into();

    let server = McpServerRepository::get(server_id.clone()).await?;
    RunEnabledMcpServerRepository::enable(run_id, server_id).await?;

    Ok(ToolUseResponseData::success(ToolUseResponseData::EnableMcpServer(EnableMcpServerPayload {
      server_id: server.id.uuid().to_string(),
      display_name: server.display_name,
      description: server.description,
      auth_state: server.auth_state,
    })))
  }

  fn schema() -> Vec<ToolSpec> {
    let schema = schemars::schema_for!(EnableMcpServerArgs);
    let json = serde_json::to_value(&schema).expect("[EnableMcpServerArgs] schema is required");
    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[EnableMcpServerArgs] properties are required"),
      "required": json.get("required").cloned().unwrap_or_else(|| serde_json::json!([]))
    });
    let name = json.get("title").expect("[EnableMcpServerArgs] title is required").clone();
    let description = json.get("description").expect("[EnableMcpServerArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}