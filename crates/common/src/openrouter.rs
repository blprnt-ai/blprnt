use http::header::AUTHORIZATION;
use surrealdb_types::SurrealValue;

use crate::errors::ApiError;

pub struct OpenRouterApi {
  base_url: String,
  api_key:  String,
}

impl Default for OpenRouterApi {
  fn default() -> Self {
    Self::new()
  }
}

impl OpenRouterApi {
  pub fn new() -> Self {
    let api_key = std::env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY environment variable must be set");

    Self { base_url: "https://openrouter.ai/api/v1".into(), api_key }
  }
}

impl OpenRouterApi {
  pub async fn get_generation_result(&self, response_id: String) -> anyhow::Result<GenerationResponse> {
    let url = format!("{}/generation?id={}", self.base_url, response_id);
    let response = reqwest::Client::new()
      .get(url)
      .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
      .send()
      .await
      .map_err(|e| ApiError::FailedToGetResponse(e.to_string()))?;

    let body = response.text().await.map_err(|e| ApiError::FailedToGetResponse(format!("response: {}", e)))?;

    let generation_response = serde_json::from_str::<GenerationResponse>(&body)
      .map_err(|e| ApiError::FailedToGetResponse(format!("parse: {}", e)))?;

    Ok(generation_response)
  }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct GenerationResponse {
  pub data: GenerationResult,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct GenerationResult {
  pub created_at:               String,
  pub model:                    String,
  pub cancelled:                bool,
  pub latency:                  i64,
  pub generation_time:          i64,
  pub tokens_prompt:            i64,
  pub tokens_completion:        i64,
  pub native_tokens_prompt:     i64,
  pub native_tokens_completion: i64,
  pub native_tokens_reasoning:  i64,
  pub native_tokens_cached:     i64,
  pub num_media_prompt:         Option<i64>,
  pub num_input_audio_prompt:   Option<i64>,
  pub num_media_completion:     i64,
  pub num_search_results:       Option<i64>,
  pub is_byok:                  bool,
  pub finish_reason:            String,
  pub native_finish_reason:     String,
  pub usage:                    i64,
  pub api_type:                 String,
  pub id:                       String,
  pub upstream_id:              String,
  pub total_cost:               i64,
  pub upstream_inference_cost:  i64,
  pub provider_name:            String,
}
