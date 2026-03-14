use std::sync::Arc;

use serde_json::Value;

use crate::tools::registry::ToolSchemaRegistry;

const ALLOWED_JSON_SCHEMA_TYPES: [&str; 7] = ["object", "array", "string", "number", "integer", "boolean", "null"];

fn is_allowed_type(type_name: &str) -> bool {
  ALLOWED_JSON_SCHEMA_TYPES.contains(&type_name)
}

#[derive(Clone, Debug, Default)]
pub struct OpenAiTools;

impl OpenAiTools {
  pub fn to_tools_json(reg: Option<Arc<ToolSchemaRegistry>>) -> Value {
    let Some(reg) = reg else {
      return Value::Array(Vec::new());
    };

    let mut arr = Vec::new();
    if let Some(obj) = reg.schemas().as_array() {
      for tool in obj.iter() {
        let name = tool.get("name");

        let is_mcp = name.map(|n| n.as_str().unwrap_or("")).map(|n| n.starts_with("mcp__")).unwrap_or(false);

        let schema = tool.get("params");
        let description = tool.get("description");
        let properties = schema.and_then(|s| s.get("properties")).cloned().unwrap_or(serde_json::json!({}));
        let properties_keys = properties.as_object().map(|o| o.keys().collect::<Vec<&String>>()).unwrap_or(vec![]);
        let properties = if properties_keys.is_empty() { serde_json::json!({}) } else { properties.clone() };
        let required =
          if properties_keys.is_empty() { serde_json::json!([]) } else { serde_json::json!(properties_keys) };

        let mut parameters = serde_json::json!({
          "type": "object",
          "properties": properties,
          "required": required,
          "additionalProperties": false,
        });

        Self::sanitize_json_schema(&mut parameters);

        let json = serde_json::json!({
          "type": "function",
          "name": name,
          "description": description,
          "parameters": parameters,
          "strict": !is_mcp,
        });

        arr.push(json);
      }
    }

    arr.sort_by(|a, b| {
      let name_a = a.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
      let name_b = b.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
      name_a.cmp(name_b)
    });

    // arr.push(serde_json::json!({ "type": "web_search" }));

    if arr.len() > 100 {
      tracing::warn!("Tool catalog exceeds OpenAi limits, truncating from {} to 100 tools", arr.len());
      arr.truncate(100);
    }

    Value::Array(arr)
  }

  pub fn to_plain_tools_json(reg: &ToolSchemaRegistry) -> Value {
    let mut arr = Vec::new();
    if let Some(obj) = reg.schemas().as_array() {
      for tool in obj.iter() {
        let name = tool.get("name");
        let schema = tool.get("params");
        let description = tool.get("description");
        let properties = schema.and_then(|s| s.get("properties"));
        let required = schema.and_then(|s| s.get("required"));

        arr.push(serde_json::json!({
          "name": name,
          "description": description,
          "parameters": {
            "type": "object",
            "properties": properties,
            "required": required,
          },
        }));
      }
    }

    arr.sort_by(|a, b| {
      let name_a = a.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
      let name_b = b.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()).unwrap_or("");
      name_a.cmp(name_b)
    });

    if arr.len() > 100 {
      tracing::warn!("Tool catalog exceeds OpenAi limits, truncating from {} to 100 tools", arr.len());
      arr.truncate(100);
    }

    Value::Array(arr)
  }

  fn sanitize_json_schema(value: &mut Value) {
    match value {
      Value::Bool(_) => {
        *value = serde_json::json!({ "type": "string" });
      }
      Value::Array(arr) => {
        for v in arr.iter_mut() {
          Self::sanitize_json_schema(v);
        }
      }
      Value::Object(map) => {
        // `schemars` can emit `additionalProperties: {}` to mean "any value". OpenAI rejects
        // this in strict schemas, so strip the empty object entirely.
        let should_remove_additional_properties =
          map.get("additionalProperties").is_some_and(|v| matches!(v, Value::Object(obj) if obj.is_empty()));
        if should_remove_additional_properties {
          map.insert("additionalProperties".to_owned(), Value::Bool(false));
        }

        let has_array_type = map.get("type").is_some_and(|type_value| match type_value {
          Value::String(type_name) => type_name == "array",
          Value::Array(type_names) => type_names.iter().any(|v| v.as_str() == Some("array")),
          _ => false,
        });

        if map.contains_key("properties") {
          map.entry("additionalProperties").or_insert(Value::Bool(false));
        }

        if let Some(props) = map.get_mut("properties")
          && let Some(props_map) = props.as_object_mut()
        {
          for (_k, v) in props_map.iter_mut() {
            Self::sanitize_json_schema(v);
          }

          let required_keys = props_map.keys().cloned().collect::<Vec<String>>();
          map.insert("required".to_owned(), serde_json::json!(required_keys));
        }

        if let Some(items) = map.get_mut("items") {
          if items.is_null() {
            *items = serde_json::json!({ "type": "string" });
          } else {
            Self::sanitize_json_schema(items);
          }
        } else if has_array_type && !map.contains_key("prefixItems") {
          // OpenAI strict schemas require arrays to specify an `items` schema.
          map.insert("items".to_owned(), serde_json::json!({ "type": "string" }));
        }

        for combiner in ["oneOf", "anyOf", "allOf", "prefixItems"] {
          if let Some(v) = map.get_mut(combiner) {
            Self::sanitize_json_schema(v);
          }
        }

        // If `additionalProperties` is itself a schema object (not a boolean), sanitize it too.
        if let Some(ap) = map.get_mut("additionalProperties")
          && matches!(ap, Value::Object(_) | Value::Array(_))
        {
          Self::sanitize_json_schema(ap);
        }

        if let Some(type_value) = map.get("type") {
          let normalized_type_value = match type_value {
            Value::String(type_name) => Value::String(type_name.clone()),

            Value::Array(type_names) => Value::Array(
              type_names
                .iter()
                .filter_map(Value::as_str)
                .filter(|type_name| is_allowed_type(type_name))
                .map(|type_name| Value::String(type_name.to_owned()))
                .collect(),
            ),

            _ => Value::String("string".to_owned()),
          };

          map.insert("type".to_owned(), normalized_type_value);
        } else {
          map.insert("type".to_owned(), Value::String("string".to_owned()));
        }

        // OpenAI strict schemas require `enum` to include `null` when the schema type allows `null`.
        let type_allows_null = map.get("type").is_some_and(|type_value| match type_value {
          Value::String(type_name) => type_name == "null",
          Value::Array(type_names) => type_names.iter().any(|v| v.as_str() == Some("null")),
          _ => false,
        });
        if type_allows_null
          && let Some(enum_value) = map.get_mut("enum")
          && let Some(enum_values) = enum_value.as_array_mut()
          && !enum_values.iter().any(Value::is_null)
        {
          enum_values.push(Value::Null);
        }
      }
      _ => {}
    }
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
  fn test_build_tools() {
    let tools = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Planner,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });
    let tools = serde_json::to_value(tools).unwrap_or_default();
    let tools = OpenAiTools::to_tools_json(Some(Arc::new(ToolSchemaRegistry::new(tools))));
    println!("{}", serde_json::to_string_pretty(&tools).unwrap_or_default());
  }

  #[test]
  fn test_to_tools_json() {
    let tools = serde_json::to_value(TestTool::schema()).unwrap_or_default();
    let reg = Arc::new(ToolSchemaRegistry::new(tools));
    let json = OpenAiTools::to_tools_json(Some(reg));

    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());

    // Web search is included
    assert_eq!(json.as_array().map(|arr| arr.len()), Some(2));
  }

  #[test]
  fn test_to_tools_json_full() -> std::io::Result<()> {
    let tools = Tools::schema(&ToolsSchemaConfig {
      agent_kind:           AgentKind::Crew,
      working_directories:  WorkingDirectories::new(vec![]),
      is_subagent:          false,
      memory_tools_enabled: true,
      enabled_models:       vec![],
    });
    let reg = Arc::new(ToolSchemaRegistry::new(json!(tools)));
    let json = OpenAiTools::to_tools_json(Some(reg));

    let json_string = serde_json::to_string_pretty(&json).unwrap_or_default();

    std::fs::write("tools.json", json_string)?;
    Ok(())
  }

  #[test]
  fn test_sanitize_json_schema() {
    let mut value = serde_json::json!({
      "type": "object",
      "properties": {
        "pattern": {
          "type": "string",
          "description": "URL glob (picomatch)."
        },
        "modifications": {
          "type": "object",
          "properties": {
            "headers": {
              "type": "object",
              "additionalProperties": {
                "type": "string"
              }
            },
            "body": {
              "anyOf": [
                {
                  "type": "string"
                },
                {
                  "type": "object",
                  "additionalProperties": {}
                },
                {
                  "type": "array"
                }
              ],
              "type": "string"
            },
            "method": {
              "type": "string",
              "description": "Override method (e.g. GET, POST)."
            }
          },
          "additionalProperties": {
            "type": "string"
          },
          "description": "Changes to apply to the request.",
          "required": [
            "headers",
            "body",
            "method"
          ]
        },
        "delayMs": {
          "type": "integer",
          "minimum": 0
        },
        "times": {
          "type": "integer",
          "description": "Max applications; -1 = infinite."
        }
      },
      "required": [
        "pattern",
        "modifications",
        "delayMs",
        "times"
      ],
      "additionalProperties": {
        "type": "string"
      }
    });
    OpenAiTools::sanitize_json_schema(&mut value);

    fn assert_no_empty_additional_properties(v: &Value) {
      match v {
        Value::Array(arr) => arr.iter().for_each(assert_no_empty_additional_properties),
        Value::Object(map) => {
          if let Some(ap) = map.get("additionalProperties")
            && matches!(ap, Value::Object(obj) if obj.is_empty())
          {
            panic!("Found empty additionalProperties object: {}", serde_json::to_string_pretty(v).unwrap_or_default());
          }
          map.values().for_each(assert_no_empty_additional_properties);
        }
        _ => {}
      }
    }

    fn assert_arrays_have_items(v: &Value) {
      match v {
        Value::Array(arr) => arr.iter().for_each(assert_arrays_have_items),
        Value::Object(map) => {
          let has_array_type = map.get("type").is_some_and(|type_value| match type_value {
            Value::String(type_name) => type_name == "array",
            Value::Array(type_names) => type_names.iter().any(|v| v.as_str() == Some("array")),
            _ => false,
          });

          if has_array_type && !map.contains_key("items") && !map.contains_key("prefixItems") {
            panic!("Found array schema without items: {}", serde_json::to_string_pretty(v).unwrap_or_default());
          }

          map.values().for_each(assert_arrays_have_items);
        }
        _ => {}
      }
    }

    assert_no_empty_additional_properties(&value);
    assert_arrays_have_items(&value);
  }

  #[test]
  fn test_sanitize_json_schema_injects_null_into_enum_when_type_allows_null() {
    let mut value = serde_json::json!({
      "type": ["string", "null"],
      "enum": ["a", "b"]
    });
    OpenAiTools::sanitize_json_schema(&mut value);

    let enum_values = value.get("enum").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    assert!(enum_values.iter().any(Value::is_null), "Expected enum to include null, got: {enum_values:?}");
  }
}
