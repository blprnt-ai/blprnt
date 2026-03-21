use std::sync::Arc;

use qmd::EmbeddingResult;
use qmd::ExpandedQuery;
use qmd::LlmBackend;
use qmd::RerankDocumentResult;
use qmd::RerankResult;
use qmd::SurrealStorage;
use tempfile::TempDir;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn write_file(dir: &TempDir, rel: &str, content: &str) -> std::path::PathBuf {
  let path = dir.path().join(rel);
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).unwrap();
  }
  std::fs::write(&path, content).unwrap();
  path
}

async fn mem_db() -> persistence::prelude::DbConnection {
  use surrealdb::Surreal;
  use surrealdb::engine::local::Mem;

  let db = Surreal::new::<Mem>(()).await.unwrap();
  db.use_ns("app").use_db("main").await.unwrap();
  db
}

#[derive(Debug, Clone)]
struct FakeLlm;

#[async_trait::async_trait]
impl LlmBackend for FakeLlm {
  async fn embed(&self, text: &str, model: Option<&str>) -> qmd::Result<EmbeddingResult> {
    let model = model.unwrap_or("fake").to_string();
    let lower = text.to_lowercase();
    let rust = lower.matches("rust").count() as f32;
    let hello = lower.matches("hello").count() as f32;
    let len = (text.len() % 97) as f32;

    Ok(EmbeddingResult { embedding: vec![rust, hello, len], model })
  }

  async fn generate(&self, prompt: &str, model: Option<&str>) -> qmd::Result<qmd::GenerateResult> {
    Ok(qmd::GenerateResult {
      text:  prompt.chars().rev().collect(),
      model: model.unwrap_or("fake").to_string(),
      done:  true,
    })
  }

  async fn rerank(&self, query: &str, documents: &[String], model: Option<&str>) -> qmd::Result<RerankResult> {
    let q = query.to_lowercase();
    let mut results: Vec<RerankDocumentResult> = documents
      .iter()
      .enumerate()
      .map(|(idx, d)| {
        let score = d.to_lowercase().matches(&q).count() as f32;
        RerankDocumentResult { file: String::new(), score, index: idx }
      })
      .collect();
    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    Ok(RerankResult { results, model: model.unwrap_or("fake").to_string() })
  }

  async fn expand_query(&self, _query: &str, _intent: Option<&str>) -> qmd::Result<Vec<ExpandedQuery>> {
    Ok(Vec::new())
  }
}

#[tokio::test]
async fn pull_models_supports_local_paths() {
  let tmp = TempDir::new().unwrap();
  let p = write_file(&tmp, "tiny.gguf", "not a real model");

  let models = vec![p.to_string_lossy().to_string()];
  let results =
    qmd::pull_models(&models, qmd::PullModelsOptions { refresh: false, cache_dir: Some(tmp.path().join("cache")) })
      .await
      .unwrap();

  assert_eq!(results.len(), 1);
  assert!(results[0].size_bytes > 0);
  assert_eq!(results[0].path, p.to_string_lossy().to_string());
}

#[tokio::test]
async fn index_then_get_document_body() {
  let tmp = TempDir::new().unwrap();
  write_file(&tmp, "notes.md", "# Hello\n\nThis is a test about rust.\n\nSecond paragraph.\n");

  let db = mem_db().await;
  SurrealStorage::migrate(&db).await.unwrap();
  let storage = Arc::new(SurrealStorage::new(db));
  let store = qmd::create_store(qmd::StoreOptions { storage: storage.clone(), llm: None, config: None }).await.unwrap();

  store
    .add_collection(
      "notes",
      &qmd::AddCollectionOptions {
        path:    tmp.path().to_string_lossy().to_string(),
        pattern: Some("**/*.md".to_string()),
        ignore:  None,
      },
    )
    .await
    .unwrap();

  let _ = store.update(None).await.unwrap();

  let fts = store.search_lex("rust", None).await.unwrap();
  assert!(!fts.is_empty());

  let vp = qmd::build_virtual_path("notes", "notes.md");
  let doc = store.get(&vp, Some(&qmd::GetOptions { include_body: Some(true) })).await.unwrap().unwrap();

  assert_eq!(doc.collection_name, "notes");
  assert!(doc.title.to_lowercase().contains("hello"));
  assert!(doc.body.as_deref().unwrap_or_default().contains("rust"));
  assert!(!doc.hash.is_empty());
  assert!(!doc.docid.is_empty());
}

#[tokio::test]
async fn embed_updates_health_and_enables_vector_search() {
  let _guard = ENV_LOCK.lock().unwrap();
  let old = std::env::var("QMD_EMBED_MODEL").ok();
  unsafe { std::env::set_var("QMD_EMBED_MODEL", "test-embed-model") };

  let tmp = TempDir::new().unwrap();
  write_file(&tmp, "a.md", "# Rust\n\nRust is a systems programming language.\n");

  let db = mem_db().await;
  SurrealStorage::migrate(&db).await.unwrap();
  let storage = Arc::new(SurrealStorage::new(db));
  let store =
    qmd::create_store(qmd::StoreOptions { storage: storage.clone(), llm: Some(Arc::new(FakeLlm)), config: None })
      .await
      .unwrap();

  store
    .add_collection(
      "docs",
      &qmd::AddCollectionOptions {
        path:    tmp.path().to_string_lossy().to_string(),
        pattern: Some("**/*.md".to_string()),
        ignore:  None,
      },
    )
    .await
    .unwrap();
  let _ = store.update(None).await.unwrap();

  let before = store.get_index_health().await.unwrap();
  assert!(before.total_docs >= 1);
  assert!(before.needs_embedding >= 1);

  let _ = store.embed(None).await.unwrap();

  let after = store.get_index_health().await.unwrap();
  assert_eq!(after.needs_embedding, 0);

  let status = store.get_status().await.unwrap();
  assert!(status.has_vector_index);

  let hits = store
    .search_vector("rust", Some(&qmd::VectorSearchOptions { limit: Some(5), collection: Some("docs".to_string()) }))
    .await
    .unwrap();
  assert!(!hits.is_empty());

  match old {
    Some(v) => unsafe { std::env::set_var("QMD_EMBED_MODEL", v) },
    None => unsafe { std::env::remove_var("QMD_EMBED_MODEL") },
  }
}
