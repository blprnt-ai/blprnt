use common::shared::prelude::LlmModel;

pub mod backoff;
pub mod history_pruning;
pub mod leading_emphasis;
pub mod sse;
pub mod window_licker;

pub fn get_oauth_slug(model: &LlmModel) -> String {
  model.provider_slug.clone().expect("Model must have a provider slug")
}
