use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use persistence::prelude::DbConnection;
use sha2::Digest;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;

use crate::Collection;
use crate::CollectionConfig;
use crate::CollectionListItem;
use crate::ContextItem;
use crate::DocumentResult;
use crate::EmbeddingCandidate;
use crate::EmbeddingModel;
use crate::EmbeddingRecord;
use crate::EmbeddingRepository;
use crate::IndexHealthInfo;
use crate::IndexStatus;
use crate::LexSearchOptions;
use crate::MultiGetOptions;
use crate::MultiGetResponse;
use crate::MultiGetResult;
use crate::NamedCollection;
use crate::QmdError;
use crate::Result;
use crate::SearchResult;
use crate::SearchSource;
use crate::Storage;
use crate::VectorSearchOptions;

const QMD_COLLECTIONS_TABLE: &str = "qmd_collections";
const QMD_CONTEXTS_TABLE: &str = "qmd_contexts";
const QMD_META_TABLE: &str = "qmd_meta";
const QMD_CONTENTS_TABLE: &str = "qmd_contents";
const QMD_DOCUMENTS_TABLE: &str = "qmd_documents";
const QMD_EMBEDDINGS_TABLE: &str = "qmd_embeddings";
const QMD_LLM_CACHE_TABLE: &str = "qmd_llm_cache";

const META_GLOBAL_CONTEXT_ID: &str = "global_context";

const QMD_EMBEDDINGS_VECTOR_INDEX: &str = "qmd_embeddings_embedding_hnsw";
const DEFAULT_HNSW_EFC: i64 = 40;

fn db_err(e: surrealdb::Error) -> QmdError {
  QmdError::Storage { message: e.to_string() }
}

fn sha256_hex(input: &str) -> String {
  let mut hasher = sha2::Sha256::new();
  hasher.update(input.as_bytes());
  hex::encode(hasher.finalize())
}

fn collection_rid(name: &str) -> RecordId {
  RecordId::new(QMD_COLLECTIONS_TABLE, name.to_string())
}

fn meta_rid(id: &str) -> RecordId {
  RecordId::new(QMD_META_TABLE, id.to_string())
}

fn content_rid(hash: &str) -> RecordId {
  RecordId::new(QMD_CONTENTS_TABLE, hash.to_string())
}

fn context_rid(collection: &str, path_prefix: &str) -> RecordId {
  let key = sha256_hex(&format!("{collection}\n{path_prefix}"));
  RecordId::new(QMD_CONTEXTS_TABLE, key)
}

fn document_rid(collection: &str, path: &str) -> RecordId {
  let key = sha256_hex(&format!("{collection}\n{path}"));
  RecordId::new(QMD_DOCUMENTS_TABLE, key)
}

fn llm_cache_rid(cache_key: &str) -> RecordId {
  RecordId::new(QMD_LLM_CACHE_TABLE, cache_key.to_string())
}

fn normalize(path: &str) -> String {
  crate::normalize_path_separators(path)
}

fn parse_dimension_from_index_def(def: &str) -> Option<u32> {
  let mut it = def.split_whitespace();
  while let Some(tok) = it.next() {
    if tok.eq_ignore_ascii_case("DIMENSION") {
      return it.next().and_then(|s| s.trim_end_matches(';').parse::<u32>().ok());
    }
  }
  None
}

fn is_hex_6(s: &str) -> bool {
  s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn expand_home(path: &str) -> String {
  if let Some(rest) = path.strip_prefix("~/") {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| "/".to_string());
    return format!("{home}/{rest}");
  }
  path.to_string()
}

fn get_real_path(path: &str) -> String {
  std::fs::canonicalize(path).ok().and_then(|p| p.to_str().map(|s| s.to_string())).unwrap_or_else(|| path.to_string())
}

#[derive(Clone)]
pub struct SurrealStorage {
  db:                          DbConnection,
  embeddings_vector_index_dim: Arc<AtomicU32>,
}

impl std::fmt::Debug for SurrealStorage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SurrealStorage").finish_non_exhaustive()
  }
}

impl SurrealStorage {
  pub fn new(db: DbConnection) -> Self {
    Self { db, embeddings_vector_index_dim: Arc::new(AtomicU32::new(0)) }
  }

  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_COLLECTIONS_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_CONTEXTS_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_META_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_CONTENTS_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_DOCUMENTS_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_EMBEDDINGS_TABLE} SCHEMALESS;")).await.map_err(db_err)?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {QMD_LLM_CACHE_TABLE} SCHEMALESS;")).await.map_err(db_err)?;

    // Help SurrealDB treat embeddings as float vectors (required for vector indexes).
    db.query(format!("DEFINE FIELD IF NOT EXISTS embedding ON TABLE {QMD_EMBEDDINGS_TABLE} TYPE array;"))
      .await
      .map_err(db_err)?;
    db.query(format!("DEFINE FIELD IF NOT EXISTS embedding.* ON TABLE {QMD_EMBEDDINGS_TABLE} TYPE float;"))
      .await
      .map_err(db_err)?;

    // Full-text search analyzer + index (used by Storage::search_lex).
    db.query(
      "DEFINE ANALYZER IF NOT EXISTS qmd_text TOKENIZERS class, blank, punct, camel FILTERS lowercase, ascii, snowball(english);",
    )
    .await
    .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_contents_doc_ft ON TABLE {QMD_CONTENTS_TABLE} FIELDS doc FULLTEXT ANALYZER qmd_text BM25(1.2, 0.75);"
    ))
    .await
    .map_err(db_err)?;

    // Uniqueness / lookup helpers.
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_collections_name_unique ON TABLE {QMD_COLLECTIONS_TABLE} FIELDS name UNIQUE;"
    ))
    .await
    .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_documents_collection_path_unique ON TABLE {QMD_DOCUMENTS_TABLE} FIELDS collection, path UNIQUE;"
    ))
    .await
    .map_err(db_err)?;
    db.query(format!("DEFINE INDEX IF NOT EXISTS qmd_documents_hash_idx ON TABLE {QMD_DOCUMENTS_TABLE} FIELDS hash;"))
      .await
      .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_embeddings_hash_model_seq_unique ON TABLE {QMD_EMBEDDINGS_TABLE} FIELDS hash, model, seq UNIQUE;"
    ))
    .await
    .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_embeddings_hash_model_idx ON TABLE {QMD_EMBEDDINGS_TABLE} FIELDS hash, model;"
    ))
    .await
    .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_contexts_collection_prefix_unique ON TABLE {QMD_CONTEXTS_TABLE} FIELDS collection, path_prefix UNIQUE;"
    ))
    .await
    .map_err(db_err)?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS qmd_llm_cache_key_unique ON TABLE {QMD_LLM_CACHE_TABLE} FIELDS cache_key UNIQUE;"
    ))
    .await
    .map_err(db_err)?;

    Ok(())
  }

  async fn embeddings_vector_index_dim_from_info(&self) -> Result<Option<u32>> {
    let info: Option<serde_json::Value> = self
      .db
      .query(format!("INFO FOR TABLE {QMD_EMBEDDINGS_TABLE};"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let Some(info) = info else {
      return Ok(None);
    };

    let indexes = info.get("indexes").and_then(|v| v.as_object());
    let Some(indexes) = indexes else {
      return Ok(None);
    };

    let def = indexes.get(QMD_EMBEDDINGS_VECTOR_INDEX).and_then(|v| v.as_str());
    Ok(def.and_then(parse_dimension_from_index_def))
  }

  async fn embeddings_count(&self) -> Result<u64> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let rows: Vec<CountRow> = self
      .db
      .query(format!("SELECT count() AS count FROM {QMD_EMBEDDINGS_TABLE} GROUP ALL;"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    Ok(rows.first().map(|r| r.count.max(0) as u64).unwrap_or(0))
  }

  async fn ensure_embeddings_vector_index(&self, dim: u32) -> Result<bool> {
    if dim == 0 {
      return Ok(false);
    }

    let cached = self.embeddings_vector_index_dim.load(Ordering::Relaxed);
    if cached == dim {
      return Ok(true);
    }

    let existing_dim = self.embeddings_vector_index_dim_from_info().await?;
    if let Some(existing_dim) = existing_dim {
      self.embeddings_vector_index_dim.store(existing_dim, Ordering::Relaxed);

      if existing_dim == dim {
        return Ok(true);
      }

      // If the table is empty, we can safely overwrite the index definition with the new dimension.
      if self.embeddings_count().await? == 0 {
        self
          .db
          .query(format!(
            "DEFINE INDEX OVERWRITE {QMD_EMBEDDINGS_VECTOR_INDEX} ON TABLE {QMD_EMBEDDINGS_TABLE} FIELDS embedding HNSW DIMENSION {dim} DIST COSINE;"
          ))
          .await
          .map_err(db_err)?;
        self.embeddings_vector_index_dim.store(dim, Ordering::Relaxed);
        return Ok(true);
      }

      // Mixed dimensions can't share a single vector index.
      return Ok(false);
    }

    self
      .db
      .query(format!(
        "DEFINE INDEX IF NOT EXISTS {QMD_EMBEDDINGS_VECTOR_INDEX} ON TABLE {QMD_EMBEDDINGS_TABLE} FIELDS embedding HNSW DIMENSION {dim} DIST COSINE;"
      ))
      .await
      .map_err(db_err)?;
    self.embeddings_vector_index_dim.store(dim, Ordering::Relaxed);
    Ok(true)
  }

  async fn resolve_fs_path(&self, input: &str) -> Result<Option<(String, String)>> {
    let raw = expand_home(input);
    if crate::is_virtual_path(&raw) || raw.starts_with("qmd://") {
      if let Some(vp) = crate::parse_virtual_path(&raw) {
        return Ok(Some((vp.collection_name, vp.path)));
      }
      return Ok(None);
    }

    let path = if crate::is_absolute_path(&raw) {
      raw
    } else {
      let pwd = std::env::var("PWD").ok().unwrap_or_else(|| ".".to_string());
      crate::resolve(&[pwd.as_str(), raw.as_str()])?
    };

    let real_path = normalize(&get_real_path(&path));
    let collections = self.list_collections().await?;

    let mut best: Option<(String, String)> = None; // (name, collection_path)
    for c in collections {
      let coll_path = normalize(&expand_home(&c.collection.path));
      if real_path == coll_path || real_path.starts_with(&(coll_path.clone() + "/")) {
        match &best {
          Some((_n, best_path)) if best_path.len() >= coll_path.len() => {}
          _ => best = Some((c.name, coll_path)),
        }
      }
    }

    let Some((coll_name, coll_path)) = best else {
      return Ok(None);
    };

    let rel = if real_path == coll_path { String::new() } else { real_path[(coll_path.len() + 1)..].to_string() };

    let rel = if rel.is_empty() { rel } else { crate::handelize(&rel).unwrap_or(rel) };

    Ok(Some((coll_name, rel)))
  }

  async fn load_content_meta(&self, hash: &str) -> Result<Option<(u64, Option<String>)>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      doc:       Option<String>,
      doc_bytes: Option<u64>,
    }

    let rid = content_rid(hash);
    let row: Option<Row> = self.db.select(rid).await.map_err(db_err)?;
    Ok(row.map(|r| (r.doc_bytes.unwrap_or(0), r.doc)))
  }

  async fn load_content_meta_map(
    &self,
    hashes: &[String],
    include_body: bool,
  ) -> Result<HashMap<String, (u64, Option<String>)>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      hash:      String,
      doc:       Option<String>,
      doc_bytes: Option<u64>,
    }

    if hashes.is_empty() {
      return Ok(HashMap::new());
    }

    let select = if include_body { "hash, doc, doc_bytes" } else { "hash, doc_bytes" };
    let rows: Vec<Row> = self
      .db
      .query(format!("SELECT {select} FROM {QMD_CONTENTS_TABLE} WHERE hash IN $hashes"))
      .bind(("hashes", hashes.to_vec()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(rows.into_iter().map(|row| (row.hash, (row.doc_bytes.unwrap_or(0), row.doc))).collect())
  }

  async fn build_doc_result(
    &self,
    collection: &str,
    path: &str,
    title: &str,
    hash: &str,
    modified_at: &str,
    include_body: bool,
  ) -> Result<DocumentResult> {
    let (body_length, body) = match self.load_content_meta(hash).await? {
      Some((bytes, maybe_doc)) => {
        if include_body {
          (bytes, maybe_doc)
        } else {
          (bytes, None)
        }
      }
      None => (0, None),
    };

    Ok(Self::build_doc_result_from_meta(collection, path, title, hash, modified_at, body_length, body))
  }

  fn build_doc_result_from_meta(
    collection: &str,
    path: &str,
    title: &str,
    hash: &str,
    modified_at: &str,
    body_length: u64,
    body: Option<String>,
  ) -> DocumentResult {
    let vp = crate::build_virtual_path(collection, path);
    let display_path = if path.is_empty() { collection.to_string() } else { format!("{collection}/{path}") };

    DocumentResult {
      filepath: vp,
      display_path,
      title: title.to_string(),
      context: None,
      hash: hash.to_string(),
      docid: crate::get_docid(hash),
      collection_name: collection.to_string(),
      modified_at: modified_at.to_string(),
      body_length,
      body,
    }
  }
}

// =============================================================================
// EmbeddingRepository
// =============================================================================

#[async_trait]
impl EmbeddingRepository for SurrealStorage {
  async fn get_hashes_for_embedding(&self) -> Result<Vec<EmbeddingCandidate>> {
    let model = std::env::var("QMD_EMBED_MODEL").ok().unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL.to_string());

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocHashRow {
      hash:       String,
      path:       String,
      collection: String,
    }

    let docs: Vec<DocHashRow> = self
      .db
      .query(format!("SELECT hash, path, collection FROM {QMD_DOCUMENTS_TABLE} WHERE active = true"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    if docs.is_empty() {
      return Ok(Vec::new());
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct EmbeddedHashRow {
      hash: String,
    }

    let embedded_hashes: Vec<EmbeddedHashRow> = self
      .db
      .query(format!("SELECT hash FROM {QMD_EMBEDDINGS_TABLE} WHERE model = $model GROUP BY hash"))
      .bind(("model", model.clone()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let embedded: HashSet<String> = embedded_hashes.into_iter().map(|r| r.hash).collect();

    // Pick one representative path per hash; content is shared by hash.
    let mut by_hash: HashMap<String, (String, String)> = HashMap::new(); // hash -> (collection, path)
    for d in docs {
      by_hash.entry(d.hash).or_insert((d.collection, d.path));
    }

    let needed: Vec<String> = by_hash.keys().filter(|h| !embedded.contains(*h)).cloned().collect();

    if needed.is_empty() {
      return Ok(Vec::new());
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct ContentRow {
      hash: String,
      doc:  String,
    }

    let contents: Vec<ContentRow> = self
      .db
      .query(format!("SELECT hash, doc FROM {QMD_CONTENTS_TABLE} WHERE hash IN $hashes"))
      .bind(("hashes", needed.clone()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let mut doc_by_hash: HashMap<String, String> = HashMap::new();
    for c in contents {
      doc_by_hash.insert(c.hash, c.doc);
    }

    let mut out: Vec<EmbeddingCandidate> = Vec::new();
    for h in needed {
      if let Some(body) = doc_by_hash.get(&h) {
        let (_collection, path) = by_hash.get(&h).cloned().unwrap_or_default();
        out.push(EmbeddingCandidate { hash: h, body: body.clone(), path });
      }
    }

    Ok(out)
  }

  async fn clear_all_embeddings(&self) -> Result<()> {
    self.db.query(format!("DELETE FROM {QMD_EMBEDDINGS_TABLE};")).await.map_err(db_err)?;
    self.embeddings_vector_index_dim.store(0, Ordering::Relaxed);
    Ok(())
  }

  async fn create_embedding(&self, model: EmbeddingModel) -> Result<EmbeddingRecord> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      hash:        String,
      seq:         u32,
      pos:         u32,
      embedding:   Vec<f32>,
      model:       String,
      embedded_at: String,
    }

    let key = sha256_hex(&format!("{}\n{}\n{}", model.hash, model.model, model.seq));
    let rid = RecordId::new(QMD_EMBEDDINGS_TABLE, key.clone());
    let db_model = DbModel {
      hash:        model.hash.clone(),
      seq:         model.seq,
      pos:         model.pos,
      embedding:   model.embedding.clone(),
      model:       model.model.clone(),
      embedded_at: model.embedded_at.clone(),
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid.clone()))
      .bind(("content", db_model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    // Best-effort: keep a vector index available for KNN queries without making embedding writes fail.
    let dim = model.embedding.len().min(u32::MAX as usize) as u32;
    if dim > 0 {
      let _ = self.ensure_embeddings_vector_index(dim).await;
    }

    Ok(EmbeddingRecord { id: key, model })
  }
}

// =============================================================================
// Storage
// =============================================================================

#[async_trait]
impl Storage for SurrealStorage {
  async fn list_collections(&self) -> Result<Vec<NamedCollection>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      name:               String,
      path:               String,
      pattern:            String,
      ignore:             Option<Vec<String>>,
      update:             Option<String>,
      include_by_default: Option<bool>,
    }

    let mut rows: Vec<Row> = self
      .db
      .query(format!(
        "SELECT name, path, pattern, ignore, `update`, include_by_default FROM {QMD_COLLECTIONS_TABLE} ORDER BY name"
      ))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    // Defensive: if older records lack include_by_default, default to true.
    for r in &mut rows {
      if r.include_by_default.is_none() {
        r.include_by_default = Some(true);
      }
    }

    Ok(
      rows
        .into_iter()
        .map(|r| NamedCollection {
          name:       r.name.clone(),
          collection: Collection {
            path:               r.path,
            pattern:            r.pattern,
            ignore:             r.ignore,
            context:            None,
            update:             r.update,
            include_by_default: r.include_by_default.unwrap_or(true),
          },
        })
        .collect(),
    )
  }

  async fn list_collections_info(&self) -> Result<Vec<CollectionListItem>> {
    let collections = self.list_collections().await?;

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CollectionCountRow {
      collection: String,
      count:      i64,
    }
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct ModifiedRow {
      collection:  String,
      modified_at: Option<String>,
    }

    let doc_counts: Vec<CollectionCountRow> = self
      .db
      .query(format!("SELECT collection, count() AS count FROM {QMD_DOCUMENTS_TABLE} GROUP BY collection"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let doc_counts: HashMap<String, u64> =
      doc_counts.into_iter().map(|row| (row.collection, row.count.max(0) as u64)).collect();

    let active_counts: Vec<CollectionCountRow> = self
      .db
      .query(format!(
        "SELECT collection, count() AS count FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP BY collection"
      ))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let active_counts: HashMap<String, u64> =
      active_counts.into_iter().map(|row| (row.collection, row.count.max(0) as u64)).collect();

    let latest_rows: Vec<ModifiedRow> = self
      .db
      .query(format!("SELECT collection, modified_at FROM {QMD_DOCUMENTS_TABLE} ORDER BY collection, modified_at DESC"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let mut latest_by_collection: HashMap<String, String> = HashMap::new();
    for row in latest_rows {
      if let Some(modified_at) = row.modified_at {
        latest_by_collection.entry(row.collection).or_insert(modified_at);
      }
    }

    let mut out: Vec<CollectionListItem> = Vec::with_capacity(collections.len());
    for c in collections {
      let doc_count = doc_counts.get(&c.name).copied().unwrap_or(0);
      let active_count = active_counts.get(&c.name).copied().unwrap_or(0);
      let last_modified = latest_by_collection.get(&c.name).cloned();

      out.push(CollectionListItem {
        name: c.name.clone(),
        pwd: c.collection.path.clone(),
        glob_pattern: c.collection.pattern.clone(),
        doc_count,
        active_count,
        last_modified,
        include_by_default: c.collection.include_by_default,
      });
    }

    Ok(out)
  }

  async fn upsert_collection(&self, name: &str, collection: &crate::Collection) -> Result<()> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      name:               String,
      path:               String,
      pattern:            String,
      ignore:             Option<Vec<String>>,
      update:             Option<String>,
      include_by_default: bool,
    }

    let rid = collection_rid(name);
    let model = DbModel {
      name:               name.to_string(),
      path:               normalize(&expand_home(&collection.path)),
      pattern:            collection.pattern.clone(),
      ignore:             collection.ignore.clone(),
      update:             collection.update.clone(),
      include_by_default: collection.include_by_default,
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(())
  }

  async fn delete_collection(&self, name: &str) -> Result<bool> {
    let rid = collection_rid(name);

    let existed: Option<serde_json::Value> = self.db.select(rid.clone()).await.map_err(db_err)?;
    if existed.is_none() {
      return Ok(false);
    }

    let _: Option<serde_json::Value> = self.db.delete(rid).await.map_err(db_err)?;
    self
      .db
      .query(format!("DELETE FROM {QMD_CONTEXTS_TABLE} WHERE collection = $c;"))
      .bind(("c", name.to_string()))
      .await
      .map_err(db_err)?;
    self
      .db
      .query(format!("DELETE FROM {QMD_DOCUMENTS_TABLE} WHERE collection = $c;"))
      .bind(("c", name.to_string()))
      .await
      .map_err(db_err)?;

    // Keep large content/vector tables tidy after destructive collection removal.
    let _ = self.cleanup_orphaned_content().await?;
    let _ = self.cleanup_orphaned_vectors().await?;

    Ok(true)
  }

  async fn rename_collection(&self, old_name: &str, new_name: &str) -> Result<bool> {
    if old_name.trim().is_empty() || new_name.trim().is_empty() {
      return Err(QmdError::InvalidArgument { message: "rename_collection requires non-empty names".to_string() });
    }

    // Fetch old collection
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      name:               String,
      path:               String,
      pattern:            String,
      ignore:             Option<Vec<String>>,
      update:             Option<String>,
      include_by_default: Option<bool>,
    }

    let rows: Vec<Row> = self
      .db
      .query(format!(
        "SELECT name, path, pattern, ignore, `update`, include_by_default FROM {QMD_COLLECTIONS_TABLE} WHERE name = $n LIMIT 1"
      ))
      .bind(("n", old_name.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let Some(old) = rows.into_iter().next() else {
      return Ok(false);
    };

    // Ensure new doesn't exist
    let new_exists: Vec<Row> = self
      .db
      .query(format!("SELECT name FROM {QMD_COLLECTIONS_TABLE} WHERE name = $n LIMIT 1"))
      .bind(("n", new_name.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    if !new_exists.is_empty() {
      return Err(QmdError::InvalidArgument { message: format!("Collection '{new_name}' already exists") });
    }

    // Create a new collection record.
    let include_by_default = old.include_by_default.unwrap_or(true);
    self
      .upsert_collection(
        new_name,
        &Collection {
          path: old.path,
          pattern: old.pattern,
          ignore: old.ignore,
          context: None,
          update: old.update,
          include_by_default,
        },
      )
      .await?;

    // Re-key dependent records to preserve our deterministic RecordId scheme
    // (document/context IDs are derived from collection + path).

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CtxRow {
      path_prefix: String,
      context:     String,
    }

    let contexts: Vec<CtxRow> = self
      .db
      .query(format!("SELECT path_prefix, context FROM {QMD_CONTEXTS_TABLE} WHERE collection = $c"))
      .bind(("c", old_name.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    for ctx in contexts {
      self.upsert_context(new_name, &ctx.path_prefix, &ctx.context).await?;
    }
    self
      .db
      .query(format!("DELETE FROM {QMD_CONTEXTS_TABLE} WHERE collection = $c;"))
      .bind(("c", old_name.to_string()))
      .await
      .map_err(db_err)?;

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocRow {
      path:        String,
      title:       String,
      hash:        String,
      created_at:  String,
      modified_at: String,
      active:      bool,
    }

    let docs: Vec<DocRow> = self
      .db
      .query(format!(
        "SELECT path, title, hash, created_at, modified_at, active FROM {QMD_DOCUMENTS_TABLE} WHERE collection = $c"
      ))
      .bind(("c", old_name.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    for d in docs {
      self.upsert_document(new_name, &d.path, &d.title, &d.hash, &d.created_at, &d.modified_at, d.active).await?;
    }
    self
      .db
      .query(format!("DELETE FROM {QMD_DOCUMENTS_TABLE} WHERE collection = $c;"))
      .bind(("c", old_name.to_string()))
      .await
      .map_err(db_err)?;

    let _: Option<serde_json::Value> = self.db.delete(collection_rid(old_name)).await.map_err(db_err)?;

    Ok(true)
  }

  async fn get_global_context(&self) -> Result<Option<String>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      context: Option<String>,
    }

    let row: Option<Row> = self.db.select(meta_rid(META_GLOBAL_CONTEXT_ID)).await.map_err(db_err)?;
    Ok(row.and_then(|r| r.context))
  }

  async fn set_global_context(&self, context: Option<&str>) -> Result<()> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      id:      String,
      context: Option<String>,
    }

    let rid = meta_rid(META_GLOBAL_CONTEXT_ID);
    let model = DbModel { id: META_GLOBAL_CONTEXT_ID.to_string(), context: context.map(|s| s.to_string()) };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(())
  }

  async fn list_contexts(&self) -> Result<Vec<ContextItem>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      collection:  String,
      path_prefix: String,
      context:     String,
    }

    let rows: Vec<Row> = self
      .db
      .query(format!(
        "SELECT collection, path_prefix, context FROM {QMD_CONTEXTS_TABLE} ORDER BY collection, path_prefix"
      ))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(
      rows
        .into_iter()
        .map(|r| ContextItem { collection: r.collection, path: r.path_prefix, context: r.context })
        .collect(),
    )
  }

  async fn upsert_context(&self, collection: &str, path_prefix: &str, context: &str) -> Result<bool> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      collection:  String,
      path_prefix: String,
      context:     String,
    }

    let rid = context_rid(collection, path_prefix);
    let model = DbModel {
      collection:  collection.to_string(),
      path_prefix: path_prefix.to_string(),
      context:     context.to_string(),
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(true)
  }

  async fn remove_context(&self, collection: &str, path_prefix: &str) -> Result<bool> {
    let rid = context_rid(collection, path_prefix);
    let deleted: Option<serde_json::Value> = self.db.delete(rid).await.map_err(db_err)?;
    Ok(deleted.is_some())
  }

  async fn sync_config(&self, config: &CollectionConfig) -> Result<()> {
    self.set_global_context(config.global_context.as_deref()).await?;

    for (name, coll) in &config.collections {
      self.upsert_collection(name, coll).await?;
      if let Some(ctx) = &coll.context {
        for (prefix, text) in ctx {
          self.upsert_context(name, prefix, text).await?;
        }
      }
    }

    Ok(())
  }

  async fn upsert_content(&self, hash: &str, doc: &str, created_at: &str) -> Result<()> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      hash:       String,
      doc:        String,
      doc_bytes:  u64,
      created_at: String,
    }

    let rid = content_rid(hash);
    let model = DbModel {
      hash:       hash.to_string(),
      doc:        doc.to_string(),
      doc_bytes:  doc.as_bytes().len() as u64,
      created_at: created_at.to_string(),
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(())
  }

  async fn get_content(&self, hash: &str) -> Result<Option<String>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      doc: String,
    }

    let row: Option<Row> = self.db.select(content_rid(hash)).await.map_err(db_err)?;
    Ok(row.map(|r| r.doc))
  }

  async fn upsert_document(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    hash: &str,
    created_at: &str,
    modified_at: &str,
    active: bool,
  ) -> Result<()> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      collection:  String,
      path:        String,
      title:       String,
      hash:        String,
      created_at:  String,
      modified_at: String,
      active:      bool,
    }

    let rid = document_rid(collection_name, path);
    let model = DbModel {
      collection: collection_name.to_string(),
      path: path.to_string(),
      title: title.to_string(),
      hash: hash.to_string(),
      created_at: created_at.to_string(),
      modified_at: modified_at.to_string(),
      active,
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(())
  }

  async fn find_active_document(&self, collection_name: &str, path: &str) -> Result<Option<(String, String)>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      hash:  String,
      title: String,
    }

    let rows: Vec<Row> = self
      .db
      .query(format!(
        "SELECT hash, title FROM {QMD_DOCUMENTS_TABLE} WHERE collection = $c AND path = $p AND active = true LIMIT 1"
      ))
      .bind(("c", collection_name.to_string()))
      .bind(("p", path.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(rows.into_iter().next().map(|r| (r.hash, r.title)))
  }

  async fn update_document_title(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    modified_at: &str,
  ) -> Result<()> {
    let rid = document_rid(collection_name, path);
    self
      .db
      .query("UPDATE $rid SET title = $title, modified_at = $modified_at;")
      .bind(("rid", rid))
      .bind(("title", title.to_string()))
      .bind(("modified_at", modified_at.to_string()))
      .await
      .map_err(db_err)?;
    Ok(())
  }

  async fn update_document_hash(
    &self,
    collection_name: &str,
    path: &str,
    title: &str,
    hash: &str,
    modified_at: &str,
  ) -> Result<()> {
    let rid = document_rid(collection_name, path);
    self
      .db
      .query("UPDATE $rid SET title = $title, hash = $hash, modified_at = $modified_at, active = true;")
      .bind(("rid", rid))
      .bind(("title", title.to_string()))
      .bind(("hash", hash.to_string()))
      .bind(("modified_at", modified_at.to_string()))
      .await
      .map_err(db_err)?;
    Ok(())
  }

  async fn deactivate_document(&self, collection_name: &str, path: &str) -> Result<()> {
    let rid = document_rid(collection_name, path);
    self.db.query("UPDATE $rid SET active = false;").bind(("rid", rid)).await.map_err(db_err)?;
    Ok(())
  }

  async fn list_active_document_paths(&self, collection_name: &str) -> Result<Vec<String>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      path: String,
    }

    let rows: Vec<Row> = self
      .db
      .query(format!("SELECT path FROM {QMD_DOCUMENTS_TABLE} WHERE collection = $c AND active = true ORDER BY path"))
      .bind(("c", collection_name.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    Ok(rows.into_iter().map(|r| r.path).collect())
  }

  async fn search_lex(&self, query: &str, options: &LexSearchOptions) -> Result<Vec<SearchResult>> {
    let q = query.trim();
    if q.is_empty() {
      return Ok(Vec::new());
    }

    let limit = options.limit.unwrap_or(10).max(1) as usize;

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct HitRow {
      hash:      String,
      score:     f32,
      doc_bytes: Option<u64>,
    }

    let hits: Vec<HitRow> = self
      .db
      .query(format!(
        "SELECT hash, doc_bytes, search::score(0) AS score FROM {QMD_CONTENTS_TABLE} WHERE doc @0@ $q ORDER BY score DESC LIMIT $limit"
      ))
      .bind(("q", q.to_string()))
      .bind(("limit", limit as i64))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    if hits.is_empty() {
      return Ok(Vec::new());
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocRow {
      collection:  String,
      path:        String,
      title:       String,
      hash:        String,
      modified_at: String,
    }

    let hashes: Vec<String> = hits.iter().map(|h| h.hash.clone()).collect();
    let mut sql = format!(
      "SELECT collection, path, title, hash, modified_at FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND hash IN $hashes"
    );
    if options.collection.is_some() {
      sql.push_str(" AND collection = $collection");
    }
    sql.push_str(" ORDER BY collection, path");

    let mut query = self.db.query(sql).bind(("hashes", hashes));
    if let Some(collection) = &options.collection {
      query = query.bind(("collection", collection.clone()));
    }

    let docs: Vec<DocRow> = query.await.map_err(db_err)?.take(0).map_err(db_err)?;
    let docs_by_hash: HashMap<String, DocRow> = docs.into_iter().map(|doc| (doc.hash.clone(), doc)).collect();

    let mut out: Vec<SearchResult> = Vec::new();
    for h in hits {
      let Some(d) = docs_by_hash.get(&h.hash) else {
        continue;
      };

      let doc = Self::build_doc_result_from_meta(
        &d.collection,
        &d.path,
        &d.title,
        &d.hash,
        &d.modified_at,
        h.doc_bytes.unwrap_or(0),
        None,
      );

      out.push(SearchResult { doc, score: h.score, source: SearchSource::Fts, chunk_pos: None });
    }

    Ok(out)
  }

  async fn search_vector(
    &self,
    embedding: &[f32],
    model: &str,
    options: &VectorSearchOptions,
  ) -> Result<Vec<SearchResult>> {
    let limit = options.limit.unwrap_or(10).max(1) as usize;
    if embedding.is_empty() {
      return Ok(Vec::new());
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct ChunkHit {
      hash:     String,
      pos:      u32,
      distance: f64,
    }

    let (hash_filter, collection_filter) = match &options.collection {
      Some(coll) if !coll.trim().is_empty() => {
        #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
        struct HashRow {
          hash: String,
        }

        let hashes: Vec<HashRow> = self
          .db
          .query(format!(
            "SELECT hash FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND collection = $c GROUP BY hash"
          ))
          .bind(("c", coll.to_string()))
          .await
          .map_err(db_err)?
          .take(0)
          .map_err(db_err)?;

        let hs: Vec<String> = hashes.into_iter().map(|r| r.hash).collect();
        if hs.is_empty() {
          return Ok(Vec::new());
        }
        (Some(hs), Some(coll.clone()))
      }
      _ => (None, None),
    };

    let qvec: Vec<f64> = embedding.iter().map(|v| *v as f64).collect();
    let dim = qvec.len().min(u32::MAX as usize) as u32;
    let _ = self.ensure_embeddings_vector_index(dim).await;

    let requested = (limit * 25).max(limit) as i64;
    let index_ok = dim != 0 && self.embeddings_vector_index_dim.load(Ordering::Relaxed) == dim;

    let knn_mode = if index_ok {
      std::env::var("QMD_HNSW_EFC")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(DEFAULT_HNSW_EFC)
        .max(1)
        .to_string()
    } else {
      "COSINE".to_string()
    };

    let mut sql = format!(
      "SELECT hash, pos, vector::distance::knn() AS distance FROM {QMD_EMBEDDINGS_TABLE} WHERE model = $model AND embedding <| {requested}, {knn_mode} |> $qvec"
    );
    if hash_filter.is_some() {
      sql.push_str(" AND hash IN $hashes");
    }
    sql.push_str(" ORDER BY distance ASC LIMIT $limit");

    let mut q = self.db.query(sql).bind(("qvec", qvec)).bind(("model", model.to_string())).bind(("limit", requested));
    if let Some(hashes) = &hash_filter {
      q = q.bind(("hashes", hashes.clone()));
    }

    let hits: Vec<ChunkHit> = q.await.map_err(db_err)?.take(0).map_err(db_err)?;
    if hits.is_empty() {
      return Ok(Vec::new());
    }

    // Keep best chunk per hash.
    let mut best_by_hash: HashMap<String, (f64, u32)> = HashMap::new();
    for h in hits {
      let entry = best_by_hash.entry(h.hash).or_insert((h.distance, h.pos));
      if h.distance < entry.0 {
        *entry = (h.distance, h.pos);
      }
    }

    let mut hashes: Vec<String> = best_by_hash.keys().cloned().collect();
    hashes.sort();

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocRow {
      collection:  String,
      path:        String,
      title:       String,
      hash:        String,
      modified_at: String,
    }

    let mut docs: Vec<DocRow> = self
      .db
      .query(format!(
        "SELECT collection, path, title, hash, modified_at FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND hash IN $hashes"
      ))
      .bind(("hashes", hashes.clone()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    if let Some(coll) = &collection_filter {
      docs.retain(|d| &d.collection == coll);
    }

    let content_meta = self.load_content_meta_map(&hashes, false).await?;
    let mut out: Vec<SearchResult> = Vec::new();
    for d in docs {
      let Some((distance, pos)) = best_by_hash.get(&d.hash).copied() else {
        continue;
      };
      let (body_length, body) = content_meta.get(&d.hash).cloned().unwrap_or((0, None));
      let doc =
        Self::build_doc_result_from_meta(&d.collection, &d.path, &d.title, &d.hash, &d.modified_at, body_length, body);
      let score = 1.0f32 - (distance as f32);
      out.push(SearchResult { doc, score, source: SearchSource::Vec, chunk_pos: Some(pos as u64) });
    }

    out.sort_by(|a, b| b.score.total_cmp(&a.score));
    out.truncate(limit);
    Ok(out)
  }

  async fn get_document(&self, path_or_docid: &str, include_body: bool) -> Result<Option<DocumentResult>> {
    let input = path_or_docid.trim();
    if input.is_empty() {
      return Ok(None);
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocRow {
      collection:  String,
      path:        String,
      title:       String,
      hash:        String,
      modified_at: String,
      active:      bool,
    }

    // 1) Virtual path
    if crate::is_virtual_path(input) || input.starts_with("qmd://") {
      if let Some(vp) = crate::parse_virtual_path(input) {
        let rows: Vec<DocRow> = self
          .db
          .query(format!(
            "SELECT collection, path, title, hash, modified_at, active FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND collection = $c AND path = $p LIMIT 1"
          ))
          .bind(("c", vp.collection_name))
          .bind(("p", vp.path))
          .await
          .map_err(db_err)?
          .take(0)
          .map_err(db_err)?;

        let Some(d) = rows.into_iter().next() else {
          return Ok(None);
        };
        return Ok(Some(
          self.build_doc_result(&d.collection, &d.path, &d.title, &d.hash, &d.modified_at, include_body).await?,
        ));
      }
    }

    // 2) Docid (hash prefix)
    if is_hex_6(input) {
      let rows: Vec<DocRow> = self
        .db
        .query(format!(
          "SELECT collection, path, title, hash, modified_at, active FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND hash.starts_with($pfx) LIMIT 1"
        ))
        .bind(("pfx", input.to_string()))
        .await
        .map_err(db_err)?
        .take(0)
        .map_err(db_err)?;

      if let Some(d) = rows.into_iter().next() {
        return Ok(Some(
          self.build_doc_result(&d.collection, &d.path, &d.title, &d.hash, &d.modified_at, include_body).await?,
        ));
      }
    }

    // 3) Filesystem path -> (collection, relative)
    if let Some((collection, rel)) = self.resolve_fs_path(input).await? {
      let rows: Vec<DocRow> = self
        .db
        .query(format!(
          "SELECT collection, path, title, hash, modified_at, active FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND collection = $c AND path = $p LIMIT 1"
        ))
        .bind(("c", collection))
        .bind(("p", rel))
        .await
        .map_err(db_err)?
        .take(0)
        .map_err(db_err)?;

      if let Some(d) = rows.into_iter().next() {
        return Ok(Some(
          self.build_doc_result(&d.collection, &d.path, &d.title, &d.hash, &d.modified_at, include_body).await?,
        ));
      }
    }

    // 4) Treat as (already handelized) relative path across collections.
    let rows: Vec<DocRow> = self
      .db
      .query(format!(
        "SELECT collection, path, title, hash, modified_at, active FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND path = $p LIMIT 10"
      ))
      .bind(("p", input.to_string()))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    if let Some(d) = rows.into_iter().next() {
      return Ok(Some(
        self.build_doc_result(&d.collection, &d.path, &d.title, &d.hash, &d.modified_at, include_body).await?,
      ));
    }

    Ok(None)
  }

  async fn multi_get(&self, pattern: &str, options: &MultiGetOptions) -> Result<MultiGetResponse> {
    let include_body = options.include_body.unwrap_or(false);
    let max_bytes = options.max_bytes.unwrap_or(crate::DEFAULT_MULTI_GET_MAX_BYTES);

    let parts: Vec<&str> = pattern.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
      return Err(QmdError::InvalidArgument { message: "multi_get requires a non-empty pattern".to_string() });
    }

    // Build a corpus list once (virtual path + display path).
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct DocRow {
      collection: String,
      path:       String,
    }

    let docs: Vec<DocRow> = self
      .db
      .query(format!("SELECT collection, path FROM {QMD_DOCUMENTS_TABLE} WHERE active = true"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let mut corpus: Vec<(String, String)> = Vec::with_capacity(docs.len()); // (virtual, display)
    for d in docs {
      let vp = crate::build_virtual_path(&d.collection, &d.path);
      let dp = if d.path.is_empty() { d.collection.clone() } else { format!("{}/{}", d.collection, d.path) };
      corpus.push((vp, dp));
    }
    corpus.sort_by(|a, b| a.0.cmp(&b.0));

    let mut matched: Vec<String> = Vec::new(); // virtual paths
    let mut errors: Vec<String> = Vec::new();

    for p in parts {
      // Direct docid lookup.
      if is_hex_6(p) {
        if let Some(doc) = self.get_document(p, include_body).await? {
          matched.push(doc.filepath);
        } else {
          errors.push(format!("No document found for docid: {p}"));
        }
        continue;
      }

      let is_virtual = crate::is_virtual_path(p) || p.starts_with("qmd://");
      let pat_norm = if is_virtual { crate::normalize_virtual_path(p) } else { p.to_string() };

      let is_glob = pat_norm.contains('*') || pat_norm.contains('?') || pat_norm.contains('[');

      if !is_glob {
        // Exact doc
        if is_virtual {
          if let Some(doc) = self.get_document(&pat_norm, include_body).await? {
            matched.push(doc.filepath);
          } else {
            errors.push(format!("No document found: {p}"));
          }
        } else {
          // Try relative path exact across collections.
          if let Some(doc) = self.get_document(p, include_body).await? {
            matched.push(doc.filepath);
          } else {
            errors.push(format!("No document found: {p}"));
          }
        }
        continue;
      }

      let mut builder = globset::GlobSetBuilder::new();
      match globset::Glob::new(&pat_norm) {
        Ok(g) => {
          builder.add(g);
        }
        Err(e) => {
          errors.push(format!("Invalid glob pattern '{p}': {e}"));
          continue;
        }
      }
      let set = match builder.build() {
        Ok(s) => s,
        Err(e) => {
          errors.push(format!("Invalid glob set for '{p}': {e}"));
          continue;
        }
      };

      let mut any = false;
      for (vp, dp) in &corpus {
        let target = if is_virtual { vp } else { dp };
        if set.is_match(target) {
          matched.push(vp.clone());
          any = true;
        }
      }
      if !any {
        errors.push(format!("Pattern matched no documents: {p}"));
      }
    }

    matched.sort();
    matched.dedup();

    let mut results: Vec<MultiGetResult> = Vec::new();
    for vp in matched {
      let doc_opt = self.get_document(&vp, include_body).await?;
      let Some(doc) = doc_opt else {
        continue;
      };

      if include_body {
        let body_len = doc.body.as_ref().map(|b| b.as_bytes().len()).unwrap_or(0);
        if body_len > max_bytes {
          results.push(MultiGetResult::Skipped {
            doc:         crate::MultiGetDocRef { filepath: doc.filepath, display_path: doc.display_path },
            skipped:     true,
            skip_reason: format!("body too large ({body_len} bytes > {max_bytes} bytes)"),
          });
          continue;
        }
      }

      results.push(MultiGetResult::Found { doc, skipped: false });
    }

    Ok(MultiGetResponse { docs: results, errors })
  }

  async fn list_files(&self, target: Option<&str>) -> Result<Vec<String>> {
    let mut collection_filter: Option<String> = None;
    let mut prefix_filter: Option<String> = None;

    if let Some(t) = target.map(|s| s.trim()).filter(|s| !s.is_empty()) {
      if crate::is_virtual_path(t) || t.starts_with("qmd://") {
        if let Some(vp) = crate::parse_virtual_path(t) {
          collection_filter = Some(vp.collection_name);
          if !vp.path.is_empty() {
            prefix_filter = Some(vp.path);
          }
        }
      } else {
        // If it matches a collection name, treat it as such; otherwise treat as filesystem path prefix.
        let cols = self.list_collections().await?;
        if cols.iter().any(|c| c.name == t) {
          collection_filter = Some(t.to_string());
        } else if let Some((coll, rel)) = self.resolve_fs_path(t).await? {
          collection_filter = Some(coll);
          if !rel.is_empty() {
            prefix_filter = Some(rel);
          }
        }
      }
    }

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      collection: String,
      path:       String,
    }

    let mut sql = format!("SELECT collection, path FROM {QMD_DOCUMENTS_TABLE} WHERE active = true");
    if collection_filter.is_some() {
      sql.push_str(" AND collection = $c");
    }
    if prefix_filter.is_some() {
      sql.push_str(" AND path.starts_with($pfx)");
    }
    sql.push_str(" ORDER BY collection, path");

    let mut q = self.db.query(sql);
    if let Some(c) = &collection_filter {
      q = q.bind(("c", c.to_string()));
    }
    if let Some(pfx) = &prefix_filter {
      q = q.bind(("pfx", pfx.to_string()));
    }

    let rows: Vec<Row> = q.await.map_err(db_err)?.take(0).map_err(db_err)?;

    Ok(rows.into_iter().map(|r| crate::build_virtual_path(&r.collection, &r.path)).collect())
  }

  async fn get_cached_result(&self, cache_key: &str) -> Result<Option<String>> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct Row {
      result_json: Option<String>,
      result:      Option<String>,
    }

    let row: Option<Row> = self.db.select(llm_cache_rid(cache_key)).await.map_err(db_err)?;
    Ok(row.and_then(|r| r.result_json.or(r.result)))
  }

  async fn set_cached_result(&self, cache_key: &str, result: &str) -> Result<()> {
    #[derive(Debug, Clone, serde::Serialize, SurrealValue)]
    struct DbModel {
      cache_key:   String,
      result_json: String,
      created_at:  String,
    }

    let rid = llm_cache_rid(cache_key);
    let model = DbModel {
      cache_key:   cache_key.to_string(),
      result_json: result.to_string(),
      created_at:  Utc::now().to_rfc3339(),
    };

    let _: Option<serde_json::Value> = self
      .db
      .query("UPSERT $rid CONTENT $content;")
      .bind(("rid", rid))
      .bind(("content", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    Ok(())
  }

  async fn clear_cached_results(&self) -> Result<usize> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let before: Vec<CountRow> = self
      .db
      .query(format!("SELECT count() AS count FROM {QMD_LLM_CACHE_TABLE} GROUP ALL;"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let count = before.first().map(|r| r.count.max(0) as usize).unwrap_or(0);

    self.db.query(format!("DELETE FROM {QMD_LLM_CACHE_TABLE};")).await.map_err(db_err)?;

    Ok(count)
  }

  async fn get_status(&self) -> Result<IndexStatus> {
    let cols = self.list_collections().await?;

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let total_docs: Vec<CountRow> = self
      .db
      .query(format!("SELECT count() AS count FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP ALL"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let total_docs = total_docs.first().map(|r| r.count.max(0) as u64).unwrap_or(0);

    let health = self.get_index_health().await?;

    let vector_dim = self.embeddings_vector_index_dim_from_info().await?;
    if let Some(d) = vector_dim {
      self.embeddings_vector_index_dim.store(d, Ordering::Relaxed);
    }
    let has_vector_index = vector_dim.is_some();

    let mut collections: Vec<crate::CollectionInfo> = Vec::new();
    for c in cols {
      let active_docs: Vec<CountRow> = self
        .db
        .query(format!(
          "SELECT count() AS count FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND collection = $c GROUP ALL"
        ))
        .bind(("c", c.name.clone()))
        .await
        .map_err(db_err)?
        .take(0)
        .map_err(db_err)?;
      let active_docs = active_docs.first().map(|r| r.count.max(0) as u64).unwrap_or(0);

      // Prefer last modified doc timestamp; fall back to "now" for empty collections.
      #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
      struct ModifiedRow {
        modified_at: Option<String>,
      }
      let latest: Vec<ModifiedRow> = self
        .db
        .query(format!(
          "SELECT modified_at FROM {QMD_DOCUMENTS_TABLE} WHERE active = true AND collection = $c ORDER BY modified_at DESC LIMIT 1"
        ))
        .bind(("c", c.name.clone()))
        .await
        .map_err(db_err)?
        .take(0)
        .map_err(db_err)?;
      let last_updated = latest.first().and_then(|r| r.modified_at.clone()).unwrap_or_else(|| Utc::now().to_rfc3339());

      collections.push(crate::CollectionInfo {
        name: c.name.clone(),
        path: Some(c.collection.path),
        pattern: Some(c.collection.pattern),
        documents: active_docs,
        last_updated,
      });
    }

    Ok(IndexStatus {
      total_documents: total_docs,
      needs_embedding: health.needs_embedding,
      has_vector_index,
      collections,
    })
  }

  async fn get_index_health(&self) -> Result<IndexHealthInfo> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let total_docs: Vec<CountRow> = self
      .db
      .query(format!("SELECT count() AS count FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP ALL"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let total_docs = total_docs.first().map(|r| r.count.max(0) as u64).unwrap_or(0);

    // needs_embedding: active documents whose hash has zero embeddings for the current model
    let model = std::env::var("QMD_EMBED_MODEL").ok().unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL.to_string());

    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct HashRow {
      hash: String,
    }

    let doc_hashes: Vec<HashRow> = self
      .db
      .query(format!("SELECT hash FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP BY hash"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let doc_hashes: HashSet<String> = doc_hashes.into_iter().map(|r| r.hash).collect();

    let embedded_hashes: Vec<HashRow> = self
      .db
      .query(format!("SELECT hash FROM {QMD_EMBEDDINGS_TABLE} WHERE model = $m GROUP BY hash"))
      .bind(("m", model))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let embedded_hashes: HashSet<String> = embedded_hashes.into_iter().map(|r| r.hash).collect();

    let needs_embedding = doc_hashes.iter().filter(|h| !embedded_hashes.contains(*h)).count() as u64;

    // daysStale: days since most recent modified_at (best-effort)
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct ModifiedRow {
      modified_at: Option<String>,
    }

    let latest: Vec<ModifiedRow> = self
      .db
      .query(format!(
        "SELECT modified_at FROM {QMD_DOCUMENTS_TABLE} WHERE active = true ORDER BY modified_at DESC LIMIT 1"
      ))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;

    let days_stale =
      latest.first().and_then(|r| r.modified_at.clone()).and_then(|s| DateTime::parse_from_rfc3339(&s).ok()).map(
        |dt| {
          let dt = dt.with_timezone(&Utc);
          (Utc::now() - dt).num_days()
        },
      );

    Ok(IndexHealthInfo { needs_embedding, total_docs, days_stale })
  }

  async fn cleanup_orphaned_content(&self) -> Result<usize> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct HashRow {
      hash: String,
    }
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let active_hashes: Vec<HashRow> = self
      .db
      .query(format!("SELECT hash FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP BY hash"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let hashes: Vec<String> = active_hashes.into_iter().map(|r| r.hash).collect();

    let mut count_q = self.db.query(if hashes.is_empty() {
      format!("SELECT count() AS count FROM {QMD_CONTENTS_TABLE} GROUP ALL;")
    } else {
      format!("SELECT count() AS count FROM {QMD_CONTENTS_TABLE} WHERE hash NOT IN $hashes GROUP ALL;")
    });
    if !hashes.is_empty() {
      count_q = count_q.bind(("hashes", hashes.clone()));
    }
    let before: Vec<CountRow> = count_q.await.map_err(db_err)?.take(0).map_err(db_err)?;
    let count = before.first().map(|r| r.count.max(0) as usize).unwrap_or(0);
    if count == 0 {
      return Ok(0);
    }

    let mut del_q = self.db.query(if hashes.is_empty() {
      format!("DELETE FROM {QMD_CONTENTS_TABLE};")
    } else {
      format!("DELETE FROM {QMD_CONTENTS_TABLE} WHERE hash NOT IN $hashes;")
    });
    if !hashes.is_empty() {
      del_q = del_q.bind(("hashes", hashes));
    }
    del_q.await.map_err(db_err)?;

    Ok(count)
  }

  async fn cleanup_orphaned_vectors(&self) -> Result<usize> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct HashRow {
      hash: String,
    }
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let active_hashes: Vec<HashRow> = self
      .db
      .query(format!("SELECT hash FROM {QMD_DOCUMENTS_TABLE} WHERE active = true GROUP BY hash"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let hashes: Vec<String> = active_hashes.into_iter().map(|r| r.hash).collect();

    let mut count_q = self.db.query(if hashes.is_empty() {
      format!("SELECT count() AS count FROM {QMD_EMBEDDINGS_TABLE} GROUP ALL;")
    } else {
      format!("SELECT count() AS count FROM {QMD_EMBEDDINGS_TABLE} WHERE hash NOT IN $hashes GROUP ALL;")
    });
    if !hashes.is_empty() {
      count_q = count_q.bind(("hashes", hashes.clone()));
    }
    let before: Vec<CountRow> = count_q.await.map_err(db_err)?.take(0).map_err(db_err)?;
    let count = before.first().map(|r| r.count.max(0) as usize).unwrap_or(0);
    if count == 0 {
      return Ok(0);
    }

    let mut del_q = self.db.query(if hashes.is_empty() {
      format!("DELETE FROM {QMD_EMBEDDINGS_TABLE};")
    } else {
      format!("DELETE FROM {QMD_EMBEDDINGS_TABLE} WHERE hash NOT IN $hashes;")
    });
    if !hashes.is_empty() {
      del_q = del_q.bind(("hashes", hashes));
    }
    del_q.await.map_err(db_err)?;

    Ok(count)
  }

  async fn delete_inactive_docs(&self) -> Result<usize> {
    #[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
    struct CountRow {
      count: i64,
    }

    let before: Vec<CountRow> = self
      .db
      .query(format!("SELECT count() AS count FROM {QMD_DOCUMENTS_TABLE} WHERE active = false GROUP ALL;"))
      .await
      .map_err(db_err)?
      .take(0)
      .map_err(db_err)?;
    let count = before.first().map(|r| r.count.max(0) as usize).unwrap_or(0);
    if count == 0 {
      return Ok(0);
    }

    self.db.query(format!("DELETE FROM {QMD_DOCUMENTS_TABLE} WHERE active = false;")).await.map_err(db_err)?;
    Ok(count)
  }
}
