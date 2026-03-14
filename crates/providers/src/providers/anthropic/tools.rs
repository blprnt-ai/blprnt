use std::sync::Arc;

use crate::tools::registry::ToolSchemaRegistry;

#[derive(Clone, Debug, Default)]
pub struct AnthropicTools;

impl AnthropicTools {
  pub fn to_tools_json(reg: Option<Arc<ToolSchemaRegistry>>) -> serde_json::Value {
    let Some(reg) = reg else {
      return serde_json::Value::Array(Vec::new());
    };

    let mut result = Vec::new();

    if let Some(arr) = reg.schemas().as_array() {
      let mut arr = arr.clone();
      arr.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
        name_a.cmp(name_b)
      });

      for tool in arr.iter() {
        let name = tool.get("name");
        let description = tool.get("description");
        let schema = tool.get("params");
        let properties = schema.and_then(|s| s.get("properties"));
        let required = schema.and_then(|s| s.get("required"));

        let value = serde_json::json!({
          "name": name,
          "description": description,
          "input_schema": {
            "type": "object",
            "properties": properties,
            "required": required,
          }
        });

        result.push(value);
      }
    }

    serde_json::Value::Array(result)
  }
}

#[cfg(test)]
mod tests {
  use common::agent::AgentKind;
  use common::tools::WorkingDirectories;
  use common::tools::config::ToolsSchemaConfig;
  use common::tools::test::TestTool;
  use serde_json::json;
  use tools::Tool;
  use tools::Tools;

  use super::*;

  #[test]
  fn test_to_tools_json() {
    let tools = serde_json::to_value(TestTool::schema()).unwrap_or_default();
    let reg = Arc::new(ToolSchemaRegistry::new(tools));
    let json = AnthropicTools::to_tools_json(Some(reg));

    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());

    assert_eq!(json.as_array().map(|arr| arr.len()), Some(1));
  }

  #[test]
  fn test_to_tools_json_only_anthropic() -> std::io::Result<()> {
    let tools = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Crew,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });

    let reg = Arc::new(ToolSchemaRegistry::new(json!(tools)));
    let json = AnthropicTools::to_tools_json(Some(reg));
    let json_string = serde_json::to_string_pretty(&json).unwrap_or_default();

    std::fs::write("tools.json", json_string)?;
    Ok(())
  }

  #[test]
  fn test_to_tools_json_all() -> std::io::Result<()> {
    let tools = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Crew,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });

    let reg = Arc::new(ToolSchemaRegistry::new(json!(tools)));
    let json = AnthropicTools::to_tools_json(Some(reg));
    let json_string = serde_json::to_string_pretty(&json).unwrap_or_default();

    std::fs::write("tools.json", json_string)?;
    Ok(())
  }
}
