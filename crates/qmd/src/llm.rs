use async_trait::async_trait;

use crate::Result;

#[derive(Debug, Clone)]
pub struct EmbeddingResult {
  pub embedding: Vec<f32>,
  pub model:     String,
}

#[derive(Debug, Clone)]
pub struct GenerateResult {
  pub text:  String,
  pub model: String,
  pub done:  bool,
}

#[derive(Debug, Clone)]
pub struct RerankDocumentResult {
  pub file:  String,
  pub score: f32,
  pub index: usize,
}

#[derive(Debug, Clone)]
pub struct RerankResult {
  pub results: Vec<RerankDocumentResult>,
  pub model:   String,
}

#[async_trait]
pub trait LlmBackend: Send + Sync {
  async fn embed(&self, _text: &str, _model: Option<&str>) -> Result<EmbeddingResult>;
  async fn generate(&self, _prompt: &str, _model: Option<&str>) -> Result<GenerateResult>;
  async fn rerank(&self, _query: &str, _documents: &[String], _model: Option<&str>) -> Result<RerankResult>;
  async fn expand_query(&self, _query: &str, _intent: Option<&str>) -> Result<Vec<crate::ExpandedQuery>>;
}

pub struct StubLlm;

#[async_trait]
impl LlmBackend for StubLlm {
  async fn embed(&self, _text: &str, _model: Option<&str>) -> Result<EmbeddingResult> {
    Err(crate::QmdError::NotImplemented { op: "LlmBackend::embed" })
  }

  async fn generate(&self, _prompt: &str, _model: Option<&str>) -> Result<GenerateResult> {
    Err(crate::QmdError::NotImplemented { op: "LlmBackend::generate" })
  }

  async fn rerank(&self, _query: &str, _documents: &[String], _model: Option<&str>) -> Result<RerankResult> {
    Err(crate::QmdError::NotImplemented { op: "LlmBackend::rerank" })
  }

  async fn expand_query(&self, _query: &str, _intent: Option<&str>) -> Result<Vec<crate::ExpandedQuery>> {
    Err(crate::QmdError::NotImplemented { op: "LlmBackend::expand_query" })
  }
}
