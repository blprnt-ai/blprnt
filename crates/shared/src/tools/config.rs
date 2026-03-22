#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct LlmModel {
  pub name:               String,
  pub slug:               String,
  pub context_length:     i64,
  pub supports_reasoning: bool,
  pub provider_slug:      Option<String>,
  pub enabled:            bool,
}
