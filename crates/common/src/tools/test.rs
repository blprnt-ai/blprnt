use crate::tools::ToolSpec;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(
  title = "test_tool",
  description = "Internal test tool used for validating tool schema plumbing.\n\nWhen to use: Only for testing or development of tool metadata.\n\nBehavior:\n- Returns whatever fields are provided\n- Not intended for production workflows"
)]
pub struct TestTool {
  #[schemars(description = "The id of the tool")]
  pub id:          usize,
  #[schemars(description = "The name of the tool")]
  pub name:        String,
  #[schemars(description = "The description of the tool")]
  pub description: String,
  #[schemars(description = "The optional parameters of the tool")]
  pub params:      Option<String>,
}

impl TestTool {
  pub fn schema() -> Vec<ToolSpec> {
    let schema = schemars::schema_for!(TestTool);
    let json = serde_json::to_value(&schema).unwrap_or_default();

    let properties =
      json.get("properties").cloned().unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let required = json.get("required").cloned().unwrap_or_else(|| serde_json::Value::Array(vec![]));

    let params = serde_json::json!({
      "type": "object",
      "properties": properties,
      "required": required,
    });

    let name = schema.get("title").cloned().unwrap_or_else(|| serde_json::Value::String("test_tool".to_string()));
    let description =
      schema.get("description").cloned().unwrap_or_else(|| serde_json::Value::String("A test tool".to_string()));

    vec![ToolSpec { name, description, params }]
  }
}
