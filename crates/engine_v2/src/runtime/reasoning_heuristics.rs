use std::sync::Arc;

use common::models::ReasoningEffort;
use providers::ProviderAdapter;
use tokio_util::sync::CancellationToken;

use crate::runtime::provider_model_heuristics::base_model_for_provider;

const CLASSIFIER_SYSTEM_PROMPT: &str = "You are a strict classifier for assistant reasoning effort. \
Return exactly one token from this set and nothing else: None, Minimal, Low, Medium, High, XHigh. \
Do not include punctuation, quotes, or extra words. \
Bias towards Medium over High and XHigh. Use High sparingly. Use XHigh only for the most complex and technical reasoning.\
Do not attempt to solve or complete the user's task; only classify.\n\
Guidelines: \
Minimal for greetings/acknowledgements/very short trivial requests. \
Low for simple or single-step tasks. \
Medium for typical multi-step coding or explanation. \
High for complex debugging, refactors, or detailed analysis. \
XHigh for exhaustive or deeply technical reasoning. \
If unsure, choose Low over Minimal.";

pub async fn classify_reasoning_effort(
  prompt: &str,
  recent_history: &[ReasoningEffort],
  provider_adapter: Arc<ProviderAdapter>,
  cancel_token: CancellationToken,
) -> ReasoningEffort {
  let model = base_model_for_provider(provider_adapter.provider());

  let user_prompt = build_classifier_prompt(prompt, recent_history);

  let response = provider_adapter
    .one_off_request(user_prompt, CLASSIFIER_SYSTEM_PROMPT.to_string(), Some(model.to_string()), cancel_token)
    .await;

  match response {
    Ok(chat_basic) => {
      let raw = chat_basic.messages.first().cloned().unwrap_or_default();
      parse_reasoning_effort(&raw).unwrap_or_else(|| {
        tracing::warn!("Reasoning classifier returned unrecognized output: {:?}", raw);
        ReasoningEffort::Medium
      })
    }
    Err(err) => {
      tracing::warn!("Reasoning classifier failed: {}", err);
      ReasoningEffort::Medium
    }
  }
}

fn build_classifier_prompt(prompt: &str, recent_history: &[ReasoningEffort]) -> String {
  let history = if recent_history.is_empty() {
    "none".to_string()
  } else {
    recent_history.iter().map(|level| format!("{:?}", level)).collect::<Vec<_>>().join(", ")
  };

  format!(
    "Classify the reasoning effort for the next response.\nRecent user effort levels (most recent first): {}\nUser prompt:\n{}",
    history, prompt
  )
}

fn parse_reasoning_effort(output: &str) -> Option<ReasoningEffort> {
  let letters: String = output.chars().filter(|c| c.is_ascii_alphabetic()).collect::<String>().to_lowercase();

  let perfect_match = match letters.as_str() {
    "none" => Some(ReasoningEffort::None),
    "minimal" => Some(ReasoningEffort::Minimal),
    "low" => Some(ReasoningEffort::Low),
    "medium" => Some(ReasoningEffort::Medium),
    "high" => Some(ReasoningEffort::High),
    "xhigh" => Some(ReasoningEffort::XHigh),
    _ => None,
  };

  if let Some(perfect_match) = perfect_match {
    Some(perfect_match)
  } else {
    if letters.starts_with("xhigh") {
      Some(ReasoningEffort::XHigh)
    } else if letters.starts_with("high") {
      Some(ReasoningEffort::High)
    } else if letters.starts_with("medium") {
      Some(ReasoningEffort::Medium)
    } else if letters.starts_with("low") {
      Some(ReasoningEffort::Low)
    } else if letters.starts_with("minimal") {
      Some(ReasoningEffort::Minimal)
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_reasoning_effort() {
    assert_eq!(parse_reasoning_effort("Minimal"), Some(ReasoningEffort::Minimal));
    assert_eq!(parse_reasoning_effort("low"), Some(ReasoningEffort::Low));
    assert_eq!(parse_reasoning_effort("Medium."), Some(ReasoningEffort::Medium));
    assert_eq!(parse_reasoning_effort("High\n"), Some(ReasoningEffort::High));
    assert_eq!(parse_reasoning_effort("X-High"), Some(ReasoningEffort::XHigh));
  }
}
