use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use globset::Glob;
use globset::GlobSetBuilder;
use globwalk::GlobWalkerBuilder;
use sha2::Digest;
use sha2::Sha256;

use crate::AddCollectionOptions;
use crate::Collection;
use crate::CollectionConfig;
use crate::CollectionListItem;
use crate::ContextItem;
use crate::DocumentNotFound;
use crate::DocumentResult;
use crate::EmbedOptions;
use crate::EmbedResult;
use crate::ExpandQueryOptions;
use crate::ExpandedQuery;
use crate::GetBodyOptions;
use crate::GetOptions;
use crate::HybridQueryResult;
use crate::IndexHealthInfo;
use crate::IndexStatus;
use crate::LexSearchOptions;
use crate::LlmBackend;
use crate::Maintenance;
use crate::MultiGetOptions;
use crate::MultiGetResponse;
use crate::NamedCollection;
use crate::QmdError;
use crate::Result;
use crate::SearchOptions;
use crate::SearchResult;
use crate::Storage;
use crate::StubStorage;
use crate::UpdateOptions;
use crate::UpdateResult;
use crate::VectorSearchOptions;
use crate::VirtualPath;

pub const DEFAULT_MULTI_GET_MAX_BYTES: usize = 10 * 1024;

const EMBEDDED_QMD_SKILL_MAIN: &str = r#"---
name: qmd
description: Search markdown knowledge bases, notes, and documentation with QMD.
license: MIT
compatibility: Requires qmd CLI or MCP server.
allowed-tools: Bash(qmd:*), mcp__qmd__*
---

# QMD - Quick Markdown Search

Local search engine for markdown content.

## Status

Run `qmd status` to inspect indexed collections and embedding health.

## MCP

Use the `qmd mcp` server when you want structured search, document retrieval, or status checks.
"#;

const EMBEDDED_QMD_SKILL_MCP_SETUP: &str = r#"# QMD MCP Server Setup

## Install

```bash
npm install -g @tobilu/qmd
qmd collection add ~/path/to/markdown --name myknowledge
qmd embed
```

## Client Config

Configure your MCP client to launch:

```json
{
  "mcpServers": {
    "qmd": { "command": "qmd", "args": ["mcp"] }
  }
}
```

You can also run `qmd mcp` directly when testing the server locally.
"#;

#[derive(Debug, Clone)]
pub struct CleanupResult {
  pub cleared_llm_cache:  usize,
  pub cleared_embeddings: bool,
}

#[derive(Clone)]
struct ExecutedSearchList {
  query_type: crate::QueryType,
  source:     crate::SearchSource,
  query:      String,
  results:    Vec<SearchResult>,
}

#[derive(Clone)]
pub struct StoreOptions {
  pub storage: Arc<dyn Storage>,
  pub llm:     Option<Arc<dyn LlmBackend>>,
  pub config:  Option<CollectionConfig>,
}

impl std::fmt::Debug for StoreOptions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("StoreOptions").finish_non_exhaustive()
  }
}

impl Default for StoreOptions {
  fn default() -> Self {
    Self { storage: Arc::new(StubStorage), llm: None, config: None }
  }
}

#[derive(Clone)]
pub struct QmdStore {
  storage: Arc<dyn Storage>,
  llm:     Option<Arc<dyn LlmBackend>>,
}

impl std::fmt::Debug for QmdStore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("QmdStore").finish_non_exhaustive()
  }
}

pub async fn create_store(options: StoreOptions) -> Result<QmdStore> {
  if let Some(cfg) = &options.config {
    options.storage.sync_config(cfg).await?;
  }
  Ok(QmdStore { storage: options.storage, llm: options.llm })
}

fn get_pwd() -> String {
  std::env::var("PWD").ok().unwrap_or_else(|| {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")).to_string_lossy().to_string()
  })
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

fn run_collection_update_command(cwd: &str, command: &str) -> Result<()> {
  let trimmed = command.trim();
  if trimmed.is_empty() {
    return Ok(());
  }

  #[cfg(target_os = "windows")]
  let output = Command::new("cmd").args(["/C", trimmed]).current_dir(cwd).output();

  #[cfg(not(target_os = "windows"))]
  let output = Command::new("bash").args(["-lc", trimmed]).current_dir(cwd).output();

  let output = output
    .map_err(|err| QmdError::Storage { message: format!("failed to run collection update command in {cwd}: {err}") })?;

  if output.status.success() {
    return Ok(());
  }

  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
  let detail = if !stderr.is_empty() {
    stderr
  } else if !stdout.is_empty() {
    stdout
  } else {
    format!("exit status {}", output.status)
  };

  Err(QmdError::Storage { message: format!("collection update command failed in {cwd}: {detail}") })
}

fn env_path(var: &str) -> Result<PathBuf> {
  std::env::var_os(var)
    .map(PathBuf::from)
    .ok_or_else(|| QmdError::InvalidArgument { message: format!("required environment variable is missing: {var}") })
}

fn skill_install_dir(global: bool) -> Result<PathBuf> {
  if global {
    Ok(env_path("HOME")?.join(".agents").join("skills").join("qmd"))
  } else {
    Ok(PathBuf::from(get_pwd()).join(".agents").join("skills").join("qmd"))
  }
}

fn claude_skill_link_dir(global: bool) -> Result<PathBuf> {
  if global {
    Ok(env_path("HOME")?.join(".claude").join("skills").join("qmd"))
  } else {
    Ok(PathBuf::from(get_pwd()).join(".claude").join("skills").join("qmd"))
  }
}

fn remove_path(path: &Path) -> Result<()> {
  let meta = std::fs::symlink_metadata(path)
    .map_err(|err| QmdError::Storage { message: format!("failed to inspect {}: {err}", path.display()) })?;

  if meta.file_type().is_symlink() || meta.is_file() {
    std::fs::remove_file(path)
      .map_err(|err| QmdError::Storage { message: format!("failed to remove {}: {err}", path.display()) })?;
  } else {
    std::fs::remove_dir_all(path)
      .map_err(|err| QmdError::Storage { message: format!("failed to remove {}: {err}", path.display()) })?;
  }

  Ok(())
}

fn write_embedded_skill(target_dir: &Path, force: bool) -> Result<()> {
  if target_dir.exists() {
    if !force {
      return Err(QmdError::InvalidArgument {
        message: format!("Skill already exists: {} (use force=true to replace it)", target_dir.display()),
      });
    }
    remove_path(target_dir)?;
  }

  std::fs::create_dir_all(target_dir.join("references")).map_err(|err| QmdError::Storage {
    message: format!("failed to create skill directory {}: {err}", target_dir.display()),
  })?;
  std::fs::write(target_dir.join("SKILL.md"), EMBEDDED_QMD_SKILL_MAIN)
    .map_err(|err| QmdError::Storage { message: format!("failed to write embedded skill: {err}") })?;
  std::fs::write(target_dir.join("references").join("mcp-setup.md"), EMBEDDED_QMD_SKILL_MCP_SETUP)
    .map_err(|err| QmdError::Storage { message: format!("failed to write embedded skill reference: {err}") })?;

  Ok(())
}

fn ensure_claude_skill_link(link_path: &Path, target_dir: &Path, force: bool) -> Result<()> {
  if let Some(parent) = link_path.parent() {
    std::fs::create_dir_all(parent)
      .map_err(|err| QmdError::Storage { message: format!("failed to create {}: {err}", parent.display()) })?;
  }

  if link_path.exists() || std::fs::symlink_metadata(link_path).is_ok() {
    if !force {
      return Err(QmdError::InvalidArgument {
        message: format!("Claude skill path already exists: {} (use force=true to replace it)", link_path.display()),
      });
    }
    remove_path(link_path)?;
  }

  #[cfg(target_os = "windows")]
  std::os::windows::fs::symlink_dir(target_dir, link_path).map_err(|err| QmdError::Storage {
    message: format!("failed to link {} -> {}: {err}", link_path.display(), target_dir.display()),
  })?;

  #[cfg(not(target_os = "windows"))]
  std::os::unix::fs::symlink(target_dir, link_path).map_err(|err| QmdError::Storage {
    message: format!("failed to link {} -> {}: {err}", link_path.display(), target_dir.display()),
  })?;

  Ok(())
}

fn slice_lines(text: &str, from_line: usize, max_lines: Option<usize>) -> String {
  if from_line == 0 {
    return String::new();
  }

  let start_idx = from_line.saturating_sub(1);
  let lines: Vec<&str> = text.lines().collect();
  if start_idx >= lines.len() {
    return String::new();
  }

  let end_idx = match max_lines {
    Some(n) => (start_idx + n).min(lines.len()),
    None => lines.len(),
  };

  lines[start_idx..end_idx].join("\n")
}

fn add_line_numbers(text: &str, start_line: u32) -> String {
  let mut out = String::new();
  for (idx, line) in text.lines().enumerate() {
    let n = start_line.saturating_add(idx as u32);
    out.push_str(&format!("{n}|{line}\n"));
  }
  out
}

fn now_iso() -> String {
  Utc::now().to_rfc3339()
}

fn system_time_to_iso(t: std::time::SystemTime) -> String {
  chrono::DateTime::<Utc>::from(t).to_rfc3339()
}

fn sha256_hex(content: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(content.as_bytes());
  hex::encode(hasher.finalize())
}

fn extract_title(content: &str, relative_file: &str) -> String {
  let mut in_fence = false;
  for line in content.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with("```") {
      in_fence = !in_fence;
      continue;
    }
    if in_fence {
      continue;
    }

    let t = trimmed.trim_start_matches('#').trim();
    if trimmed.starts_with('#') && !t.is_empty() {
      return t.to_string();
    }
  }

  Path::new(relative_file)
    .file_stem()
    .map(|s| s.to_string_lossy().to_string())
    .unwrap_or_else(|| relative_file.to_string())
}

fn build_context_index(contexts: Vec<ContextItem>) -> HashMap<String, Vec<ContextItem>> {
  let mut by_collection: HashMap<String, Vec<ContextItem>> = HashMap::new();
  for c in contexts {
    by_collection.entry(c.collection.clone()).or_default().push(c);
  }
  for items in by_collection.values_mut() {
    items.sort_by(|a, b| b.path.len().cmp(&a.path.len()));
  }
  by_collection
}

fn resolve_context(
  global_ctx: &Option<String>,
  by_collection: &HashMap<String, Vec<ContextItem>>,
  collection: &str,
  path: &str,
) -> Option<String> {
  let specific = by_collection
    .get(collection)
    .and_then(|items| {
      items.iter().find(|c| c.path.is_empty() || path == c.path || path.starts_with(&(c.path.clone() + "/")))
    })
    .map(|c| c.context.clone());

  match (global_ctx.clone(), specific) {
    (Some(g), Some(s)) => Some(format!("{g}\n\n{s}")),
    (Some(g), None) => Some(g),
    (None, Some(s)) => Some(s),
    (None, None) => None,
  }
}

#[derive(Debug, Clone)]
struct ChunkSlice {
  text: String,
  pos:  u64,
}

fn clamp_to_char_boundary(s: &str, mut idx: usize) -> usize {
  if idx > s.len() {
    idx = s.len();
  }
  while idx > 0 && !s.is_char_boundary(idx) {
    idx -= 1;
  }
  idx
}

fn chunk_document_smart(text: &str) -> Vec<ChunkSlice> {
  if text.is_empty() {
    return vec![ChunkSlice { text: String::new(), pos: 0 }];
  }

  let len = text.len();
  if len <= crate::CHUNK_SIZE_CHARS {
    return vec![ChunkSlice { text: text.to_string(), pos: 0 }];
  }

  let break_points = crate::scan_break_points(text);
  let fences = crate::find_code_fences(text);

  let mut chunks: Vec<ChunkSlice> = Vec::new();
  let mut start = 0usize;
  while start < len {
    let target = (start + crate::CHUNK_SIZE_CHARS).min(len);
    let mut end = if target >= len {
      len
    } else {
      crate::find_best_cutoff(&break_points, target, crate::CHUNK_WINDOW_CHARS, 0.7, &fences)
    };

    end = clamp_to_char_boundary(text, end);
    if end <= start {
      end = clamp_to_char_boundary(text, target);
    }

    let chunk_text = text[start..end].to_string();
    chunks.push(ChunkSlice { text: chunk_text, pos: start as u64 });

    if end >= len {
      break;
    }
    let next_start = end.saturating_sub(crate::CHUNK_OVERLAP_CHARS);
    start = clamp_to_char_boundary(text, next_start);
  }

  chunks
}

fn query_terms(query: &str) -> Vec<String> {
  let mut uniq: HashSet<String> = HashSet::new();
  for raw in query.split(|c: char| !c.is_alphanumeric()) {
    let t = raw.trim().to_ascii_lowercase();
    if t.len() >= 3 {
      uniq.insert(t);
    }
  }
  let mut out: Vec<String> = uniq.into_iter().collect();
  out.sort();
  out
}

fn best_chunk_for_query(query: &str, chunks: &[ChunkSlice]) -> (String, u64) {
  if chunks.is_empty() {
    return (String::new(), 0);
  }
  let terms = query_terms(query);
  if terms.is_empty() {
    let first = &chunks[0];
    return (first.text.clone(), first.pos);
  }

  let mut best_idx = 0usize;
  let mut best_score: usize = 0;
  for (idx, c) in chunks.iter().enumerate() {
    let hay = c.text.to_ascii_lowercase();
    let mut score = 0usize;
    for t in &terms {
      score += hay.matches(t).count();
    }
    if score > best_score {
      best_score = score;
      best_idx = idx;
    }
  }

  let best = &chunks[best_idx];
  (best.text.clone(), best.pos)
}

struct FusedDoc {
  doc:           DocumentResult,
  base_score:    f32,
  contributions: Vec<crate::RrfContributionTrace>,
  fts_scores:    Vec<f32>,
  vector_scores: Vec<f32>,
}

fn rrf_fuse(lists: &[ExecutedSearchList], k: f32, explain: bool) -> Vec<FusedDoc> {
  let mut fts_count = 0usize;
  let mut vec_count = 0usize;
  let mut fts_index_by_list: Vec<Option<usize>> = Vec::with_capacity(lists.len());
  let mut vec_index_by_list: Vec<Option<usize>> = Vec::with_capacity(lists.len());

  for list in lists {
    match list.source {
      crate::SearchSource::Fts => {
        fts_index_by_list.push(Some(fts_count));
        vec_index_by_list.push(None);
        fts_count += 1;
      }
      crate::SearchSource::Vec => {
        fts_index_by_list.push(None);
        vec_index_by_list.push(Some(vec_count));
        vec_count += 1;
      }
    }
  }

  let mut map: HashMap<String, FusedDoc> = HashMap::new();

  for (list_index, list) in lists.iter().enumerate() {
    for (rank_index, r) in list.results.iter().enumerate() {
      let rank = (rank_index + 1) as u32;
      let weight = 1.0f32;
      let rrf = weight / (k + rank as f32);

      let key = r.doc.filepath.clone();
      let entry = map.entry(key).or_insert_with(|| FusedDoc {
        doc:           r.doc.clone(),
        base_score:    0.0,
        contributions: Vec::new(),
        fts_scores:    vec![0.0; fts_count],
        vector_scores: vec![0.0; vec_count],
      });

      entry.base_score += rrf;

      if explain {
        entry.contributions.push(crate::RrfContributionTrace {
          list_index: list_index as u32,
          source: list.source,
          query_type: list.query_type,
          query: list.query.clone(),
          rank,
          weight,
          backend_score: r.score,
          rrf_contribution: rrf,
        });
      }

      if let Some(fi) = fts_index_by_list[list_index] {
        entry.fts_scores[fi] = r.score;
      }
      if let Some(vi) = vec_index_by_list[list_index] {
        entry.vector_scores[vi] = r.score;
      }
    }
  }

  let mut fused: Vec<FusedDoc> = map.into_values().collect();
  fused.sort_by(|a, b| b.base_score.total_cmp(&a.base_score));
  fused
}

async fn detect_collection_from_fs_path(storage: &dyn Storage, fs_path: &str) -> Result<Option<(String, String)>> {
  let real_path = crate::normalize_path_separators(&get_real_path(fs_path));
  let collections = storage.list_collections().await?;

  let mut best_match: Option<(String, String)> = None; // (name, collection_path)
  for coll in collections {
    let coll_path = crate::normalize_path_separators(&coll.collection.path);
    if real_path == coll_path || real_path.starts_with(&(coll_path.clone() + "/")) {
      match &best_match {
        Some((_n, best_path)) if best_path.len() >= coll_path.len() => {}
        _ => best_match = Some((coll.name, coll_path)),
      }
    }
  }

  let Some((name, coll_path)) = best_match else {
    return Ok(None);
  };

  let relative_path =
    if real_path == coll_path { String::new() } else { real_path[(coll_path.len() + 1)..].to_string() };

  Ok(Some((name, relative_path)))
}

#[derive(Debug, Clone)]
struct StructuredQuery {
  intent:   Option<String>,
  searches: Vec<ExpandedQuery>,
}

fn parse_structured_query(input: &str) -> Result<Option<StructuredQuery>> {
  let raw_lines: Vec<(String, String, u32)> = input
    .split('\n')
    .enumerate()
    .map(|(idx, raw)| (raw.to_string(), raw.trim().to_string(), (idx + 1) as u32))
    .filter(|(_raw, trimmed, _n)| !trimmed.is_empty())
    .collect();

  if raw_lines.is_empty() {
    return Ok(None);
  }

  let mut typed: Vec<ExpandedQuery> = Vec::new();
  let mut intent: Option<String> = None;

  for (_raw, trimmed, line_no) in &raw_lines {
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("expand:") {
      if raw_lines.len() > 1 {
        return Err(QmdError::InvalidArgument {
          message: format!(
            "Line {line_no} starts with expand:, but query documents cannot mix expand with typed lines. Submit a single expand query instead."
          ),
        });
      }
      let text = trimmed[7..].trim();
      if text.is_empty() {
        return Err(QmdError::InvalidArgument { message: "expand: query must include text.".to_string() });
      }
      return Ok(None);
    }

    if lower.starts_with("intent:") {
      if intent.is_some() {
        return Err(QmdError::InvalidArgument {
          message: format!("Line {line_no}: only one intent: line is allowed per query document."),
        });
      }
      let text = trimmed[7..].trim();
      if text.is_empty() {
        return Err(QmdError::InvalidArgument { message: format!("Line {line_no}: intent: must include text.") });
      }
      intent = Some(text.to_string());
      continue;
    }

    let (query_type, prefix_len) = if lower.starts_with("lex:") {
      (crate::ExpandedQueryType::Lex, 4usize)
    } else if lower.starts_with("vec:") {
      (crate::ExpandedQueryType::Vec, 4usize)
    } else if lower.starts_with("hyde:") {
      (crate::ExpandedQueryType::Hyde, 5usize)
    } else {
      if raw_lines.len() == 1 {
        // Single plain line -> implicit expand
        return Ok(None);
      }
      return Err(QmdError::InvalidArgument {
        message: format!(
          "Line {line_no} is missing a lex:/vec:/hyde:/intent: prefix. Each line in a query document must start with one."
        ),
      });
    };

    let text = trimmed[prefix_len..].trim();
    if text.is_empty() {
      let label = match query_type {
        crate::ExpandedQueryType::Lex => "lex:",
        crate::ExpandedQueryType::Vec => "vec:",
        crate::ExpandedQueryType::Hyde => "hyde:",
      };
      return Err(QmdError::InvalidArgument { message: format!("Line {line_no} ({label}) must include text.") });
    }
    if text.contains('\r') || text.contains('\n') {
      return Err(QmdError::InvalidArgument {
        message: format!("Line {line_no} contains a newline. Keep each query on a single line."),
      });
    }

    typed.push(ExpandedQuery { query_type, query: text.to_string(), line: Some(*line_no) });
  }

  if intent.is_some() && typed.is_empty() {
    return Err(QmdError::InvalidArgument {
      message: "intent: cannot appear alone. Add at least one lex:, vec:, or hyde: line.".to_string(),
    });
  }

  Ok(if typed.is_empty() { None } else { Some(StructuredQuery { intent, searches: typed }) })
}

impl QmdStore {
  // ── Search ──────────────────────────────────────────────────────────

  pub async fn search(&self, options: &SearchOptions) -> Result<Vec<HybridQueryResult>> {
    if options.query.is_none() && options.queries.as_ref().map(|q| q.is_empty()).unwrap_or(true) {
      return Err(QmdError::InvalidArgument {
        message: "search() requires either 'query' or non-empty 'queries'".to_string(),
      });
    }
    // Full hybrid orchestration is implemented via CLI-parity `cmd_query` / structured search.
    // Here we route:
    // - queries => structured search (no expansion)
    // - query   => hybrid search with optional expansion/rerank
    if let Some(searches) = &options.queries {
      return self.structured_search(searches, options).await;
    }
    let query = options.query.as_deref().unwrap_or_default();
    self.hybrid_search(query, options).await
  }

  pub async fn search_lex(&self, _query: &str, _options: Option<&LexSearchOptions>) -> Result<Vec<SearchResult>> {
    let mut opts = _options.cloned().unwrap_or_default();
    if opts.limit.is_none() {
      opts.limit = Some(10);
    }
    self.storage.search_lex(_query, &opts).await
  }

  pub async fn search_vector(&self, _query: &str, _options: Option<&VectorSearchOptions>) -> Result<Vec<SearchResult>> {
    let llm = self.llm.as_ref().ok_or_else(|| QmdError::InvalidArgument {
      message: "search_vector requires an LLM backend for embeddings".to_string(),
    })?;

    let mut opts = _options.cloned().unwrap_or_default();
    if opts.limit.is_none() {
      opts.limit = Some(10);
    }

    let model = std::env::var("QMD_EMBED_MODEL").ok().unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL.to_string());
    let formatted = crate::format_query_for_embedding(_query, Some(&model));
    let embedding = llm.embed(&formatted, Some(&model)).await?;

    self.storage.search_vector(&embedding.embedding, &embedding.model, &opts).await
  }

  pub async fn expand_query(&self, query: &str, options: Option<&ExpandQueryOptions>) -> Result<Vec<ExpandedQuery>> {
    let llm = self
      .llm
      .as_ref()
      .ok_or_else(|| QmdError::InvalidArgument { message: "expand_query requires an LLM backend".to_string() })?;
    llm.expand_query(query, options.and_then(|o| o.intent.as_deref())).await
  }

  // ── Document retrieval ──────────────────────────────────────────────

  pub async fn get(
    &self,
    _path_or_docid: &str,
    _options: Option<&GetOptions>,
  ) -> Result<std::result::Result<DocumentResult, DocumentNotFound>> {
    let include_body = _options.and_then(|o| o.include_body).unwrap_or(false);
    let doc = self.storage.get_document(_path_or_docid, include_body).await?;
    match doc {
      Some(d) => Ok(Ok(d)),
      None => Ok(Err(DocumentNotFound {
        error:         "not_found".to_string(),
        query:         _path_or_docid.to_string(),
        similar_files: Vec::new(),
      })),
    }
  }

  pub async fn get_document_body(
    &self,
    _path_or_docid: &str,
    _options: Option<&GetBodyOptions>,
  ) -> Result<Option<String>> {
    let doc = self.storage.get_document(_path_or_docid, true).await?;
    let Some(doc) = doc else {
      return Ok(None);
    };

    let mut body = if let Some(b) = doc.body.clone() {
      b
    } else {
      match self.storage.get_content(&doc.hash).await? {
        Some(b) => b,
        None => return Ok(None),
      }
    };

    if let Some(opts) = _options {
      let from_line = opts.from_line.unwrap_or(1).max(1) as usize;
      let max_lines = opts.max_lines.map(|n| n.max(0) as usize);
      body = slice_lines(&body, from_line, max_lines);
    }

    Ok(Some(body))
  }

  pub async fn multi_get(&self, _pattern: &str, _options: Option<&MultiGetOptions>) -> Result<MultiGetResponse> {
    let opts = _options.cloned().unwrap_or_default();
    self.storage.multi_get(_pattern, &opts).await
  }

  // ── Collection management ───────────────────────────────────────────

  pub async fn add_collection(&self, name: &str, opts: &AddCollectionOptions) -> Result<()> {
    let collection = Collection {
      path:               opts.path.clone(),
      pattern:            opts.pattern.clone().unwrap_or_else(|| "**/*.md".to_string()),
      ignore:             opts.ignore.clone(),
      context:            None,
      update:             None,
      include_by_default: true,
    };
    self.storage.upsert_collection(name, &collection).await
  }

  pub async fn remove_collection(&self, name: &str) -> Result<bool> {
    self.storage.delete_collection(name).await
  }

  pub async fn rename_collection(&self, old_name: &str, new_name: &str) -> Result<bool> {
    self.storage.rename_collection(old_name, new_name).await
  }

  pub async fn list_collections(&self) -> Result<Vec<CollectionListItem>> {
    self.storage.list_collections_info().await
  }

  pub async fn get_default_collection_names(&self) -> Result<Vec<String>> {
    let cols = self.storage.list_collections_info().await?;
    Ok(cols.into_iter().filter(|c| c.include_by_default).map(|c| c.name).collect())
  }

  // ── Context management ──────────────────────────────────────────────

  pub async fn add_context(&self, collection_name: &str, path_prefix: &str, context_text: &str) -> Result<bool> {
    self.storage.upsert_context(collection_name, path_prefix, context_text).await
  }

  pub async fn remove_context(&self, collection_name: &str, path_prefix: &str) -> Result<bool> {
    self.storage.remove_context(collection_name, path_prefix).await
  }

  pub async fn set_global_context(&self, context: Option<&str>) -> Result<()> {
    self.storage.set_global_context(context).await
  }

  pub async fn get_global_context(&self) -> Result<Option<String>> {
    self.storage.get_global_context().await
  }

  pub async fn list_contexts(&self) -> Result<Vec<ContextItem>> {
    self.storage.list_contexts().await
  }

  // ── Indexing ────────────────────────────────────────────────────────

  pub async fn update(&self, _options: Option<&UpdateOptions>) -> Result<UpdateResult> {
    self.reindex_collections(_options).await
  }

  pub async fn embed(&self, _options: Option<&EmbedOptions>) -> Result<EmbedResult> {
    self.generate_embeddings(_options).await
  }

  // ── Index health ────────────────────────────────────────────────────

  pub async fn get_status(&self) -> Result<IndexStatus> {
    self.storage.get_status().await
  }

  pub async fn get_index_health(&self) -> Result<IndexHealthInfo> {
    self.storage.get_index_health().await
  }

  // ── Lifecycle ───────────────────────────────────────────────────────

  pub async fn close(&self) -> Result<()> {
    Ok(())
  }

  // ── Internal implementations ────────────────────────────────────────

  async fn hybrid_search(&self, query: &str, options: &SearchOptions) -> Result<Vec<HybridQueryResult>> {
    let query = query.trim();
    if query.is_empty() {
      return Ok(Vec::new());
    }

    let mut searches: Vec<ExpandedQuery> = Vec::new();
    searches.push(ExpandedQuery {
      query_type: crate::ExpandedQueryType::Lex,
      query:      query.to_string(),
      line:       None,
    });

    if self.llm.is_some() {
      searches.push(ExpandedQuery {
        query_type: crate::ExpandedQueryType::Vec,
        query:      query.to_string(),
        line:       None,
      });

      match self.expand_query(query, Some(&ExpandQueryOptions { intent: options.intent.clone() })).await {
        Ok(expanded) => searches.extend(expanded),
        Err(QmdError::NotImplemented { .. }) => {}
        Err(e) => return Err(e),
      }
    }

    // Dedupe to avoid redundant DB / LLM work.
    let mut seen: HashSet<(crate::ExpandedQueryType, String)> = HashSet::new();
    searches.retain(|q| seen.insert((q.query_type, q.query.clone())));

    self.structured_search(&searches, options).await
  }

  async fn structured_search(
    &self,
    searches: &[ExpandedQuery],
    options: &SearchOptions,
  ) -> Result<Vec<HybridQueryResult>> {
    let explain = options.explain.unwrap_or(false);
    let limit = options.limit.unwrap_or(10).max(1) as usize;
    let min_backend_score = options.min_score.unwrap_or(0.0);

    // Storage supports only one explicit collection parameter; for multi-collection
    // requests we search across all, then filter.
    let single_collection = options
      .collection
      .clone()
      .or_else(|| options.collections.clone().and_then(|c| if c.len() == 1 { Some(c[0].clone()) } else { None }));

    let collection_filter = options.collections.clone();

    let fetch_limit: usize = (limit * 5).max(40);
    let mut lists: Vec<ExecutedSearchList> = Vec::new();

    // Execute each typed query.
    for q in searches {
      match q.query_type {
        crate::ExpandedQueryType::Lex => {
          let mut lex_opts = LexSearchOptions::default();
          lex_opts.limit = Some(fetch_limit);
          lex_opts.collection = single_collection.clone();
          let mut results = self.storage.search_lex(&q.query, &lex_opts).await?;
          results.retain(|r| r.score >= min_backend_score);
          if let Some(cols) = &collection_filter {
            results.retain(|r| cols.contains(&r.doc.collection_name));
          }
          if !results.is_empty() {
            lists.push(ExecutedSearchList {
              query_type: if options.query.as_deref() == Some(q.query.as_str()) {
                crate::QueryType::Original
              } else {
                crate::QueryType::Lex
              },
              source: crate::SearchSource::Fts,
              query: q.query.clone(),
              results,
            });
          }
        }
        crate::ExpandedQueryType::Vec | crate::ExpandedQueryType::Hyde => {
          let llm = self.llm.as_ref().ok_or_else(|| QmdError::InvalidArgument {
            message: "structured search contains semantic queries, but no LLM backend was provided".to_string(),
          })?;

          let model = std::env::var("QMD_EMBED_MODEL").ok().unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL.to_string());
          let formatted = match q.query_type {
            crate::ExpandedQueryType::Hyde => crate::format_doc_for_embedding(&q.query, None, Some(&model)),
            crate::ExpandedQueryType::Vec => crate::format_query_for_embedding(&q.query, Some(&model)),
            _ => unreachable!(),
          };
          let embedding = llm.embed(&formatted, Some(&model)).await?;

          let mut vec_opts = VectorSearchOptions::default();
          vec_opts.limit = Some(fetch_limit);
          vec_opts.collection = single_collection.clone();

          let mut results = self.storage.search_vector(&embedding.embedding, &embedding.model, &vec_opts).await?;
          results.retain(|r| r.score >= min_backend_score);
          if let Some(cols) = &collection_filter {
            results.retain(|r| cols.contains(&r.doc.collection_name));
          }
          if !results.is_empty() {
            lists.push(ExecutedSearchList {
              query_type: if options.query.as_deref() == Some(q.query.as_str()) {
                crate::QueryType::Original
              } else if q.query_type == crate::ExpandedQueryType::Hyde {
                crate::QueryType::Hyde
              } else {
                crate::QueryType::Vec
              },
              source: crate::SearchSource::Vec,
              query: q.query.clone(),
              results,
            });
          }
        }
      }
    }

    if lists.is_empty() {
      return Ok(Vec::new());
    }

    // Preload contexts for result enrichment.
    let global_ctx = self.storage.get_global_context().await?;
    let contexts = self.storage.list_contexts().await?;
    let ctx_by_collection = build_context_index(contexts);

    // Fuse lists (RRF) and rank.
    let rrf_k: f32 = 60.0;
    let fused = rrf_fuse(&lists, rrf_k, explain);

    // Select candidates for reranking and final output.
    let candidate_limit = limit.max(10) * 4;
    let mut candidates = fused;
    candidates.truncate(candidate_limit);

    // Enrich with bodies, best chunks, and optional rerank.
    let rerank_enabled = options.rerank.unwrap_or(true);
    let rerank_query =
      options.query.clone().unwrap_or_else(|| searches.iter().map(|q| q.query.clone()).collect::<Vec<_>>().join("\n"));

    #[derive(Clone)]
    struct CandidateEnriched {
      doc:            DocumentResult,
      body:           String,
      best_chunk:     String,
      best_chunk_pos: u64,
      base_score:     f32,
      explain:        Option<crate::HybridQueryExplain>,
    }

    let mut enriched: Vec<CandidateEnriched> = Vec::with_capacity(candidates.len());
    for (rank_idx, fused) in candidates.into_iter().enumerate() {
      let mut doc = fused.doc.clone();

      let body = match doc.body.clone() {
        Some(b) => b,
        None => self.storage.get_content(&doc.hash).await?.unwrap_or_default(),
      };

      let chunks = chunk_document_smart(&body);
      let (best_chunk, best_chunk_pos) = best_chunk_for_query(&rerank_query, &chunks);

      if doc.context.is_none() {
        if let Some(vp) = crate::parse_virtual_path(&doc.filepath) {
          doc.context = resolve_context(&global_ctx, &ctx_by_collection, &vp.collection_name, &vp.path);
        }
      }

      let explain_obj = if explain {
        let rank = (rank_idx + 1) as u32;
        Some(crate::HybridQueryExplain {
          fts_scores:    fused.fts_scores.clone(),
          vector_scores: fused.vector_scores.clone(),
          rrf:           crate::RrfExplain {
            rank,
            position_score: fused.base_score,
            weight: 1.0,
            base_score: fused.base_score,
            top_rank_bonus: 0.0,
            total_score: fused.base_score,
            contributions: fused.contributions.clone(),
          },
          rerank_score:  0.0,
          blended_score: fused.base_score,
        })
      } else {
        None
      };

      enriched.push(CandidateEnriched {
        doc,
        body,
        best_chunk,
        best_chunk_pos,
        base_score: fused.base_score,
        explain: explain_obj,
      });
    }

    // Optional rerank: use best chunks.
    if rerank_enabled {
      if let Some(llm) = self.llm.as_ref() {
        let texts: Vec<String> = enriched.iter().map(|c| c.best_chunk.clone()).collect();
        if !texts.is_empty() {
          let rerank = match llm.rerank(&rerank_query, &texts, None).await {
            Ok(r) => Some(r),
            Err(QmdError::NotImplemented { .. }) => None,
            Err(e) => return Err(e),
          };
          let Some(rerank) = rerank else {
            // Graceful degradation when a consumer provides an LLM backend
            // that doesn't implement reranking yet.
            return Ok(
              enriched
                .into_iter()
                .take(limit)
                .map(|c| HybridQueryResult {
                  file:           c.doc.filepath,
                  display_path:   c.doc.display_path,
                  title:          c.doc.title,
                  body:           c.body,
                  best_chunk:     c.best_chunk,
                  best_chunk_pos: c.best_chunk_pos,
                  score:          c.base_score,
                  context:        c.doc.context,
                  docid:          c.doc.docid,
                  explain:        c.explain,
                })
                .collect(),
            );
          };
          let mut rerank_by_index: HashMap<usize, f32> = HashMap::new();
          for r in rerank.results {
            rerank_by_index.insert(r.index, r.score);
          }

          for (idx, c) in enriched.iter_mut().enumerate() {
            let rr = rerank_by_index.get(&idx).copied().unwrap_or(0.0);
            let blended = (c.base_score * 0.6) + (rr * 0.4);
            if let Some(ex) = c.explain.as_mut() {
              ex.rerank_score = rr;
              ex.blended_score = blended;
            }
            c.base_score = blended;
          }
        }
      }
    }

    // Final ranking and truncation to requested limit.
    enriched.sort_by(|a, b| b.base_score.total_cmp(&a.base_score));
    enriched.truncate(limit);

    Ok(
      enriched
        .into_iter()
        .map(|c| HybridQueryResult {
          file:           c.doc.filepath,
          display_path:   c.doc.display_path,
          title:          c.doc.title,
          body:           c.body,
          best_chunk:     c.best_chunk,
          best_chunk_pos: c.best_chunk_pos,
          score:          c.base_score,
          context:        c.doc.context,
          docid:          c.doc.docid,
          explain:        c.explain,
        })
        .collect(),
    )
  }

  async fn reindex_collections(&self, options: Option<&UpdateOptions>) -> Result<UpdateResult> {
    let opts = options.cloned().unwrap_or_default();

    let mut collections = self.storage.list_collections().await?;
    if let Some(filter) = &opts.collections {
      collections.retain(|c| filter.contains(&c.name));
    }

    let collections_count = collections.len();
    let mut indexed: usize = 0;
    let mut updated: usize = 0;
    let mut unchanged: usize = 0;
    let mut removed: usize = 0;

    for named in collections {
      let coll_name = named.name.clone();
      let coll_path = expand_home(&named.collection.path);
      let glob_pattern = if named.collection.pattern.trim().is_empty() {
        "**/*.md".to_string()
      } else {
        named.collection.pattern.clone()
      };
      let now = now_iso();

      if let Some(update_cmd) = named.collection.update.as_deref() {
        run_collection_update_command(&coll_path, update_cmd)?;
      }

      let mut ignore_builder = GlobSetBuilder::new();
      // Default excludes (same as TS)
      for d in ["node_modules", ".git", ".cache", "vendor", "dist", "build"] {
        ignore_builder.add(
          Glob::new(&format!("**/{d}/**"))
            .map_err(|e| QmdError::InvalidArgument { message: format!("invalid ignore glob for {d}: {e}") })?,
        );
      }
      if let Some(ignore) = &named.collection.ignore {
        for pat in ignore {
          ignore_builder.add(
            Glob::new(pat)
              .map_err(|e| QmdError::InvalidArgument { message: format!("invalid ignore glob \"{pat}\": {e}") })?,
          );
        }
      }
      let ignore_set = ignore_builder
        .build()
        .map_err(|e| QmdError::InvalidArgument { message: format!("failed to build ignore set: {e}") })?;

      let walker = GlobWalkerBuilder::from_patterns(coll_path.as_str(), &[glob_pattern.as_str()])
        .follow_links(false)
        .build()
        .map_err(|e| QmdError::InvalidArgument {
          message: format!("globwalk build failed for collection {coll_name}: {e}"),
        })?;

      let mut files: Vec<(String, String)> = Vec::new(); // (relative, absolute)
      for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let abs = entry.path().to_path_buf();
        let rel = match abs.strip_prefix(coll_path.as_str()) {
          Ok(p) => p,
          Err(_) => continue,
        };
        let rel = rel.to_string_lossy().to_string();
        let rel = crate::normalize_path_separators(&rel);

        if rel.is_empty() {
          continue;
        }
        if rel.split('/').any(|part| part.starts_with('.')) {
          continue;
        }
        if ignore_set.is_match(&rel) {
          continue;
        }

        files.push((rel, get_real_path(abs.to_string_lossy().as_ref())));
      }

      let total = files.len();
      let mut processed: usize = 0;
      let mut seen_paths: HashSet<String> = HashSet::new();

      for (relative_file, absolute_file) in files {
        processed += 1;
        if let Some(cb) = &opts.on_progress {
          cb(crate::UpdateProgress {
            collection: coll_name.clone(),
            file: relative_file.clone(),
            current: processed,
            total,
          });
        }

        let content = match std::fs::read_to_string(&absolute_file) {
          Ok(c) => c,
          Err(_) => continue,
        };
        if content.trim().is_empty() {
          continue;
        }

        let hash = sha256_hex(&content);
        let title = extract_title(&content, &relative_file);
        let path = crate::handelize(&relative_file)?;
        seen_paths.insert(path.clone());

        let existing = self.storage.find_active_document(&coll_name, &path).await?;
        match existing {
          Some((existing_hash, existing_title)) => {
            if existing_hash == hash {
              if existing_title != title {
                self.storage.update_document_title(&coll_name, &path, &title, &now).await?;
                updated += 1;
              } else {
                unchanged += 1;
              }
            } else {
              self.storage.upsert_content(&hash, &content, &now).await?;
              let modified_at = std::fs::metadata(&absolute_file)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(system_time_to_iso)
                .unwrap_or_else(|| now.clone());
              self.storage.update_document_hash(&coll_name, &path, &title, &hash, &modified_at).await?;
              updated += 1;
            }
          }
          None => {
            indexed += 1;
            self.storage.upsert_content(&hash, &content, &now).await?;
            let (created_at, modified_at) = std::fs::metadata(&absolute_file)
              .ok()
              .map(|m| {
                let created = m.created().ok().map(system_time_to_iso).unwrap_or_else(|| now.clone());
                let modified = m.modified().ok().map(system_time_to_iso).unwrap_or_else(|| now.clone());
                (created, modified)
              })
              .unwrap_or_else(|| (now.clone(), now.clone()));

            self.storage.upsert_document(&coll_name, &path, &title, &hash, &created_at, &modified_at, true).await?;
          }
        }
      }

      // Deactivate documents that no longer exist
      let active_paths = self.storage.list_active_document_paths(&coll_name).await?;
      for p in active_paths {
        if !seen_paths.contains(&p) {
          self.storage.deactivate_document(&coll_name, &p).await?;
          removed += 1;
        }
      }
    }

    Ok(UpdateResult {
      collections: collections_count,
      indexed,
      updated,
      unchanged,
      removed,
      needs_embedding: self.storage.get_index_health().await?.needs_embedding as usize,
    })
  }

  async fn generate_embeddings(&self, options: Option<&EmbedOptions>) -> Result<EmbedResult> {
    let llm = self.llm.as_ref().ok_or_else(|| QmdError::InvalidArgument {
      message: "embed requires an LLM backend for embeddings".to_string(),
    })?;

    let started = Instant::now();
    let opts = options.cloned().unwrap_or_default();

    let model = opts
      .model
      .clone()
      .or_else(|| std::env::var("QMD_EMBED_MODEL").ok())
      .unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL.to_string());

    if opts.force.unwrap_or(false) {
      self.storage.clear_all_embeddings().await?;
    }

    let now = now_iso();
    let docs_to_embed = self.storage.get_hashes_for_embedding().await?;

    if docs_to_embed.is_empty() {
      return Ok(EmbedResult { docs_processed: 0, chunks_embedded: 0, errors: 0, duration_ms: 0 });
    }

    let total_bytes: u64 = docs_to_embed.iter().map(|d| d.body.as_bytes().len() as u64).sum();

    let mut docs_processed: usize = 0;
    let mut chunks_embedded: usize = 0;
    let mut errors: usize = 0;
    let mut bytes_processed: u64 = 0;
    let mut total_chunks: usize = 0;

    for doc in docs_to_embed {
      docs_processed += 1;

      if doc.body.trim().is_empty() {
        bytes_processed += doc.body.as_bytes().len() as u64;
        continue;
      }

      let title = extract_title(&doc.body, &doc.path);
      let chunks = chunk_document_smart(&doc.body);
      total_chunks += chunks.len();

      for (seq, chunk) in chunks.into_iter().enumerate() {
        let formatted = crate::format_doc_for_embedding(&chunk.text, Some(&title), Some(&model));
        match llm.embed(&formatted, Some(&model)).await {
          Ok(emb) => {
            let pos_u32: u32 = chunk.pos.min(u32::MAX as u64) as u32;
            let record = crate::EmbeddingModel {
              hash:        doc.hash.clone(),
              seq:         seq as u32,
              pos:         pos_u32,
              embedding:   emb.embedding,
              model:       emb.model,
              embedded_at: now.clone(),
            };
            if self.storage.create_embedding(record).await.is_ok() {
              chunks_embedded += 1;
            } else {
              errors += 1;
            }
          }
          Err(_) => {
            errors += 1;
          }
        }

        let chunk_bytes = chunk.text.as_bytes().len() as u64;
        bytes_processed += chunk_bytes;

        if let Some(cb) = &opts.on_progress {
          cb(crate::EmbedProgress {
            chunks_embedded,
            total_chunks,
            bytes_processed: bytes_processed.min(total_bytes),
            total_bytes,
            errors,
          });
        }
      }

      if let Some(cb) = &opts.on_progress {
        cb(crate::EmbedProgress {
          chunks_embedded,
          total_chunks,
          bytes_processed: bytes_processed.min(total_bytes),
          total_bytes,
          errors,
        });
      }
    }

    Ok(EmbedResult { docs_processed, chunks_embedded, errors, duration_ms: started.elapsed().as_millis() as u64 })
  }

  // =============================================================================
  // CLI parity methods (library-only; no bin)
  // =============================================================================

  pub fn maintenance(&self) -> Maintenance {
    Maintenance::new(self.storage.clone())
  }

  pub async fn cmd_query(
    &self,
    query_input: &str,
    opts: Option<&crate::SearchOptions>,
  ) -> Result<Vec<HybridQueryResult>> {
    let parsed = parse_structured_query(query_input)?;

    if let Some(parsed) = parsed {
      let mut options = SearchOptions::default();
      options.queries = Some(parsed.searches);
      options.intent = opts.and_then(|o| o.intent.clone()).or(parsed.intent);
      options.rerank = opts.and_then(|o| o.rerank);
      options.collection = opts.and_then(|o| o.collection.clone());
      options.collections = opts.and_then(|o| o.collections.clone());
      options.limit = opts.and_then(|o| o.limit);
      options.min_score = opts.and_then(|o| o.min_score);
      options.explain = opts.and_then(|o| o.explain);
      return self.search(&options).await;
    }

    // Plain query string (including explicit expand: prefix)
    let trimmed = query_input.trim();
    let query = if trimmed.to_ascii_lowercase().starts_with("expand:") { trimmed[7..].trim() } else { trimmed };
    let mut options = opts.cloned().unwrap_or_default();
    options.query = Some(query.to_string());
    self.search(&options).await
  }

  pub async fn cmd_search(&self, query: &str, opts: Option<&LexSearchOptions>) -> Result<Vec<SearchResult>> {
    self.search_lex(query, opts).await
  }

  pub async fn cmd_vsearch(&self, query: &str, opts: Option<&VectorSearchOptions>) -> Result<Vec<SearchResult>> {
    // CLI default min-score for vector search is 0.3, but our VectorSearchOptions
    // does not include minScore; that filtering is expected in the caller.
    let _ = opts;
    self.search_vector(query, None).await
  }

  pub async fn cmd_vector_search(&self, query: &str, opts: Option<&VectorSearchOptions>) -> Result<Vec<SearchResult>> {
    self.cmd_vsearch(query, opts).await
  }

  pub async fn cmd_deep_search(
    &self,
    query_input: &str,
    opts: Option<&crate::SearchOptions>,
  ) -> Result<Vec<HybridQueryResult>> {
    self.cmd_query(query_input, opts).await
  }

  pub async fn cmd_status(&self) -> Result<IndexStatus> {
    self.get_status().await
  }

  pub async fn cmd_update(&self, opts: Option<&UpdateOptions>) -> Result<UpdateResult> {
    self.update(opts).await
  }

  pub async fn cmd_embed(&self, opts: Option<&EmbedOptions>) -> Result<EmbedResult> {
    self.embed(opts).await
  }

  pub async fn cmd_cleanup(&self) -> Result<CleanupResult> {
    let maintenance = self.maintenance();
    let cleared_llm_cache = maintenance.clear_llm_cache().await?;
    maintenance.clear_embeddings().await?;
    Ok(CleanupResult { cleared_llm_cache, cleared_embeddings: true })
  }

  // ── context ─────────────────────────────────────────────────────────

  pub async fn cmd_context_add(&self, path_arg: Option<&str>, context_text: &str) -> Result<bool> {
    if path_arg == Some("/") {
      self.set_global_context(Some(context_text)).await?;
      return Ok(true);
    }

    let pwd = get_pwd();
    let mut fs_path = path_arg.unwrap_or(".");
    if fs_path == "." || fs_path == "./" {
      fs_path = pwd.as_str();
    }

    let fs_path = expand_home(fs_path);
    if crate::is_virtual_path(&fs_path) {
      let parsed: VirtualPath = crate::parse_virtual_path(&fs_path)
        .ok_or_else(|| QmdError::InvalidArgument { message: format!("Invalid virtual path: {fs_path}") })?;

      // Ensure collection exists
      let cols = self.storage.list_collections().await?;
      if !cols.iter().any(|c| c.name == parsed.collection_name) {
        return Err(QmdError::InvalidArgument { message: format!("Collection not found: {}", parsed.collection_name) });
      }

      return self.add_context(&parsed.collection_name, &parsed.path, context_text).await;
    }

    let fs_path =
      if crate::is_absolute_path(&fs_path) { fs_path } else { crate::resolve(&[pwd.as_str(), fs_path.as_str()])? };

    let detected = detect_collection_from_fs_path(self.storage.as_ref(), &fs_path).await?;
    let Some((collection_name, relative_path)) = detected else {
      return Err(QmdError::InvalidArgument { message: format!("Path is not in any indexed collection: {fs_path}") });
    };

    self.add_context(&collection_name, &relative_path, context_text).await
  }

  pub async fn cmd_context_list(&self) -> Result<Vec<ContextItem>> {
    self.list_contexts().await
  }

  pub async fn cmd_context_rm(&self, path_arg: &str) -> Result<bool> {
    if path_arg == "/" {
      self.set_global_context(None).await?;
      return Ok(true);
    }

    let fs_path = expand_home(path_arg);
    if crate::is_virtual_path(&fs_path) {
      let parsed = crate::parse_virtual_path(&fs_path)
        .ok_or_else(|| QmdError::InvalidArgument { message: format!("Invalid virtual path: {fs_path}") })?;
      return self.remove_context(&parsed.collection_name, &parsed.path).await;
    }

    let fs_path = if crate::is_absolute_path(&fs_path) {
      fs_path
    } else {
      crate::resolve(&[get_pwd().as_str(), fs_path.as_str()])?
    };

    let detected = detect_collection_from_fs_path(self.storage.as_ref(), &fs_path).await?;
    let Some((collection_name, relative_path)) = detected else {
      return Err(QmdError::InvalidArgument { message: format!("Path is not in any indexed collection: {fs_path}") });
    };

    self.remove_context(&collection_name, &relative_path).await
  }

  // ── collection ──────────────────────────────────────────────────────

  pub async fn cmd_collection_list(&self) -> Result<Vec<CollectionListItem>> {
    self.list_collections().await
  }

  pub async fn cmd_collection_show(&self, name: &str) -> Result<Option<NamedCollection>> {
    let cols = self.storage.list_collections().await?;
    Ok(cols.into_iter().find(|c| c.name == name))
  }

  pub async fn cmd_collection_add(&self, pwd: &str, glob_pattern: &str, name: Option<&str>) -> Result<String> {
    let pwd = crate::normalize_path_separators(&expand_home(pwd));

    let coll_name = match name {
      Some(n) if !n.trim().is_empty() => n.trim().to_string(),
      _ => {
        let parts: Vec<&str> = pwd.split('/').filter(|p| !p.is_empty()).collect();
        parts.last().copied().unwrap_or("root").to_string()
      }
    };

    let existing = self.storage.list_collections().await?;
    if existing.iter().any(|c| c.name == coll_name) {
      return Err(QmdError::InvalidArgument { message: format!("Collection '{coll_name}' already exists") });
    }
    if existing.iter().any(|c| c.collection.path == pwd && c.collection.pattern == glob_pattern) {
      return Err(QmdError::InvalidArgument {
        message: "A collection already exists for this path and pattern".to_string(),
      });
    }

    self
      .add_collection(
        &coll_name,
        &AddCollectionOptions { path: pwd, pattern: Some(glob_pattern.to_string()), ignore: None },
      )
      .await?;

    Ok(coll_name)
  }

  pub async fn cmd_collection_remove(&self, name: &str) -> Result<bool> {
    self.remove_collection(name).await
  }

  pub async fn cmd_collection_rename(&self, old_name: &str, new_name: &str) -> Result<bool> {
    self.rename_collection(old_name, new_name).await
  }

  pub async fn cmd_collection_set_update_cmd(&self, name: &str, cmd: Option<&str>) -> Result<()> {
    let cols = self.storage.list_collections().await?;
    let Some(mut existing) = cols.into_iter().find(|c| c.name == name) else {
      return Err(QmdError::InvalidArgument { message: format!("Collection not found: {name}") });
    };

    existing.collection.update = cmd.map(|c| c.to_string());
    self.storage.upsert_collection(name, &existing.collection).await
  }

  pub async fn cmd_collection_include(&self, name: &str) -> Result<()> {
    let cols = self.storage.list_collections().await?;
    let Some(mut existing) = cols.into_iter().find(|c| c.name == name) else {
      return Err(QmdError::InvalidArgument { message: format!("Collection not found: {name}") });
    };
    existing.collection.include_by_default = true;
    self.storage.upsert_collection(name, &existing.collection).await
  }

  pub async fn cmd_collection_exclude(&self, name: &str) -> Result<()> {
    let cols = self.storage.list_collections().await?;
    let Some(mut existing) = cols.into_iter().find(|c| c.name == name) else {
      return Err(QmdError::InvalidArgument { message: format!("Collection not found: {name}") });
    };
    existing.collection.include_by_default = false;
    self.storage.upsert_collection(name, &existing.collection).await
  }

  pub fn cmd_collection_help(&self) -> &'static str {
    "collection commands: list, add, remove/rm, rename/mv, update-cmd/set-update, include, exclude, show/info"
  }

  // ── get / multi-get / ls / skill / pull ─────────────────────────────

  pub async fn cmd_get(
    &self,
    path_with_optional_line: &str,
    from_line: Option<u32>,
    max_lines: Option<u32>,
    line_numbers: bool,
  ) -> Result<String> {
    let mut path = path_with_optional_line.trim().to_string();
    let mut parsed_line: Option<u32> = None;

    if from_line.is_none() {
      if let Some((p, line_str)) = path.rsplit_once(':') {
        if let Ok(n) = line_str.parse::<u32>() {
          path = p.to_string();
          parsed_line = Some(n);
        }
      }
    }

    let from = from_line.or(parsed_line);
    let body = self.get_document_body(&path, Some(&GetBodyOptions { from_line: from, max_lines })).await?;

    let Some(body) = body else {
      return Err(QmdError::InvalidArgument { message: format!("Document not found: {path}") });
    };

    Ok(if line_numbers { add_line_numbers(&body, from.unwrap_or(1)) } else { body })
  }

  pub async fn cmd_multi_get(&self, pattern: &str, max_bytes: Option<usize>) -> Result<MultiGetResponse> {
    self.multi_get(pattern, Some(&MultiGetOptions { include_body: Some(true), max_bytes })).await
  }

  pub async fn cmd_ls(&self, target: Option<&str>) -> Result<Vec<String>> {
    self.storage.list_files(target).await
  }

  pub async fn cmd_pull(&self, refresh: bool) -> Result<()> {
    let embed_model = std::env::var("QMD_EMBED_MODEL")
      .ok()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL_URI.to_string());

    let models =
      vec![embed_model, crate::DEFAULT_GENERATE_MODEL_URI.to_string(), crate::DEFAULT_RERANK_MODEL_URI.to_string()];

    let _ = crate::pull_models(
      &models,
      crate::PullModelsOptions { refresh, cache_dir: Some(crate::default_model_cache_dir()) },
    )
    .await?;

    Ok(())
  }

  pub fn cmd_skill_show(&self) -> Result<String> {
    Ok(EMBEDDED_QMD_SKILL_MAIN.to_string())
  }

  pub async fn cmd_skill_install(&self, global: bool, force: bool, yes: bool) -> Result<()> {
    let install_dir = skill_install_dir(global)?;
    write_embedded_skill(&install_dir, force)?;

    if yes {
      let link_path = claude_skill_link_dir(global)?;
      ensure_claude_skill_link(&link_path, &install_dir, force)?;
    }

    Ok(())
  }

  pub fn cmd_skill_help(&self) -> &'static str {
    "skill commands: show, install, help"
  }
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;

  static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

  #[test]
  fn parse_structured_query_empty_is_plain_query() {
    assert!(parse_structured_query(" \n ").unwrap().is_none());
  }

  #[test]
  fn parse_structured_query_parses_intent_and_lines() {
    let parsed = parse_structured_query("intent: foo\nlex: hello\nvec: world\n").unwrap().unwrap();
    assert_eq!(parsed.intent.as_deref(), Some("foo"));
    assert_eq!(parsed.searches.len(), 2);
    assert_eq!(parsed.searches[0].line, Some(2));
  }

  #[test]
  fn parse_structured_query_errors_on_multiple_plain_lines() {
    let err = parse_structured_query("foo\nbar").unwrap_err();
    assert!(err.to_string().contains("missing a lex:/vec:/hyde:/intent: prefix"));
  }

  #[test]
  fn skill_show_returns_embedded_skill() {
    let skill = QmdStore { storage: Arc::new(StubStorage), llm: None }.cmd_skill_show().unwrap();
    assert!(skill.contains("name: qmd"));
  }

  #[tokio::test]
  async fn skill_install_writes_embedded_files_locally() {
    let _guard = ENV_LOCK.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let old_pwd = std::env::var("PWD").ok();
    unsafe { std::env::set_var("PWD", tmp.path()) };

    let store = QmdStore { storage: Arc::new(StubStorage), llm: None };
    store.cmd_skill_install(false, true, false).await.unwrap();

    let skill_dir = tmp.path().join(".agents").join("skills").join("qmd");
    let skill = std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap();
    let reference = std::fs::read_to_string(skill_dir.join("references").join("mcp-setup.md")).unwrap();
    assert!(skill.contains("QMD - Quick Markdown Search"));
    assert!(reference.contains("qmd mcp"));

    match old_pwd {
      Some(value) => unsafe { std::env::set_var("PWD", value) },
      None => unsafe { std::env::remove_var("PWD") },
    }
  }

  #[test]
  fn parse_structured_query_ignores_expand_prefix() {
    assert!(parse_structured_query("expand: hello").unwrap().is_none());
  }

  #[test]
  fn parse_structured_query_errors_on_expand_mixed_with_typed_lines() {
    let err = parse_structured_query("expand: hello\nlex: world").unwrap_err();
    assert!(err.to_string().contains("cannot mix expand with typed lines"));
  }

  #[test]
  fn parse_structured_query_errors_on_intent_alone() {
    let err = parse_structured_query("intent: foo").unwrap_err();
    assert!(err.to_string().contains("intent: cannot appear alone"));
  }
}
