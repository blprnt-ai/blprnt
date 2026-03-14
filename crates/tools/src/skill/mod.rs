pub mod get_reference;
pub mod list_skills;
pub mod skill_script;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolId;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::config::ToolsSchemaConfig;
pub use get_reference::*;
pub use list_skills::*;
pub use skill_script::*;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub enum Skill {
  ListSkills(ListSkillsTool),
  GetReference(GetReferenceTool),
  SkillScript(SkillScriptTool),
}

#[async_trait]
impl Tool for Skill {
  fn tool_id(&self) -> ToolId {
    match self {
      Self::ListSkills(_) => ToolId::ListSkills,
      Self::GetReference(_) => ToolId::GetReference,
      Self::SkillScript(_) => ToolId::SkillScript,
    }
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    match self {
      Self::ListSkills(tool) => tool.run(context).await,
      Self::GetReference(tool) => tool.run(context).await,
      Self::SkillScript(tool) => tool.run(context).await,
    }
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    let mut schema = Vec::new();
    schema.extend(ListSkillsTool::schema(config));
    schema.extend(GetReferenceTool::schema(config));
    schema.extend(SkillScriptTool::schema(config));

    schema
  }
}
