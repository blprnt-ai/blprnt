use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::tools::GetPrimerArgs;
use common::tools::GetPrimerPayload;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;
use persistence::prelude::ProjectRepositoryV2;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct GetPrimerTool;

#[async_trait]
impl Tool for GetPrimerTool {
  fn tool_id(&self) -> ToolId {
    ToolId::PrimerGet
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let project_model = ProjectRepositoryV2::get(context.project_id).await?;
    let primer = project_model.agent_primer().clone().unwrap_or_default();

    Ok(ToolUseResponseData::success(GetPrimerPayload { content: primer }.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::PrimerGet, config.agent_kind, config.is_subagent) {
      return vec![];
    }

    let schema = schemars::schema_for!(GetPrimerArgs);

    let params = serde_json::json!({
      "type": "object",
    });

    let name = schema.get("title").expect("[GetPrimerArgs] title is required").clone();
    let description = schema.get("description").expect("[GetPrimerArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
