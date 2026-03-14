use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::UpdatePrimerArgs;
use common::tools::UpdatePrimerPayload;
use common::tools::config::ToolsSchemaConfig;
use persistence::prelude::ProjectPatchV2;
use persistence::prelude::ProjectRepositoryV2;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct UpdatePrimerTool {
  pub args: UpdatePrimerArgs,
}

#[async_trait]
impl Tool for UpdatePrimerTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PrimerUpdate
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let project_model = ProjectRepositoryV2::get(context.project_id).await?;
    let project_patch = ProjectPatchV2 { agent_primer: Some(Some(self.args.content.clone())), ..Default::default() };
    let project_model = ProjectRepositoryV2::update(project_model.id.clone(), project_patch).await?;
    let content = project_model.agent_primer().clone().unwrap_or_default();

    Ok(ToolUseResponseData::success(UpdatePrimerPayload { content }.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PrimerGet, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(UpdatePrimerArgs);
    let json = serde_json::to_value(&schema).expect("[UpdatePrimerArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[UpdatePrimerArgs] properties is required"),
      "required": json.get("required").expect("[UpdatePrimerArgs] required is required"),
    });

    let name = schema.get("title").expect("[UpdatePrimerArgs] title is required").clone();
    let description = schema.get("description").expect("[UpdatePrimerArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
