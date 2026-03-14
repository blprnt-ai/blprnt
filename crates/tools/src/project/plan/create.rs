use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::errors::ToolError;
use common::plan_utils::PlanFrontmatter;
use common::plan_utils::build_plan_id;
use common::plan_utils::build_write_context;
use common::plan_utils::ensure_plan_dir;
use common::plan_utils::get_plan_content_by_parent_session_id;
use common::plan_utils::render_plan_content;
use common::plan_utils::resolve_plan_directory;
use common::tools::PlanCreateArgs;
use common::tools::PlanCreatePayload;
use common::tools::PlanDocumentStatus;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct PlanCreateTool {
  pub args: PlanCreateArgs,
}

#[async_trait]
impl Tool for PlanCreateTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PlanCreate
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let plan_directory = resolve_plan_directory(context.project_id.clone())?;
    let base_path = PathBuf::from(&plan_directory.path);
    ensure_plan_dir(&base_path)?;

    let effective_session_id = context.parent_id.clone().unwrap_or(context.session_id.clone());

    if let Some(existing_plan) =
      get_plan_content_by_parent_session_id(context.project_id.clone(), &effective_session_id.to_string())?
    {
      return Err(
        ToolError::General(format!(
          "Session already has a plan attached ('{}'). Detach it first before creating a new plan.",
          existing_plan.id
        ))
        .into(),
      );
    }

    let now = Utc::now().to_rfc3339();
    let plan_id = build_plan_id(&self.args.name);
    let write_context = build_write_context(plan_id, &base_path, now.clone(), now.clone());

    let frontmatter = PlanFrontmatter {
      name:              self.args.name.clone(),
      description:       self.args.description.clone(),
      todos:             self.args.todos.clone().unwrap_or_default(),
      created_at:        write_context.created_at.clone(),
      updated_at:        write_context.updated_at.clone(),
      status:            PlanDocumentStatus::Pending,
      parent_session_id: Some(effective_session_id.to_string()),
    };

    let content = render_plan_content(&frontmatter, &self.args.content)?;
    let path = PathBuf::from(&write_context.plan_path);

    if path.exists() {
      return Err(
        ToolError::FileWriteFailed { path: write_context.plan_path, error: "plan already exists".to_string() }.into(),
      );
    }

    std::fs::write(&path, content)
      .map_err(|e| ToolError::FileWriteFailed { path: write_context.plan_path, error: e.to_string() })?;

    let payload = PlanCreatePayload {
      id:          write_context.plan_id.clone(),
      name:        frontmatter.name,
      description: frontmatter.description,
      created_at:  frontmatter.created_at,
      updated_at:  frontmatter.updated_at,
    };

    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanCreate, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(PlanCreateArgs);
    let json = serde_json::to_value(&schema).expect("[PlanCreateArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[PlanCreateArgs] properties is required"),
      "required": json.get("required").expect("[PlanCreateArgs] required is required")
    });

    let name = schema.get("title").expect("[PlanCreateArgs] title is required").clone();
    let description = schema.get("description").expect("[PlanCreateArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
