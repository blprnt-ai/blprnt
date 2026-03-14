use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::plan_utils::get_plan_content;
use common::tools::PlanGetArgs;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct PlanGetTool {
  pub args: PlanGetArgs,
}

#[async_trait]
impl Tool for PlanGetTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PlanGet
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let payload = get_plan_content(context.project_id, self.args.id.clone())?;
    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanGet, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(PlanGetArgs);
    let json = serde_json::to_value(&schema).expect("[PlanGetArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[PlanGetArgs] properties is required"),
      "required": json.get("required").expect("[PlanGetArgs] required is required")
    });

    let name = schema.get("title").expect("[PlanGetArgs] title is required").clone();
    let description = schema.get("description").expect("[PlanGetArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
