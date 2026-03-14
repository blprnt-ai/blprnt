mod search;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;

pub use self::search::*;
use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Rg {
  Search(RgSearchTool),
}

#[async_trait]
impl Tool for Rg {
  fn tool_id(&self) -> ToolId {
    match self {
      Rg::Search(_) => ToolId::Rg,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Rg::Search(tool) => tool.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(RgSearchTool::schema(config));
    schema
  }
}
