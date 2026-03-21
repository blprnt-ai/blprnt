use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use llama_cpp::EmbeddingsParams;
use llama_cpp::LlamaModel;
use llama_cpp::LlamaParams;
use llama_cpp::SessionParams;
use llama_cpp::Token;
use llama_cpp::standard_sampler::SamplerStage;
use llama_cpp::standard_sampler::StandardSampler;
use tokio::sync::Mutex;

use crate::EmbeddingResult;
use crate::ExpandedQuery;
use crate::ExpandedQueryType;
use crate::GenerateResult;
use crate::QmdError;
use crate::RerankDocumentResult;
use crate::RerankResult;
use crate::Result;

#[derive(Clone)]
pub struct LocalLlm {
  cache_dir:           PathBuf,
  embed_model_uri:     String,
  generate_model_uri:  String,
  expand_context_size: u32,
  models:              Arc<Mutex<HashMap<String, LlamaModel>>>,
}

impl std::fmt::Debug for LocalLlm {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("LocalLlm").finish_non_exhaustive()
  }
}

fn ci_mode() -> bool {
  std::env::var("CI").ok().map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn resolve_expand_context_size() -> u32 {
  const DEFAULT: u32 = 2048;
  match std::env::var("QMD_EXPAND_CONTEXT_SIZE").ok().map(|s| s.trim().to_string()) {
    Some(s) if !s.is_empty() => s.parse::<u32>().ok().filter(|n| *n > 0).unwrap_or(DEFAULT),
    _ => DEFAULT,
  }
}

fn resolve_threads() -> u32 {
  std::thread::available_parallelism().ok().map(|n| n.get() as u32).filter(|n| *n > 0).unwrap_or(1)
}

impl Default for LocalLlm {
  fn default() -> Self {
    let cache_dir = crate::default_model_cache_dir();
    let embed_model_uri = std::env::var("QMD_EMBED_MODEL")
      .ok()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| crate::DEFAULT_EMBED_MODEL_URI.to_string());

    Self {
      cache_dir,
      embed_model_uri,
      generate_model_uri: crate::DEFAULT_GENERATE_MODEL_URI.to_string(),
      expand_context_size: resolve_expand_context_size(),
      models: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl LocalLlm {
  pub fn new(cache_dir: PathBuf, embed_model_uri: String, generate_model_uri: String) -> Self {
    Self {
      cache_dir,
      embed_model_uri,
      generate_model_uri,
      expand_context_size: resolve_expand_context_size(),
      models: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  async fn get_model(&self, model_uri: &str) -> Result<LlamaModel> {
    {
      let models = self.models.lock().await;
      if let Some(m) = models.get(model_uri) {
        return Ok(m.clone());
      }
    }

    let path = crate::resolve_model_file(model_uri, &self.cache_dir).await?;
    let params = LlamaParams::default();
    let model = LlamaModel::load_from_file_async(path, params)
      .await
      .map_err(|e| QmdError::Llm { message: format!("failed to load model '{model_uri}': {e}") })?;

    let mut models = self.models.lock().await;
    models.entry(model_uri.to_string()).or_insert_with(|| model.clone());
    Ok(model)
  }

  fn truncate_to_context_size<'a>(&self, model: &LlamaModel, text: &'a str) -> Result<Cow<'a, str>> {
    let max_tokens = model.train_len();
    if max_tokens == 0 {
      return Ok(Cow::Borrowed(text));
    }

    let tokens = model
      .tokenize_bytes(text.as_bytes(), true, false)
      .map_err(|e| QmdError::Llm { message: format!("tokenize failed: {e}") })?;

    if tokens.len() <= max_tokens {
      return Ok(Cow::Borrowed(text));
    }

    let safe_limit = max_tokens.saturating_sub(4).max(1);
    let truncated_tokens: Vec<Token> = tokens.into_iter().take(safe_limit).collect();
    Ok(Cow::Owned(model.decode_tokens(truncated_tokens)))
  }

  async fn generate_with(&self, prompt: &str, model_uri: &str, max_tokens: usize, temperature: f32) -> Result<String> {
    let model = self.get_model(model_uri).await?;

    let threads = resolve_threads();
    let mut sess_params = SessionParams::default();
    sess_params.n_ctx = self.expand_context_size.max(256);
    sess_params.n_threads = threads;
    sess_params.n_threads_batch = threads;

    let mut session = model
      .create_session(sess_params)
      .map_err(|e| QmdError::Llm { message: format!("failed to create session: {e}") })?;

    session
      .set_context_async(prompt)
      .await
      .map_err(|e| QmdError::Llm { message: format!("failed to set context: {e}") })?;

    // Rough parity with TypeScript defaults (Qwen3 recommended non-greedy sampling).
    let sampler = StandardSampler::new_softmax(
      vec![
        SamplerStage::RepetitionPenalty {
          repetition_penalty: 1.0,
          frequency_penalty:  0.0,
          presence_penalty:   0.5,
          last_n:             64,
        },
        SamplerStage::Temperature(temperature),
        SamplerStage::TopK(20),
        SamplerStage::TopP(0.8),
      ],
      1,
    );

    let handle = session
      .start_completing_with(sampler, max_tokens)
      .map_err(|e| QmdError::Llm { message: format!("failed to start completion: {e}") })?;
    Ok(handle.into_string_async().await)
  }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
  if a.len() != b.len() || a.is_empty() {
    return 0.0;
  }
  let mut dot = 0.0f32;
  let mut na = 0.0f32;
  let mut nb = 0.0f32;
  for i in 0..a.len() {
    dot += a[i] * b[i];
    na += a[i] * a[i];
    nb += b[i] * b[i];
  }
  if na == 0.0 || nb == 0.0 {
    return 0.0;
  }
  dot / (na.sqrt() * nb.sqrt())
}

#[async_trait]
impl crate::LlmBackend for LocalLlm {
  async fn embed(&self, text: &str, model: Option<&str>) -> Result<EmbeddingResult> {
    if ci_mode() {
      return Err(QmdError::Llm { message: "LLM operations are disabled in CI (set CI=false)".to_string() });
    }

    let model_uri =
      model.map(|m| m.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| self.embed_model_uri.clone());

    let model = self.get_model(&model_uri).await?;
    let safe_text = self.truncate_to_context_size(&model, text)?.into_owned();

    let threads = resolve_threads();
    let params = EmbeddingsParams { n_threads: threads, n_threads_batch: threads };
    let inputs = [safe_text.as_bytes()];
    let mut out = model
      .embeddings_async(&inputs, params)
      .await
      .map_err(|e| QmdError::Llm { message: format!("embedding failed: {e}") })?;

    let embedding = out.pop().unwrap_or_default();
    if embedding.is_empty() {
      return Err(QmdError::Llm { message: "embedding model returned an empty vector".to_string() });
    }

    Ok(EmbeddingResult { embedding, model: model_uri })
  }

  async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<GenerateResult> {
    if ci_mode() {
      return Err(QmdError::Llm { message: "LLM operations are disabled in CI (set CI=false)".to_string() });
    }

    let model_uri =
      model.map(|m| m.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| self.generate_model_uri.clone());

    let text = self.generate_with(prompt, &model_uri, 150, 0.7).await?;
    Ok(GenerateResult { text, model: model_uri, done: true })
  }

  async fn rerank(&self, query: &str, documents: &[String], _model: Option<&str>) -> Result<RerankResult> {
    if ci_mode() {
      return Err(QmdError::Llm { message: "LLM operations are disabled in CI (set CI=false)".to_string() });
    }

    let embed_model_uri = self.embed_model_uri.clone();
    let model = self.get_model(&embed_model_uri).await?;

    let threads = resolve_threads();
    let q_params = EmbeddingsParams { n_threads: threads, n_threads_batch: threads };

    let formatted_query = crate::format_query_for_embedding(query, Some(&embed_model_uri));
    let q_safe = self.truncate_to_context_size(&model, &formatted_query)?.into_owned();
    let q_inputs = [q_safe.as_bytes()];
    let q_vecs = model
      .embeddings_async(&q_inputs, q_params)
      .await
      .map_err(|e| QmdError::Llm { message: format!("rerank query embedding failed: {e}") })?;
    let q_vec = q_vecs.into_iter().next().unwrap_or_default();

    if documents.is_empty() {
      return Ok(RerankResult { results: Vec::new(), model: embed_model_uri });
    }

    let mut formatted_docs: Vec<String> = Vec::with_capacity(documents.len());
    for d in documents {
      formatted_docs.push(crate::format_doc_for_embedding(d, None, Some(&embed_model_uri)));
    }

    let doc_inputs: Vec<&[u8]> = formatted_docs.iter().map(|s| s.as_bytes()).collect();
    let params = EmbeddingsParams { n_threads: threads, n_threads_batch: threads };
    let doc_vecs = model
      .embeddings_async(&doc_inputs, params)
      .await
      .map_err(|e| QmdError::Llm { message: format!("rerank doc embeddings failed: {e}") })?;

    let mut results: Vec<RerankDocumentResult> = Vec::new();
    for (idx, vec) in doc_vecs.into_iter().enumerate() {
      let score = cosine_similarity(&q_vec, &vec);
      results.push(RerankDocumentResult { file: String::new(), score, index: idx });
    }

    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    Ok(RerankResult { results, model: embed_model_uri })
  }

  async fn expand_query(&self, query: &str, intent: Option<&str>) -> Result<Vec<ExpandedQuery>> {
    if ci_mode() {
      return Err(QmdError::Llm { message: "LLM operations are disabled in CI (set CI=false)".to_string() });
    }

    let intent_line =
      intent.map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| format!("\nQuery intent: {s}")).unwrap_or_default();

    let prompt = format!(
      "/no_think Expand this search query into multiple variations for different search backends.\n\
Return one query per line, using one of these prefixes: lex:, vec:, hyde:\n\
lex: (for exact keyword/phrase search)\n\
vec: (for semantic/vector search)\n\
hyde: (a hypothetical document snippet that would answer the query)\n\n\
Query: {query}{intent_line}\n"
    );

    let out = self.generate_with(&prompt, &self.generate_model_uri, 600, 0.7).await.unwrap_or_default();

    let query_lower = query.to_lowercase();
    let query_terms: Vec<String> = query_lower
      .chars()
      .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
      .collect::<String>()
      .split_whitespace()
      .map(|s| s.to_string())
      .collect();

    let has_query_term = |text: &str| -> bool {
      if query_terms.is_empty() {
        return true;
      }
      let lower = text.to_lowercase();
      query_terms.iter().any(|t| lower.contains(t))
    };

    let mut expanded: Vec<ExpandedQuery> = Vec::new();
    for line in out.lines() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }
      let Some(colon) = line.find(':') else {
        continue;
      };
      let kind = line[..colon].trim().to_lowercase();
      let text = line[(colon + 1)..].trim();
      if text.is_empty() || !has_query_term(text) {
        continue;
      }

      let query_type = match kind.as_str() {
        "lex" => ExpandedQueryType::Lex,
        "vec" => ExpandedQueryType::Vec,
        "hyde" => ExpandedQueryType::Hyde,
        _ => continue,
      };

      expanded.push(ExpandedQuery { query_type, query: text.to_string(), line: None });
    }

    if expanded.is_empty() {
      return Ok(vec![
        ExpandedQuery {
          query_type: ExpandedQueryType::Hyde,
          query:      format!("Information about {query}"),
          line:       None,
        },
        ExpandedQuery { query_type: ExpandedQueryType::Lex, query: query.to_string(), line: None },
        ExpandedQuery { query_type: ExpandedQueryType::Vec, query: query.to_string(), line: None },
      ]);
    }

    Ok(expanded)
  }
}
