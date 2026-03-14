use common::errors::ProviderError;
use serde_json::Value;

pub trait ToolSchemaAdapter {
  fn adapt(&self, input: &str) -> anyhow::Result<String>;

  fn extract_schemas(&self, v: Value) -> anyhow::Result<Vec<Value>> {
    match v {
      Value::Object(mut obj) => {
        if let Some(schemas_v) = obj.remove("schemas") {
          match schemas_v {
            Value::Object(map) => Ok(map.into_values().collect()),
            Value::Array(arr) => Ok(arr),
            other => {
              Err(ProviderError::InvalidSchema(format!("'schemas' must be an object or array, got {}", other)).into())
            }
          }
        } else {
          Ok(vec![Value::Object(obj)])
        }
      }

      Value::Array(items) => Ok(items),
      _ => Err(ProviderError::InvalidSchema("input must be a JSON object or array".into()).into()),
    }
  }
}
