use schemars::JsonSchema;
use surrealdb_types::SurrealValue;

use super::contracts::MemoryWriteStatus;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct MemoryWriteRequest {
  pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema, SurrealValue)]
pub struct MemoryWriteResult {
  pub status: MemoryWriteStatus,
  pub path:   String,
  pub date:   String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct MemorySearchRequest {
  pub query: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema, SurrealValue)]
pub struct MemorySearchResultItem {
  pub title:   String,
  pub content: String,
  pub score:   f64,
}

impl Eq for MemorySearchResultItem {}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema, SurrealValue)]
pub struct MemorySearchResult {
  pub memories: Vec<MemorySearchResultItem>,
}

impl Eq for MemorySearchResult {}
