use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolAllowList;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseError;
use shared::tools::config::ToolsSchemaConfig;

use crate::tool_use::ToolUseContext;

#[async_trait]
pub trait Tool: Send + Sync {
  fn tool_id(&self) -> ToolId;

  fn name(&self) -> String {
    self.tool_id().to_string()
  }

  async fn maybe_invoke(&self, context: ToolUseContext) -> ToolUseResponse {
    if !ToolAllowList::is_tool_allowed_and_enabled_for_runtime(self.tool_id(), context.agent_kind) {
      return ToolUseResponseError::error(
        self.tool_id(),
        ToolError::AccessDenied { agent_kind: context.agent_kind, tool_id: self.tool_id() },
      );
    }

    match self.run(context).await {
      Ok(response) => response,
      Err(e) => ToolUseResponseError::error(self.tool_id(), e),
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse>;

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec>;
}
