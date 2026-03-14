mod files_read;
mod patch;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;

pub use self::files_read::FilesReadTool;
pub use self::patch::ApplyPatchTool;
use crate::Tool;
use crate::ToolSpec;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum File {
  FilesRead(FilesReadTool),
  ApplyPatch(ApplyPatchTool),
}

#[async_trait]
impl Tool for File {
  fn tool_id(&self) -> ToolId {
    match self {
      File::FilesRead(_) => ToolId::FilesRead,
      File::ApplyPatch(_) => ToolId::ApplyPatch,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      File::FilesRead(cmd) => cmd.run(context).await,
      File::ApplyPatch(cmd) => cmd.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(FilesReadTool::schema(config));
    schema.extend(ApplyPatchTool::schema(config));

    schema
  }
}
