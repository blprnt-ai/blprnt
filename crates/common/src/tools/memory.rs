pub use crate::memory::MemorySearchRequest;
pub use crate::memory::MemorySearchResult;
pub use crate::memory::MemorySearchResultItem;
pub use crate::memory::MemoryWriteRequest;
pub use crate::memory::MemoryWriteResult;
pub use crate::memory::MemoryWriteSource;
pub use crate::memory::MemoryWriteStatus;
use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "memory_write", description = "Appends plain markdown content to today's daily memory log.")]
pub struct MemoryWriteArgs {
  pub content: String,
}

impl From<MemoryWriteArgs> for MemoryWriteRequest {
  fn from(value: MemoryWriteArgs) -> Self {
    Self { content: value.content }
  }
}

impl From<MemoryWriteResult> for ToolUseResponseData {
  fn from(payload: MemoryWriteResult) -> Self {
    Self::MemoryWrite(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
  title = "memory_search",
  description = "Searches app-global markdown memories through the memory search contract."
)]
pub struct MemorySearchArgs {
  pub query: String,
  pub limit: Option<usize>,
}

impl From<MemorySearchArgs> for MemorySearchRequest {
  fn from(value: MemorySearchArgs) -> Self {
    Self { query: value.query, limit: value.limit }
  }
}

impl From<MemorySearchResult> for ToolUseResponseData {
  fn from(payload: MemorySearchResult) -> Self {
    Self::MemorySearch(payload)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn memory_search_args_reject_project_id() {
    let error = serde_json::from_value::<MemorySearchArgs>(serde_json::json!({
      "query": "architecture",
      "limit": 5,
      "project_id": "projects:u'alpha'"
    }))
    .err();

    assert!(error.is_some());
  }

  #[test]
  fn memory_write_args_accept_only_content() {
    let parsed: MemoryWriteArgs = serde_json::from_value(serde_json::json!({
      "content": "Tagged memory"
    }))
    .unwrap_or_else(|error| panic!("parse failed: {error}"));

    assert_eq!(parsed.content, "Tagged memory");
  }

  #[test]
  fn memory_write_args_reject_legacy_fields() {
    let error = serde_json::from_value::<MemoryWriteArgs>(serde_json::json!({
      "content": "Tagged memory",
      "memory_type": "semantic"
    }))
    .err();

    assert!(error.is_some());
  }
}
