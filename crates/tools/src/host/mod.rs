#[cfg(target_os = "windows")]
mod baldr;
#[cfg(target_os = "linux")]
mod loki;
#[cfg(target_os = "macos")]
mod thor;

mod child;
pub mod env;

pub mod shell;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;

pub use self::shell::ShellTool;
use crate::Tool;
use crate::ToolSpec;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Host {
  Shell(ShellTool),
}

#[async_trait]
impl Tool for Host {
  fn tool_id(&self) -> ToolId {
    match self {
      Host::Shell(_) => ToolId::Shell,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Host::Shell(proc) => Tool::run(proc, context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(ShellTool::schema(config));

    schema
  }
}
