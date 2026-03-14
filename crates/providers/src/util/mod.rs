use common::api::LlmModelResponse;

pub mod backoff;
pub mod history_pruning;
pub mod leading_emphasis;
pub mod sse;
pub mod window_licker;

pub fn get_oauth_slug(model: &LlmModelResponse) -> String {
  model.oauth_slug.clone().expect("Model must have an OAuth slug")
}
