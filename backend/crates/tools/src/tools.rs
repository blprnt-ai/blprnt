use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolId;
use shared::errors::ToolError;
use shared::tools::prelude::*;

use crate::Tool;
use crate::ToolSpec;
use crate::prelude::*;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Tools {
  File(File),
  Host(Host),
}

#[async_trait]
impl Tool for Tools {
  fn tool_id(&self) -> ToolId {
    match self {
      Tools::File(File::FilesRead(_)) => ToolId::FilesRead,
      Tools::File(File::ApplyPatch(_)) => ToolId::ApplyPatch,
      Tools::Host(Host::Shell(_)) => ToolId::Shell,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Tools::File(cmd) => cmd.run(context).await,
      Tools::Host(cmd) => cmd.run(context).await,
    }
  }

  fn schema() -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(File::schema());
    schema.extend(Host::schema());

    schema
  }
}

impl TryFrom<(&ToolId, &str)> for Tools {
  type Error = anyhow::Error;

  fn try_from((tool_id, args): (&ToolId, &str)) -> Result<Self> {
    match tool_id {
      ToolId::FilesRead => {
        let args = serde_json::from_str::<FilesReadArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::File(File::FilesRead(FilesReadTool { args })))
      }
      ToolId::ApplyPatch => {
        let args = serde_json::from_str::<ApplyPatchArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::File(File::ApplyPatch(ApplyPatchTool { args })))
      }
      ToolId::Shell => {
        let args = serde_json::from_str::<ShellArgs>(args)
          .map_err(|e| ToolError::InvalidArgs { tool_id: tool_id.clone(), error: e.to_string() })?;
        Ok(Tools::Host(Host::Shell(ShellTool { args })))
      }
      _ => Err(ToolError::UnknownTool(tool_id.to_string()).into()),
    }
  }
}
