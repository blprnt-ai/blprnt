use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
)]
#[schemars(title = "list_skills", description = "Lists all available skills with their short descriptions.")]
pub struct ListSkillsArgs {}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ListSkillsPayload {
  pub items: Vec<SkillItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SkillItem {
  pub id:          String,
  pub name:        String,
  pub description: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tags:        Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub version:     Option<String>,
}

impl From<ListSkillsPayload> for ToolUseResponseData {
  fn from(payload: ListSkillsPayload) -> Self {
    Self::ListSkills(payload)
  }
}

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
)]
#[schemars(title = "apply_skill", description = "Applies a skill to the current context, injecting its instructions.")]
pub struct ApplySkillArgs {
  pub skill_name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ApplySkillPayload {
  pub success:    bool,
  pub skill_name: String,
}

impl From<ApplySkillPayload> for ToolUseResponseData {
  fn from(payload: ApplySkillPayload) -> Self {
    Self::ApplySkill(payload)
  }
}

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
)]
#[schemars(
  title = "get_reference",
  description = "Searches active skills in order and returns the first matching reference document. `reference_path` must be a relative path inside a skill directory; absolute paths and traversal segments are rejected."
)]
pub struct GetReferenceArgs {
  pub reference_path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct GetReferencePayload {
  pub content: String,
}

impl From<GetReferencePayload> for ToolUseResponseData {
  fn from(payload: GetReferencePayload) -> Self {
    Self::GetReference(payload)
  }
}

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
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
