use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::skills_utils::SkillsUtils;
use common::tools::ListSkillsArgs;
use common::tools::ListSkillsPayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct ListSkillsTool {
  pub args: ListSkillsArgs,
}

#[async_trait]
impl Tool for ListSkillsTool {
  fn tool_id(&self) -> ToolId {
    ToolId::ListSkills
  }

  async fn run(&self, _context: ToolUseContext) -> Result<ToolUseResponse> {
    let skills = SkillsUtils::list_skills()?;

    Ok(ToolUseResponseData::success(ListSkillsPayload { items: skills }.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::ListSkills, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(ListSkillsArgs);

    let params = serde_json::json!({
      "type": "object",
    });

    let name = schema.get("title").expect("[ListSkillsArgs] title is required").clone();
    let description = schema.get("description").expect("[ListSkillsArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
