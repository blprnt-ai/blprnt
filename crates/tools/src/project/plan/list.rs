use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::plan_utils::ensure_plan_dir;
use common::plan_utils::is_plan_file;
use common::plan_utils::parse_frontmatter;
use common::plan_utils::resolve_plan_directory;
use common::tools::PlanListArgs;
use common::tools::PlanListPayload;
use common::tools::PlanSummary;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct PlanListTool {
  pub args: PlanListArgs,
}

#[async_trait]
impl Tool for PlanListTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PlanList
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let plan_directory = resolve_plan_directory(context.project_id)?;
    let base_path = PathBuf::from(&plan_directory.path);
    ensure_plan_dir(&base_path)?;

    let mut items = Vec::new();
    let entries = std::fs::read_dir(&base_path)?;

    for entry in entries {
      let entry = entry?;
      let path = entry.path();
      if !path.is_file() || !is_plan_file(&path) {
        continue;
      }

      let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => continue,
      };

      let parsed = parse_frontmatter(&content);
      let (frontmatter, _) = match parsed {
        Ok(result) => result,
        Err(_) => continue,
      };

      let meta = frontmatter.into_meta();
      items.push(PlanSummary {
        id:          path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        name:        meta.name,
        description: meta.description,
        created_at:  meta.created_at,
        updated_at:  meta.updated_at,
      });
    }

    items.sort_by(|a, b| a.name.cmp(&b.name));
    let payload = PlanListPayload { items };
    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanList, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(PlanListArgs);
    let params = serde_json::json!({
      "type": "object",
    });

    let name = schema.get("title").expect("[PlanListArgs] title is required").clone();
    let description = schema.get("description").expect("[PlanListArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
