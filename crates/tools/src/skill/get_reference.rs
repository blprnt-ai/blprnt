use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::skills_utils::SkillsUtils;
use common::tools::GetReferenceArgs;
use common::tools::GetReferencePayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct GetReferenceTool {
  pub args: GetReferenceArgs,
}

#[async_trait]
impl Tool for GetReferenceTool {
  fn tool_id(&self) -> ToolId {
    ToolId::GetReference
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let current_skills = context.current_skills.clone();

    if current_skills.is_empty() {
      return Err(anyhow::anyhow!("No current skill loaded.",));
    }

    for skill_name in current_skills {
      if let Ok(content) = SkillsUtils::get_skill_references(&skill_name, &self.args.reference_path) {
        return Ok(ToolUseResponseData::success(GetReferencePayload { content }.into()));
      }
    }

    Err(anyhow::anyhow!("Reference '{}' not found under any active skill.", self.args.reference_path))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::GetReference, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(GetReferenceArgs);
    let json = serde_json::to_value(&schema).expect("[GetReferenceArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[GetReferenceArgs] properties is required"),
      "required": json.get("required").expect("[GetReferenceArgs] required is required")
    });

    let name = schema.get("title").expect("[GetReferenceArgs] title is required").clone();
    let description = schema.get("description").expect("[GetReferenceArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
