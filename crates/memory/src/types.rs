#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryTreeNode {
  Directory {
    name:     String,
    path:     String,
    #[schema(no_recursion)]
    children: Vec<MemoryTreeNode>,
  },
  File {
    name: String,
    path: String,
  },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemoryListResult {
  pub root_path: String,
  pub nodes:     Vec<MemoryTreeNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemoryReadResult {
  pub path:    String,
  pub content: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemorySearchResultItem {
  pub title:   String,
  pub content: String,
  pub score:   f64,
}

impl Eq for MemorySearchResultItem {}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MemorySearchResult {
  pub memories: Vec<MemorySearchResultItem>,
}

impl Eq for MemorySearchResult {}
