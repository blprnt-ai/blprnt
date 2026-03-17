use std::path::Path;
use std::path::PathBuf;

use directories::BaseDirs;
use reqwest::header::ETAG;

use crate::QmdError;
use crate::Result;

// HuggingFace model URIs (parity with the TypeScript implementation).
pub const DEFAULT_EMBED_MODEL_URI: &str = "hf:ggml-org/embeddinggemma-300M-GGUF/embeddinggemma-300M-Q8_0.gguf";
pub const DEFAULT_RERANK_MODEL_URI: &str = "hf:ggml-org/Qwen3-Reranker-0.6B-Q8_0-GGUF/qwen3-reranker-0.6b-q8_0.gguf";
pub const DEFAULT_GENERATE_MODEL_URI: &str =
  "hf:tobil/qmd-query-expansion-1.7B-gguf/qmd-query-expansion-1.7B-q4_k_m.gguf";

pub fn default_model_cache_dir() -> PathBuf {
  let home = BaseDirs::new().map(|b| b.home_dir().to_path_buf()).unwrap_or_else(|| PathBuf::from("/"));
  home.join(".cache").join("qmd").join("models")
}

#[derive(Debug, Clone)]
pub struct PullResult {
  pub model:      String,
  pub path:       String,
  pub size_bytes: u64,
  pub refreshed:  bool,
}

#[derive(Debug, Clone, Default)]
pub struct PullModelsOptions {
  pub refresh:   bool,
  pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct HfRef {
  repo: String,
  file: String,
}

fn parse_hf_uri(model: &str) -> Option<HfRef> {
  if !model.starts_with("hf:") {
    return None;
  }
  let without = &model[3..];
  let parts: Vec<&str> = without.split('/').collect();
  if parts.len() < 3 {
    return None;
  }
  let repo = format!("{}/{}", parts[0], parts[1]);
  let file = parts[2..].join("/");
  Some(HfRef { repo, file })
}

async fn get_remote_etag(client: &reqwest::Client, hf: &HfRef) -> Option<String> {
  let url = format!("https://huggingface.co/{}/resolve/main/{}", hf.repo, hf.file);
  match client.head(url).send().await {
    Ok(resp) if resp.status().is_success() => {
      resp.headers().get(ETAG).and_then(|v| v.to_str().ok()).map(|s| s.to_string())
    }
    _ => None,
  }
}

fn filename_from_model_uri(model: &str) -> Option<String> {
  model.rsplit('/').next().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string())
}

fn cached_candidates(cache_dir: &Path, filename: &str) -> Vec<PathBuf> {
  let entries = match std::fs::read_dir(cache_dir) {
    Ok(e) => e,
    Err(_) => return Vec::new(),
  };

  let mut out: Vec<PathBuf> = Vec::new();
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
      continue;
    };
    if name.contains(filename) {
      out.push(path);
    }
  }
  out
}

pub async fn resolve_model_file(model: &str, cache_dir: &Path) -> Result<PathBuf> {
  if let Some(hf) = parse_hf_uri(model) {
    let safe_repo = hf.repo.replace('/', "__");
    let safe_file = hf.file.replace('/', "__");
    let local_name = format!("{safe_repo}__{safe_file}");
    let local_path = cache_dir.join(local_name);
    if local_path.exists() {
      return Ok(local_path);
    }

    let url = format!("https://huggingface.co/{}/resolve/main/{}", hf.repo, hf.file);
    let client = reqwest::Client::new();
    let mut resp =
      client.get(url).send().await.map_err(|e| QmdError::Llm { message: format!("download failed: {e}") })?;
    if !resp.status().is_success() {
      return Err(QmdError::Llm { message: format!("download failed: http {}", resp.status()) });
    }

    if let Some(parent) = local_path.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|e| QmdError::Llm { message: format!("failed to create cache dir: {e}") })?;
    }

    let tmp_path = local_path.with_extension("part");
    let mut file = std::fs::File::create(&tmp_path)
      .map_err(|e| QmdError::Llm { message: format!("failed to create model file: {e}") })?;
    while let Some(chunk) =
      resp.chunk().await.map_err(|e| QmdError::Llm { message: format!("download read failed: {e}") })?
    {
      use std::io::Write as _;
      file.write_all(&chunk).map_err(|e| QmdError::Llm { message: format!("download write failed: {e}") })?;
    }
    std::fs::rename(&tmp_path, &local_path)
      .map_err(|e| QmdError::Llm { message: format!("failed to finalize model file: {e}") })?;

    return Ok(local_path);
  }

  let p = PathBuf::from(model);
  if p.exists() { Ok(p) } else { Err(QmdError::InvalidArgument { message: format!("model path not found: {model}") }) }
}

pub async fn pull_models(models: &[String], options: PullModelsOptions) -> Result<Vec<PullResult>> {
  let cache_dir = options.cache_dir.unwrap_or_else(default_model_cache_dir);
  std::fs::create_dir_all(&cache_dir)
    .map_err(|e| QmdError::Llm { message: format!("failed to create model cache dir: {e}") })?;

  let client = reqwest::Client::new();
  let mut results: Vec<PullResult> = Vec::new();

  for model in models {
    let mut refreshed = false;
    let hf_ref = parse_hf_uri(model);
    let filename = filename_from_model_uri(model);

    let cached = filename.as_deref().map(|f| cached_candidates(&cache_dir, f)).unwrap_or_default();

    if let (Some(hf), Some(filename)) = (&hf_ref, filename.as_deref()) {
      let etag_path = cache_dir.join(format!("{filename}.etag"));
      let remote_etag = get_remote_etag(&client, hf).await;
      let local_etag = std::fs::read_to_string(&etag_path).ok().map(|s| s.trim().to_string());

      let should_refresh = options.refresh || remote_etag.is_none() || remote_etag != local_etag || cached.is_empty();

      if should_refresh {
        for candidate in &cached {
          let _ = std::fs::remove_file(candidate);
        }
        let _ = std::fs::remove_file(&etag_path);
        refreshed = !cached.is_empty();
      }
    } else if options.refresh {
      for candidate in &cached {
        let _ = std::fs::remove_file(candidate);
        refreshed = true;
      }
    }

    let path = resolve_model_file(model, &cache_dir).await?;
    let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    if let (Some(hf), Some(filename)) = (&hf_ref, filename.as_deref()) {
      if let Some(remote_etag) = get_remote_etag(&client, hf).await {
        let etag_path = cache_dir.join(format!("{filename}.etag"));
        let _ = std::fs::write(&etag_path, format!("{remote_etag}\n"));
      }
    }

    results.push(PullResult { model: model.clone(), path: path.to_string_lossy().to_string(), size_bytes, refreshed });
  }

  Ok(results)
}
