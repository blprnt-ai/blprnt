use serde::Deserialize;
use serde::Serialize;

// =============================================================================
// SurrealDB record models (schema contract)
//
// These structs are intentionally minimal. A consuming application can map
// them to SurrealDB tables/records however it prefers.
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRow {
  pub name:               String,
  pub path:               String,
  pub glob_pattern:       String,
  pub ignore:             Option<Vec<String>>,
  pub include_by_default: bool,
  pub last_modified:      Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRow {
  pub collection:  String,
  pub path_prefix: String,
  pub context:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRow {
  pub hash:       String,
  pub doc:        String,
  pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRow {
  pub collection:  String,
  pub path:        String,
  pub title:       String,
  pub hash:        String,
  pub created_at:  String,
  pub modified_at: String,
  pub active:      bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRow {
  pub chunk_id:   String,
  pub docid:      String,
  pub index:      u32,
  pub body:       String,
  pub body_bytes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRow {
  pub chunk_id:  String,
  pub model:     String,
  pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVectorRow {
  pub hash:        String,
  pub seq:         u32,
  pub pos:         u32,
  pub embedding:   Vec<f32>,
  pub model:       String,
  pub embedded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheRow {
  pub cache_key:   String,
  pub body_json:   String,
  pub result_json: String,
  pub created_at:  String,
}
