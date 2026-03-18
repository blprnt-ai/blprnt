use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(
  title = "skill_script",
  description = "Searches active skills in order, finds the first matching script under `scripts/`, and executes it with the provided arguments. `name` must be a relative path inside a skill's `scripts/` directory; absolute paths and traversal segments are rejected."
)]
pub struct SkillScriptArgs {
  pub name: String,
  #[serde(default)]
  #[schemars(default)]
  pub args: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct SkillScriptPayload {
  pub result: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub error:  Option<String>,
}

impl From<SkillScriptPayload> for ToolUseResponseData {
  fn from(payload: SkillScriptPayload) -> Self {
    Self::SkillScript(payload)
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct SkillItem {
  pub id:          String,
  pub name:        String,
  pub description: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tags:        Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub version:     Option<String>,
}
