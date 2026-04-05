use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

// =============================================================================
// Query expansion
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedQuery {
  #[serde(rename = "type")]
  pub query_type: ExpandedQueryType,
  pub query:      String,
  pub line:       Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExpandedQueryType {
  Lex,
  Vec,
  Hyde,
}

// =============================================================================
// Search hooks + options (non-serializable runtime API)
// =============================================================================

pub trait SearchHooks: Send + Sync {
  fn on_strong_signal(&self, _top_score: f32) {}
  fn on_expand_start(&self) {}
  fn on_expand(&self, _original: &str, _expanded: &[ExpandedQuery], _elapsed_ms: u64) {}
  fn on_embed_start(&self, _count: usize) {}
  fn on_embed_done(&self, _elapsed_ms: u64) {}
  fn on_rerank_start(&self, _chunk_count: usize) {}
  fn on_rerank_done(&self, _elapsed_ms: u64) {}
}

#[derive(Clone, Default)]
pub struct HybridQueryOptions {
  pub collection:      Option<String>,
  pub limit:           Option<usize>,
  pub min_score:       Option<f32>,
  pub candidate_limit: Option<usize>,
  pub explain:         Option<bool>,
  pub intent:          Option<String>,
  pub skip_rerank:     Option<bool>,
  pub hooks:           Option<Arc<dyn SearchHooks>>,
}

impl std::fmt::Debug for HybridQueryOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("HybridQueryOptions").finish_non_exhaustive()
  }
}

#[derive(Clone, Default)]
pub struct StructuredSearchOptions {
  pub collections:     Option<Vec<String>>,
  pub limit:           Option<usize>,
  pub min_score:       Option<f32>,
  pub candidate_limit: Option<usize>,
  pub explain:         Option<bool>,
  pub intent:          Option<String>,
  pub skip_rerank:     Option<bool>,
  pub hooks:           Option<Arc<dyn SearchHooks>>,
}

impl std::fmt::Debug for StructuredSearchOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("StructuredSearchOptions").finish_non_exhaustive()
  }
}

// =============================================================================
// Core document + results (serializable)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResult {
  pub filepath: String,

  #[serde(rename = "displayPath")]
  pub display_path: String,

  pub title: String,

  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub context: Option<String>,

  pub hash:  String,
  pub docid: String,

  #[serde(rename = "collectionName")]
  pub collection_name: String,

  #[serde(rename = "modifiedAt")]
  pub modified_at: String,

  #[serde(rename = "bodyLength")]
  pub body_length: u64,

  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchSource {
  Fts,
  Vec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
  #[serde(flatten)]
  pub doc:    DocumentResult,
  pub score:  f32,
  pub source: SearchSource,

  #[serde(rename = "chunkPos", default, skip_serializing_if = "Option::is_none")]
  pub chunk_pos: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryType {
  Original,
  Lex,
  Vec,
  Hyde,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrfContributionTrace {
  #[serde(rename = "listIndex")]
  pub list_index:       u32,
  pub source:           SearchSource,
  #[serde(rename = "queryType")]
  pub query_type:       QueryType,
  pub query:            String,
  pub rank:             u32,
  pub weight:           f32,
  #[serde(rename = "backendScore")]
  pub backend_score:    f32,
  #[serde(rename = "rrfContribution")]
  pub rrf_contribution: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrfExplain {
  pub rank:           u32,
  #[serde(rename = "positionScore")]
  pub position_score: f32,
  pub weight:         f32,
  #[serde(rename = "baseScore")]
  pub base_score:     f32,
  #[serde(rename = "topRankBonus")]
  pub top_rank_bonus: f32,
  #[serde(rename = "totalScore")]
  pub total_score:    f32,
  pub contributions:  Vec<RrfContributionTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridQueryExplain {
  #[serde(rename = "ftsScores")]
  pub fts_scores:    Vec<f32>,
  #[serde(rename = "vectorScores")]
  pub vector_scores: Vec<f32>,
  pub rrf:           RrfExplain,
  #[serde(rename = "rerankScore")]
  pub rerank_score:  f32,
  #[serde(rename = "blendedScore")]
  pub blended_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridQueryResult {
  pub file:           String,
  #[serde(rename = "displayPath")]
  pub display_path:   String,
  pub title:          String,
  pub body:           String,
  #[serde(rename = "bestChunk")]
  pub best_chunk:     String,
  #[serde(rename = "bestChunkPos")]
  pub best_chunk_pos: u64,
  pub score:          f32,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub context:        Option<String>,
  pub docid:          String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub explain:        Option<HybridQueryExplain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentNotFound {
  pub error:         String,
  pub query:         String,
  #[serde(rename = "similarFiles")]
  pub similar_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiGetDocRef {
  pub filepath:     String,
  #[serde(rename = "displayPath")]
  pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultiGetResult {
  Found {
    doc:     DocumentResult,
    skipped: bool,
  },
  Skipped {
    doc:         MultiGetDocRef,
    skipped:     bool,
    #[serde(rename = "skipReason")]
    skip_reason: String,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
  pub name:         String,
  pub path:         Option<String>,
  pub pattern:      Option<String>,
  pub documents:    u64,
  #[serde(rename = "lastUpdated")]
  pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
  #[serde(rename = "totalDocuments")]
  pub total_documents:  u64,
  #[serde(rename = "needsEmbedding")]
  pub needs_embedding:  u64,
  #[serde(rename = "hasVectorIndex")]
  pub has_vector_index: bool,
  pub collections:      Vec<CollectionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexHealthInfo {
  #[serde(rename = "needsEmbedding")]
  pub needs_embedding: u64,
  #[serde(rename = "totalDocs")]
  pub total_docs:      u64,
  #[serde(rename = "daysStale")]
  pub days_stale:      Option<i64>,
}

// =============================================================================
// Indexing progress/result
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
  pub collection: String,
  pub file:       String,
  pub current:    usize,
  pub total:      usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
  pub collections:     usize,
  pub indexed:         usize,
  pub updated:         usize,
  pub unchanged:       usize,
  pub removed:         usize,
  #[serde(rename = "needsEmbedding")]
  pub needs_embedding: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexProgress {
  pub file:    String,
  pub current: usize,
  pub total:   usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReindexResult {
  pub indexed:          usize,
  pub updated:          usize,
  pub unchanged:        usize,
  pub removed:          usize,
  #[serde(rename = "orphanedCleaned")]
  pub orphaned_cleaned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedProgress {
  #[serde(rename = "chunksEmbedded")]
  pub chunks_embedded: usize,
  #[serde(rename = "totalChunks")]
  pub total_chunks:    usize,
  #[serde(rename = "bytesProcessed")]
  pub bytes_processed: u64,
  #[serde(rename = "totalBytes")]
  pub total_bytes:     u64,
  pub errors:          usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResult {
  #[serde(rename = "docsProcessed")]
  pub docs_processed:  usize,
  #[serde(rename = "chunksEmbedded")]
  pub chunks_embedded: usize,
  pub errors:          usize,
  #[serde(rename = "durationMs")]
  pub duration_ms:     u64,
}

// =============================================================================
// Collections config (YAML/inline parity with TS)
// =============================================================================

pub type ContextMap = BTreeMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
  pub path:               String,
  pub pattern:            String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ignore:             Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub context:            Option<ContextMap>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub update:             Option<String>,
  #[serde(rename = "includeByDefault", default = "default_include_by_default")]
  pub include_by_default: bool,
}

fn default_include_by_default() -> bool {
  true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
  pub global_context: Option<String>,
  pub collections:    BTreeMap<String, Collection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedCollection {
  pub name:       String,
  #[serde(flatten)]
  pub collection: Collection,
}

// =============================================================================
// SDK-facing store options (mirrors src/index.ts)
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub query:       Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub queries:     Option<Vec<ExpandedQuery>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub intent:      Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub rerank:      Option<bool>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub collection:  Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub collections: Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub limit:       Option<usize>,
  #[serde(rename = "minScore", default, skip_serializing_if = "Option::is_none")]
  pub min_score:   Option<f32>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub explain:     Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LexSearchOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub limit:      Option<usize>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub collection: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorSearchOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub limit:      Option<usize>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub collection: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpandQueryOptions {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub intent: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetOptions {
  #[serde(rename = "includeBody", default, skip_serializing_if = "Option::is_none")]
  pub include_body: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetBodyOptions {
  #[serde(rename = "fromLine", default, skip_serializing_if = "Option::is_none")]
  pub from_line: Option<u32>,
  #[serde(rename = "maxLines", default, skip_serializing_if = "Option::is_none")]
  pub max_lines: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiGetOptions {
  #[serde(rename = "includeBody", default, skip_serializing_if = "Option::is_none")]
  pub include_body: Option<bool>,
  #[serde(rename = "maxBytes", default, skip_serializing_if = "Option::is_none")]
  pub max_bytes:    Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiGetResponse {
  pub docs:   Vec<MultiGetResult>,
  pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
  pub collection: String,
  pub path:       String,
  pub context:    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCollectionOptions {
  pub path:    String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub pattern: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ignore:  Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionListItem {
  pub name:               String,
  pub pwd:                String,
  pub glob_pattern:       String,
  pub doc_count:          u64,
  pub active_count:       u64,
  pub last_modified:      Option<String>,
  #[serde(rename = "includeByDefault")]
  pub include_by_default: bool,
}

pub type UpdateProgressFn = Arc<dyn Fn(UpdateProgress) + Send + Sync>;

#[derive(Clone, Default)]
pub struct UpdateOptions {
  pub collections: Option<Vec<String>>,
  pub on_progress: Option<UpdateProgressFn>,
}

impl std::fmt::Debug for UpdateOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UpdateOptions").finish_non_exhaustive()
  }
}

pub type EmbedProgressFn = Arc<dyn Fn(EmbedProgress) + Send + Sync>;

#[derive(Clone, Default)]
pub struct EmbedOptions {
  pub force:              Option<bool>,
  pub model:              Option<String>,
  pub max_docs_per_batch: Option<usize>,
  pub max_batch_bytes:    Option<usize>,
  pub on_progress:        Option<EmbedProgressFn>,
}

impl std::fmt::Debug for EmbedOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EmbedOptions").finish_non_exhaustive()
  }
}
