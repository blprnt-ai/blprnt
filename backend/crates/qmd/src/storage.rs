use async_trait::async_trait;

use crate::CollectionConfig;
use crate::CollectionListItem;
use crate::ContextItem;
use crate::DocumentResult;
use crate::IndexHealthInfo;
use crate::IndexStatus;
use crate::LexSearchOptions;
use crate::MultiGetOptions;
use crate::MultiGetResponse;
use crate::NamedCollection;
use crate::Result;
use crate::SearchResult;

#[derive(Debug, Clone)]
pub struct EmbeddingCandidate {
  pub hash: String,
  pub body: String,
  pub path: String,
}

// =============================================================================
// Embeddings persistence API (matches blprnt-style Model/Record/Repository)
// =============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingModel {
  pub hash:        String,
  pub seq:         u32,
  pub pos:         u32,
  pub embedding:   Vec<f32>,
  pub model:       String,
  pub embedded_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingRecord {
  pub id:    String,
  #[serde(flatten)]
  pub model: EmbeddingModel,
}

#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
  async fn get_hashes_for_embedding(&self) -> Result<Vec<EmbeddingCandidate>>;
  async fn clear_all_embeddings(&self) -> Result<()>;
  async fn create_embedding(&self, model: EmbeddingModel) -> Result<EmbeddingRecord>;
}

// =============================================================================
// Storage contract (SurrealDB consumer implements this)
// =============================================================================

#[async_trait]
pub trait Storage: Send + Sync + EmbeddingRepository {
  // ── Collections ─────────────────────────────────────────────────────
  async fn list_collections(&self) -> Result<Vec<NamedCollection>>;
  async fn list_collections_info(&self) -> Result<Vec<CollectionListItem>>;
  async fn upsert_collection(&self, name: &str, collection: &crate::Collection) -> Result<()>;
  async fn delete_collection(&self, name: &str) -> Result<bool>;
  async fn rename_collection(&self, old_name: &str, new_name: &str) -> Result<bool>;

  // ── Contexts ────────────────────────────────────────────────────────
  async fn get_global_context(&self) -> Result<Option<String>>;
  async fn set_global_context(&self, context: Option<&str>) -> Result<()>;
  async fn list_contexts(&self) -> Result<Vec<ContextItem>>;
  async fn upsert_context(&self, collection: &str, path_prefix: &str, context: &str) -> Result<bool>;
  async fn remove_context(&self, collection: &str, path_prefix: &str) -> Result<bool>;

  // ── Config sync ─────────────────────────────────────────────────────
  async fn sync_config(&self, config: &CollectionConfig) -> Result<()>;

  // ── Documents + content-addressed storage ───────────────────────────
  async fn upsert_content(&self, hash: &str, doc: &str, created_at: &str) -> Result<()>;
  async fn get_content(&self, hash: &str) -> Result<Option<String>>;

  async fn upsert_document(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    hash: &str,
    created_at: &str,
    modified_at: &str,
    active: bool,
  ) -> Result<()>;

  async fn find_active_document(&self, collection_name: &str, path: &str) -> Result<Option<(String, String)>>; // (hash, title)

  async fn update_document_title(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    modified_at: &str,
  ) -> Result<()>;
  async fn update_document_hash(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    hash: &str,
    modified_at: &str,
  ) -> Result<()>;
  async fn deactivate_document(&self, collection_name: &str, path: &str) -> Result<()>;
  async fn list_active_document_paths(&self, collection_name: &str) -> Result<Vec<String>>;

  // ── Search (DB-backed) ──────────────────────────────────────────────
  async fn search_lex(&self, query: &str, options: &LexSearchOptions) -> Result<Vec<SearchResult>>;
  async fn search_vector(
    &self,
    embedding: &[f32],
    model: &str,
    options: &crate::VectorSearchOptions,
  ) -> Result<Vec<SearchResult>>;

  // ── Document retrieval (DB-backed) ──────────────────────────────────
  async fn get_document(&self, path_or_docid: &str, include_body: bool) -> Result<Option<DocumentResult>>;
  async fn multi_get(&self, pattern: &str, options: &MultiGetOptions) -> Result<MultiGetResponse>;
  async fn list_files(&self, target: Option<&str>) -> Result<Vec<String>>;

  // ── LLM cache (optional) ────────────────────────────────────────────
  async fn get_cached_result(&self, cache_key: &str) -> Result<Option<String>>;
  async fn set_cached_result(&self, cache_key: &str, result: &str) -> Result<()>;
  async fn clear_cached_results(&self) -> Result<usize>;

  // ── Aggregates ──────────────────────────────────────────────────────
  async fn get_status(&self) -> Result<IndexStatus>;
  async fn get_index_health(&self) -> Result<IndexHealthInfo>;

  // ── Maintenance (optional) ──────────────────────────────────────────
  async fn vacuum(&self) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::vacuum" })
  }

  async fn cleanup_orphaned_content(&self) -> Result<usize> {
    Err(crate::QmdError::NotImplemented { op: "Storage::cleanup_orphaned_content" })
  }

  async fn cleanup_orphaned_vectors(&self) -> Result<usize> {
    Err(crate::QmdError::NotImplemented { op: "Storage::cleanup_orphaned_vectors" })
  }

  async fn delete_inactive_docs(&self) -> Result<usize> {
    Err(crate::QmdError::NotImplemented { op: "Storage::delete_inactive_docs" })
  }
}

pub struct StubStorage;

#[async_trait]
impl EmbeddingRepository for StubStorage {
  async fn get_hashes_for_embedding(&self) -> Result<Vec<EmbeddingCandidate>> {
    Err(crate::QmdError::NotImplemented { op: "EmbeddingRepository::get_hashes_for_embedding" })
  }

  async fn clear_all_embeddings(&self) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "EmbeddingRepository::clear_all_embeddings" })
  }

  async fn create_embedding(&self, _model: EmbeddingModel) -> Result<EmbeddingRecord> {
    Err(crate::QmdError::NotImplemented { op: "EmbeddingRepository::create_embedding" })
  }
}

#[async_trait]
impl Storage for StubStorage {
  async fn list_collections(&self) -> Result<Vec<NamedCollection>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::list_collections" })
  }

  async fn list_collections_info(&self) -> Result<Vec<CollectionListItem>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::list_collections_info" })
  }

  async fn upsert_collection(&self, _name: &str, _collection: &crate::Collection) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::upsert_collection" })
  }

  async fn delete_collection(&self, _name: &str) -> Result<bool> {
    Err(crate::QmdError::NotImplemented { op: "Storage::delete_collection" })
  }

  async fn rename_collection(&self, _old_name: &str, _new_name: &str) -> Result<bool> {
    Err(crate::QmdError::NotImplemented { op: "Storage::rename_collection" })
  }

  async fn get_global_context(&self) -> Result<Option<String>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_global_context" })
  }

  async fn set_global_context(&self, _context: Option<&str>) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::set_global_context" })
  }

  async fn list_contexts(&self) -> Result<Vec<ContextItem>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::list_contexts" })
  }

  async fn upsert_context(&self, _collection: &str, _path_prefix: &str, _context: &str) -> Result<bool> {
    Err(crate::QmdError::NotImplemented { op: "Storage::upsert_context" })
  }

  async fn remove_context(&self, _collection: &str, _path_prefix: &str) -> Result<bool> {
    Err(crate::QmdError::NotImplemented { op: "Storage::remove_context" })
  }

  async fn sync_config(&self, _config: &CollectionConfig) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::sync_config" })
  }

  async fn upsert_content(&self, _hash: &str, _doc: &str, _created_at: &str) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::upsert_content" })
  }

  async fn get_content(&self, _hash: &str) -> Result<Option<String>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_content" })
  }

  async fn upsert_document(
    &self,
    _collection_name: &str,
    _path: &str,
    _title: &str,
    _hash: &str,
    _created_at: &str,
    _modified_at: &str,
    _active: bool,
  ) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::upsert_document" })
  }

  async fn find_active_document(&self, _collection_name: &str, _path: &str) -> Result<Option<(String, String)>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::find_active_document" })
  }

  async fn update_document_title(
    &self,
    _collection_name: &str,
    _path: &str,
    _title: &str,
    _modified_at: &str,
  ) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::update_document_title" })
  }

  async fn update_document_hash(
    &self,
    _collection_name: &str,
    _path: &str,
    _title: &str,
    _hash: &str,
    _modified_at: &str,
  ) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::update_document_hash" })
  }

  async fn deactivate_document(&self, _collection_name: &str, _path: &str) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::deactivate_document" })
  }

  async fn list_active_document_paths(&self, _collection_name: &str) -> Result<Vec<String>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::list_active_document_paths" })
  }

  async fn search_lex(&self, _query: &str, _options: &LexSearchOptions) -> Result<Vec<SearchResult>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::search_lex" })
  }

  async fn search_vector(
    &self,
    _embedding: &[f32],
    _model: &str,
    _options: &crate::VectorSearchOptions,
  ) -> Result<Vec<SearchResult>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::search_vector" })
  }

  async fn get_document(&self, _path_or_docid: &str, _include_body: bool) -> Result<Option<DocumentResult>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_document" })
  }

  async fn multi_get(&self, _pattern: &str, _options: &MultiGetOptions) -> Result<MultiGetResponse> {
    Err(crate::QmdError::NotImplemented { op: "Storage::multi_get" })
  }

  async fn list_files(&self, _target: Option<&str>) -> Result<Vec<String>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::list_files" })
  }

  async fn get_cached_result(&self, _cache_key: &str) -> Result<Option<String>> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_cached_result" })
  }

  async fn set_cached_result(&self, _cache_key: &str, _result: &str) -> Result<()> {
    Err(crate::QmdError::NotImplemented { op: "Storage::set_cached_result" })
  }

  async fn clear_cached_results(&self) -> Result<usize> {
    Err(crate::QmdError::NotImplemented { op: "Storage::clear_cached_results" })
  }

  async fn get_status(&self) -> Result<IndexStatus> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_status" })
  }

  async fn get_index_health(&self) -> Result<IndexHealthInfo> {
    Err(crate::QmdError::NotImplemented { op: "Storage::get_index_health" })
  }
}
