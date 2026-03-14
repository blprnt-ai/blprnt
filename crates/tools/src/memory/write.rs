use anyhow::Result;
use async_trait::async_trait;
use common::agent::ToolAllowList;
use common::agent::ToolId;
use common::memory::ManagedMemoryStore;
use common::memory::local_today;
use common::paths::BlprntPath;
use common::tools::MemoryWriteArgs;
use common::tools::ToolSpec;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseData;
use common::tools::config::ToolsSchemaConfig;

use crate::Tool;
use crate::tool_use::ToolUseContext;

#[derive(Clone, Debug)]
pub struct WriteMemoryTool {
  pub args: MemoryWriteArgs,
}

#[async_trait]
impl Tool for WriteMemoryTool {
  fn tool_id(&self) -> ToolId {
    ToolId::MemoryWrite
  }

  async fn run(&self, context: ToolUseContext) -> Result<ToolUseResponse> {
    let request = common::memory::MemoryWriteRequest { content: self.args.content.clone() };
    let memory_root = BlprntPath::memories_root().join(context.project_id.key().to_string());
    let store = ManagedMemoryStore::new(memory_root);
    let _ = context;
    let payload = store.append_entry_for_date(local_today(), &request.content)?;

    Ok(ToolUseResponseData::success(payload.into()))
  }

  fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
    if !ToolAllowList::is_tool_allowed_and_enabled_for_runtime(
      ToolId::MemoryWrite,
      config.agent_kind,
      config.is_subagent,
      config.memory_tools_enabled,
    ) {
      return vec![];
    }

    let schema = schemars::schema_for!(MemoryWriteArgs);
    let json = serde_json::to_value(&schema).expect("[MemoryWriteArgs] schema is required");
    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[MemoryWriteArgs] properties is required"),
      "required": json.get("required").expect("[MemoryWriteArgs] required is required")
    });
    let name = schema.get("title").expect("[MemoryWriteArgs] title is required").clone();
    let description = schema.get("description").expect("[MemoryWriteArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}

#[cfg(test)]
mod tests {

  #[test]
  fn memory_write_request_only_contains_content() {
    let request = common::memory::MemoryWriteRequest { content: "Tagged memory".to_string() };

    assert_eq!(request.content, "Tagged memory");
  }
}
