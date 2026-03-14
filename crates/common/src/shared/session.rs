use surrealdb_types::SurrealValue;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
  pub input_tokens:  u32,
  pub output_tokens: u32,
}
