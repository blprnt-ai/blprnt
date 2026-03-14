use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::errors::ToolError;
use common::plan_utils::resolve_plan_directory;
use common::tools::PlanDeleteArgs;
use common::tools::PlanDeletePayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct PlanDeleteTool {
  pub args: PlanDeleteArgs,
}

#[async_trait]
impl Tool for PlanDeleteTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PlanDelete
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let plan_directory = resolve_plan_directory(context.project_id)?;
    let base_path = PathBuf::from(&plan_directory.path);
    let path = base_path.join(&self.args.id);

    std::fs::remove_file(&path)
      .map_err(|e| ToolError::FileWriteFailed { path: path.display().to_string(), error: e.to_string() })?;

    let payload = PlanDeletePayload { id: self.args.id.clone() };
    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanDelete, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(PlanDeleteArgs);
    let json = serde_json::to_value(&schema).expect("[PlanDeleteArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[PlanDeleteArgs] properties is required"),
      "required": json.get("required").expect("[PlanDeleteArgs] required is required")
    });

    let name = schema.get("title").expect("[PlanDeleteArgs] title is required").clone();
    let description = schema.get("description").expect("[PlanDeleteArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
