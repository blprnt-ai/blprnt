use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(title = "rg", description = "Searches files using ripgrep (rg) and returns stdout.")]
pub struct RgSearchArgs {
  pub pattern:         String,
  #[schemars(default)]
  pub path:            Option<String>,
  #[schemars(default)]
  pub flags:           Vec<String>,
  #[schemars(default)]
  #[schemars(
    description = "Optional zero-based workspace index to use. If not provided, the first workspace will be used."
  )]
  pub workspace_index: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RgSearchPayload {
  pub stdout: String,
}

impl From<RgSearchPayload> for ToolUseResponseData {
  fn from(payload: RgSearchPayload) -> Self {
    Self::RgSearch(payload)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rg_args_schema_metadata() {
    let schema = schemars::schema_for!(RgSearchArgs);
    let json = serde_json::to_value(&schema).expect("schema must serialize");

    assert_eq!(json.get("title").and_then(|value| value.as_str()), Some("rg"));
    assert!(json.get("description").and_then(|value| value.as_str()).is_some());
    assert_eq!(json.get("type").and_then(|value| value.as_str()), Some("object"));
  }

  #[test]
  fn test_rg_args_schema() {
    let schema = schemars::schema_for!(RgSearchArgs);
    let json = serde_json::to_value(&schema).expect("schema must serialize");
    let properties = json.get("properties").and_then(|value| value.as_object()).expect("properties required");

    assert!(properties.contains_key("pattern"));
    assert!(properties.contains_key("path"));
    assert!(properties.contains_key("flags"));

    let required = json.get("required").and_then(|value| value.as_array()).expect("required required");
    assert!(required.iter().any(|value| value.as_str() == Some("pattern")));
  }
}
