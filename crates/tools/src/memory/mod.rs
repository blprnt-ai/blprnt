mod search;
mod write;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;

pub use self::search::*;
pub use self::write::*;
use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Memory {
  Write(WriteMemoryTool),
  Search(SearchMemoryTool),
}

#[async_trait]
impl Tool for Memory {
  fn tool_id(&self) -> ToolId {
    match self {
      Memory::Write(_) => ToolId::MemoryWrite,
      Memory::Search(_) => ToolId::MemorySearch,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Memory::Write(tool) => tool.run(context).await,
      Memory::Search(tool) => tool.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(WriteMemoryTool::schema(config));
    schema.extend(SearchMemoryTool::schema(config));
    schema
  }
}
