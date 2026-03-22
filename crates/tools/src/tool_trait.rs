use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolId;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseError;

use crate::tool_use::ToolUseContext;

#[async_trait]
pub trait Tool: Send + Sync {
  fn tool_id(&self) -> ToolId;

  fn name(&self) -> String {
    self.tool_id().to_string()
  }

  async fn maybe_invoke(&self, context: ToolUseContext) -> ToolUseResponse {
    match self.run(context).await {
      Ok(response) => response,
      Err(e) => ToolUseResponseError::error(self.tool_id(), e),
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse>;

  fn schema() -> Vec<ToolSpec>;
}
