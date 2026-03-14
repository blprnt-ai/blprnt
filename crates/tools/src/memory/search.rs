use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::memory::QmdMemorySearchService;
use common::tools::MemorySearchArgs;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct SearchMemoryTool {
  pub args: MemorySearchArgs,
}

#[async_trait]
impl Tool for SearchMemoryTool {
  fn tool_id(&self) -> ToolId {
    ToolId::MemorySearch
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let request: common::memory::MemorySearchRequest = self.args.clone().into();
    let service = QmdMemorySearchService::new(context.project_id.key().to_string());
    let payload = service.search(&request, Some(0.35)).await?;
    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled_for_runtime(
      ToolId::MemorySearch,
      config.agent_kind,
      config.is_subagent,
      config.memory_tools_enabled,
    ) {
      return vec![];
    }

    let schema = schemars::schema_for!(MemorySearchArgs);
    let json = serde_json::to_value(&schema).expect("[MemorySearchArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[MemorySearchArgs] properties is required"),
      "required": json.get("required").expect("[MemorySearchArgs] required is required")
    });

    let name = schema.get("title").expect("[MemorySearchArgs] title is required").clone();
    let description = schema.get("description").expect("[MemorySearchArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

#[cfg(test)]
mod tests {
  use common::agent::AgentKind;
  use common::tools::WorkingDirectories;
  use common::tools::config::ToolsSchemaConfig;

  use super::*;

  #[test]
  fn schema_does_not_expose_project_id_filter() {
    let schema = SearchMemoryTool::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Planner,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });

    let tool =
      schema.into_iter().find(|tool| tool.name == serde_json::json!("memory_search")).expect("memory_search schema");
    let properties = tool.params.get("properties").and_then(serde_json::Value::as_object).expect("properties object");

    assert!(!properties.contains_key("project_id"));
  }

  #[test]
  fn schema_hides_memory_search_when_runtime_gate_is_off() {
    let schema = SearchMemoryTool::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Planner,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: false,
      enabled_models:       vec![],
    });

    assert!(schema.is_empty());
  }
}
