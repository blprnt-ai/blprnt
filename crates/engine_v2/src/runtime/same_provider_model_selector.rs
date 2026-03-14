#![allow(clippy::large_enum_variant)]

use anyhow::Result;
use common::api::LlmModelResponse;
use common::shared::prelude::BlprntCredentials;
use common::shared::prelude::Provider;
use persistence::prelude::ProviderRecord;
use persistence::prelude::SessionRecord;
use providers::build_adapter;
use tokio_util::sync::CancellationToken;

use crate::runtime::context::RuntimeContext;
use crate::runtime::provider_model_heuristics::base_model_for_provider;

#[derive(Clone, Debug)]
pub(crate) struct SameProviderSmallModelSelection {
  pub credentials: BlprntCredentials,
  pub provider:    ProviderRecord,
  pub model:       LlmModelResponse,
}

#[derive(Clone, Debug)]
pub(crate) enum SameProviderSmallModelSelectionOutcome {
  Selected(SameProviderSmallModelSelection),
  Skip,
}

pub(crate) async fn select_same_provider_small_model_for_session(
  session_model: &SessionRecord,
) -> Result<SameProviderSmallModelSelectionOutcome> {
  let model_info = RuntimeContext::llm_model(session_model).await?;
  let mut exact_candidates = Vec::new();

  for provider in candidate_exact_providers_for_model(&model_info) {
    if RuntimeContext::selected_exact_provider(provider).await?.is_some() {
      exact_candidates.push(provider);
    }
  }

  if exact_candidates.is_empty() {
    tracing::warn!("No exact candidates found for model {}", model_info.slug);
    return Ok(SameProviderSmallModelSelectionOutcome::Skip);
  }

  if exact_candidates.len() > 1 {
    tracing::warn!("Multiple exact candidates found for model {}", model_info.slug);
    return Ok(SameProviderSmallModelSelectionOutcome::Skip);
  }

  select_same_provider_small_model_for_provider(exact_candidates[0]).await
}

pub async fn same_provider_one_off_text_for_session(
  session_model: &SessionRecord,
  prompt: String,
  system: String,
  cancel_token: CancellationToken,
) -> Result<Option<String>> {
  let selection = match select_same_provider_small_model_for_session(session_model).await? {
    SameProviderSmallModelSelectionOutcome::Selected(selection) => selection,
    SameProviderSmallModelSelectionOutcome::Skip => return Ok(None),
  };

  let Some(model) = one_off_request_model(&selection.provider.provider(), &selection.model) else {
    return Ok(None);
  };

  let adapter = build_adapter(
    &selection.provider.provider(),
    Some(selection.credentials.clone()),
    selection.provider.base_url.clone(),
  )?;

  let response = adapter.one_off_request(prompt, system, Some(model), cancel_token).await?;
  let text = response
    .messages
    .into_iter()
    .map(|message| message.trim().to_string())
    .filter(|message| !message.is_empty())
    .collect::<Vec<_>>()
    .join("\n");

  if text.is_empty() {
    return Ok(None);
  }

  Ok(Some(text))
}

pub(crate) async fn select_same_provider_small_model_for_provider(
  provider: Provider,
) -> Result<SameProviderSmallModelSelectionOutcome> {
  let Some((credentials, provider)) = RuntimeContext::selected_exact_provider(provider).await? else {
    tracing::warn!("No exact provider found for model {}", provider);
    return Ok(SameProviderSmallModelSelectionOutcome::Skip);
  };

  let all_models = RuntimeContext::session_resolvable_models().await?;

  let Some(model) = find_same_provider_small_model(&all_models, provider.provider()) else {
    tracing::warn!("No same-provider small model found for provider {:?}", provider.provider());
    return Ok(SameProviderSmallModelSelectionOutcome::Skip);
  };

  Ok(SameProviderSmallModelSelectionOutcome::Selected(SameProviderSmallModelSelection { credentials, provider, model }))
}

pub(crate) fn find_same_provider_small_model(
  enabled_models: &[LlmModelResponse],
  provider: Provider,
) -> Option<LlmModelResponse> {
  let target_model = base_model_for_provider(provider);

  enabled_models.iter().find(|model| model_matches_provider_small_model(model, provider, target_model)).cloned()
}

fn model_matches_provider_small_model(model: &LlmModelResponse, provider: Provider, target_model: &str) -> bool {
  if model.slug == target_model || model.oauth_slug.as_deref() == Some(target_model) {
    return true;
  }

  match provider {
    Provider::OpenAi | Provider::OpenAiFnf => model.slug.strip_prefix("openai/") == Some(target_model),
    Provider::Anthropic | Provider::AnthropicFnf => model.slug.strip_prefix("anthropic/") == Some(target_model),
    Provider::Blprnt => model.slug.strip_prefix("blprnt/") == Some(target_model),
    Provider::OpenRouter | Provider::Mock => false,
  }
}

fn candidate_exact_providers_for_model(model: &LlmModelResponse) -> Vec<Provider> {
  let supports_direct_oauth = model.supports_oauth && model.oauth_slug.is_some();

  if model.slug.starts_with("openai/") {
    if supports_direct_oauth {
      return vec![Provider::OpenAiFnf, Provider::OpenAi];
    }

    return vec![];
  }

  if model.slug.starts_with("anthropic/") {
    if supports_direct_oauth {
      return vec![Provider::AnthropicFnf, Provider::Anthropic];
    }

    return vec![];
  }

  if model.slug.starts_with("openrouter/") {
    return vec![Provider::OpenRouter];
  }

  if model.slug.starts_with("blprnt/") {
    return vec![Provider::Blprnt];
  }

  if model.slug.starts_with("mock/") {
    return vec![Provider::Mock];
  }

  vec![]
}

fn one_off_request_model(provider: &Provider, model: &LlmModelResponse) -> Option<String> {
  match provider {
    Provider::OpenAi | Provider::OpenAiFnf | Provider::Anthropic | Provider::AnthropicFnf => model.oauth_slug.clone(),
    Provider::OpenRouter | Provider::Blprnt | Provider::Mock => Some(model.slug.clone()),
  }
}

#[cfg(test)]
mod tests {
  use common::api::LlmModelResponse;
  use common::shared::prelude::Provider;

  use super::find_same_provider_small_model;
  use super::model_matches_provider_small_model;

  #[test]
  fn matches_provider_small_model_using_oauth_slug() {
    let model = LlmModelResponse {
      slug: "openai/gpt-5.1-codex-mini-2025-11-13".to_string(),
      oauth_slug: Some("gpt-5.1-codex-mini".to_string()),
      supports_oauth: true,
      ..Default::default()
    };

    assert!(model_matches_provider_small_model(&model, Provider::OpenAi, "gpt-5.1-codex-mini"));
  }

  #[test]
  fn finds_same_provider_small_model_in_enabled_catalog() {
    let models = vec![
      LlmModelResponse {
        slug: "openai/gpt-5.1-codex-mini-2025-11-13".to_string(),
        oauth_slug: Some("gpt-5.1-codex-mini".to_string()),
        supports_oauth: true,
        ..Default::default()
      },
      LlmModelResponse { slug: "openrouter/auto".to_string(), ..Default::default() },
    ];

    let selected = find_same_provider_small_model(&models, Provider::OpenAi).expect("small model should resolve");
    assert_eq!(selected.oauth_slug.as_deref(), Some("gpt-5.1-codex-mini"));
  }

  #[test]
  fn returns_none_when_small_model_is_not_enabled() {
    let models = vec![LlmModelResponse { slug: "openrouter/auto".to_string(), ..Default::default() }];

    assert!(find_same_provider_small_model(&models, Provider::Anthropic).is_none());
  }

  #[test]
  fn matches_small_model_for_exact_provider_variant() {
    let model = LlmModelResponse {
      slug: "anthropic/claude-haiku-4-5".to_string(),
      oauth_slug: Some("claude-haiku-4-5".to_string()),
      supports_oauth: true,
      ..Default::default()
    };

    assert!(model_matches_provider_small_model(&model, Provider::Anthropic, "claude-haiku-4-5"));
    assert!(model_matches_provider_small_model(&model, Provider::AnthropicFnf, "claude-haiku-4-5"));
  }
}
