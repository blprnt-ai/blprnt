pub mod skill_script;

use anyhow::Result;
use async_trait::async_trait;
use shared::agent::ToolId;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::config::ToolsSchemaConfig;
pub use skill_script::*;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Skill {
  SkillScript(SkillScriptTool),
}

#[async_trait]
impl Tool for Skill {
  fn tool_id(&self) -> ToolId {
    match self {
      Self::SkillScript(_) => ToolId::SkillScript,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Self::SkillScript(tool) => tool.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(SkillScriptTool::schema(config));

    schema
  }
}
