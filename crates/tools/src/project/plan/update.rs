use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::errors::ToolError;
use common::plan_utils::PlanFrontmatter;
use common::plan_utils::apply_plan_content_patch;
use common::plan_utils::parse_frontmatter;
use common::plan_utils::render_plan_content;
use common::plan_utils::resolve_plan_directory;
use common::tools::PlanDocumentStatus;
use common::tools::PlanStatus;
use common::tools::PlanUpdateArgs;
use common::tools::PlanUpdatePayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct PlanUpdateTool {
  pub args: PlanUpdateArgs,
}

#[async_trait]
impl Tool for PlanUpdateTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PlanUpdate
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    if self.args.content.is_some() && self.args.content_patch.is_some() {
      return Err(
        ToolError::General("plan_update content and content_patch are mutually exclusive".to_string()).into(),
      );
    }

    let plan_directory = resolve_plan_directory(context.project_id)?;
    let base_path = PathBuf::from(&plan_directory.path);
    let path = base_path.join(&self.args.id);

    let content = std::fs::read_to_string(&path)
      .map_err(|e| ToolError::FileReadFailed { path: path.display().to_string(), error: e.to_string() })?;
    let (frontmatter, body) = parse_frontmatter(&content)?;
    let mut meta = frontmatter.into_meta();

    if let Some(name) = &self.args.name {
      meta.name = name.clone();
    }
    if let Some(description) = &self.args.description {
      meta.description = description.clone();
    }
    if let Some(todos) = &self.args.todos {
      meta.todos = todos.clone();
    }
    if let Some(status) = &self.args.status {
      meta.status = status.clone();
    }

    if meta.todos.iter().all(|todo| todo.status == PlanStatus::Complete) && meta.status != PlanDocumentStatus::Archived
    {
      meta.status = common::tools::PlanDocumentStatus::Completed;
    }

    let updated_body = if let Some(content) = &self.args.content {
      content.clone()
    } else if let Some(content_patch) = &self.args.content_patch {
      apply_plan_content_patch(&body, content_patch)?
    } else {
      body
    };
    meta.updated_at = Utc::now().to_rfc3339();

    let frontmatter = PlanFrontmatter {
      name:              meta.name.clone(),
      description:       meta.description.clone(),
      todos:             meta.todos.clone(),
      created_at:        meta.created_at.clone(),
      updated_at:        meta.updated_at.clone(),
      status:            meta.status.clone(),
      parent_session_id: meta.parent_session_id.clone(),
    };

    let updated_content = render_plan_content(&frontmatter, &updated_body)?;
    std::fs::write(&path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: path.display().to_string(), error: e.to_string() })?;

    let payload = PlanUpdatePayload {
      id:          self.args.id.clone(),
      name:        meta.name,
      description: meta.description,
      created_at:  meta.created_at,
      updated_at:  meta.updated_at,
    };

    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PlanUpdate, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(PlanUpdateArgs);
    let json = serde_json::to_value(&schema).expect("[PlanUpdateArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[PlanUpdateArgs] properties is required"),
      "required": json.get("required").expect("[PlanUpdateArgs] required is required")
    });

    let name = schema.get("title").expect("[PlanUpdateArgs] title is required").clone();
    let description = schema.get("description").expect("[PlanUpdateArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use common::agent::AgentKind;
  use common::sandbox_flags::SandboxFlags;
  use common::tools::PlanContentPatch;
  use common::tools::PlanContentPatchHunk;
  use persistence::prelude::SurrealId;

  use super::*;
  use crate::Tool;
  use crate::tool_use::ToolUseContext;

  fn sample_patch() -> PlanContentPatch {
    PlanContentPatch {
      hunks: vec![PlanContentPatchHunk {
        before: vec!["alpha".to_string()],
        delete: vec!["beta".to_string()],
        insert: vec!["delta".to_string()],
        after:  vec!["gamma".to_string()],
      }],
    }
  }

  #[tokio::test]
  async fn plan_update_tool_rejects_content_and_content_patch_together() {
    let tool = PlanUpdateTool {
      args: PlanUpdateArgs {
        id:            "plan-1".to_string(),
        name:          None,
        description:   None,
        content:       Some("replacement body".to_string()),
        content_patch: Some(sample_patch()),
        todos:         None,
        status:        None,
      },
    };
    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from("/tmp")],
      vec![],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    );

    let error = tool.run(context).await.unwrap_err().to_string();

    assert!(error.contains("plan_update content and content_patch are mutually exclusive"));
  }
}
