use std::str::FromStr;

use common::api::ApiClient;
use common::api::LlmModelResponse;
use common::errors::IntoTauriResult;
use common::errors::TauriError;
use common::errors::TauriResult;
use common::shared::prelude::*;
use oauth::anthropic::oauth::AnthropicOauth;
use oauth::openai::oauth::OpenAiOauth;
use persistence::prelude::ProviderModelV2;
use persistence::prelude::ProviderRecord;
use surrealdb::types::Uuid;
use vault::Vault;

use crate::engine_manager::provider_handler::ProviderHandler;

#[tauri::command]
#[specta::specta]
pub async fn create_provider(provider: Provider, api_key: String) -> TauriResult<ProviderRecord> {
  tracing::debug!("New Auth: {}", provider);
  let provider_model = ProviderModelV2::new(provider);

  ProviderHandler::upsert_provider_with_api_key(provider_model, api_key).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn create_provider_fnf(provider: Provider) -> TauriResult<ProviderRecord> {
  tracing::debug!("New Auth: {}", provider);

  let oauth_token = match provider {
    Provider::AnthropicFnf => AnthropicOauth::authenticate_with_oauth().await.into_tauri()?,
    Provider::OpenAiFnf => OpenAiOauth::authenticate_with_oauth().await.into_tauri()?,
    _ => return Err(TauriError::new(format!("Failed to get oauth token for provider: {}", provider))),
  };

  let Some(oauth_token) = oauth_token else {
    return Err(TauriError::new("Failed to get oauth token"));
  };

  let provider_model = ProviderModelV2::new(provider);

  let provider_model = ProviderHandler::create_provider_fnf(provider_model).await.into_tauri()?;
  ProviderHandler::update_provider_credential(provider_model.id.clone(), oauth_token).await.into_tauri()?;

  Ok(provider_model)
}

#[derive(serde::Deserialize, specta::Type)]
pub struct UpsertProviderArgs {
  pub provider: Provider,
  pub api_key:  String,
  pub base_url: String,
}

#[tauri::command]
#[specta::specta]
pub async fn upsert_provider(args: UpsertProviderArgs) -> TauriResult<ProviderRecord> {
  tracing::debug!("Upsert Provider: {:?}", args.provider);
  let mut provider_model = ProviderModelV2::new(args.provider);
  provider_model.base_url = Some(args.base_url);

  ProviderHandler::upsert_provider_with_api_key(provider_model, args.api_key).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn list_enabled_providers() -> TauriResult<Vec<Provider>> {
  tracing::debug!("List Enabled Providers");
  Ok(Provider::all())
}

#[derive(serde::Serialize, specta::Type)]
pub struct ProviderDto {
  #[serde(flatten)]
  pub inner:   ProviderRecord,
  pub api_key: String,
}

#[tauri::command]
#[specta::specta]
pub async fn list_providers() -> TauriResult<Vec<ProviderDto>> {
  tracing::debug!("List Providers");
  let providers = ProviderHandler::providers_list().await.into_tauri()?;

  let mut results = vec![];
  for provider in providers {
    let id = provider.id.clone().key().to_string();
    let id = Uuid::from_str(&id).unwrap_or_default();

    let api_key = vault::get_stronghold_secret(Vault::Key, id).await.unwrap_or_default();
    results.push(ProviderDto { inner: provider, api_key: api_key });
  }

  Ok(results)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_provider(provider_id: String) -> TauriResult<()> {
  tracing::debug!("Delete Provider: {:?}", provider_id);
  let provider_id = provider_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  ProviderHandler::delete_provider(provider_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn get_models_catalog() -> TauriResult<Vec<LlmModelResponse>> {
  Ok(ApiClient::get().get_models().await.into_tauri()?.iter().filter(|m| m.enabled).cloned().collect())
}

#[tauri::command]
#[specta::specta]
pub async fn link_codex_account() -> TauriResult<()> {
  ProviderHandler::link_codex_account().await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn unlink_codex_account() -> TauriResult<()> {
  ProviderHandler::unlink_codex_account().await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn link_claude_account() -> TauriResult<()> {
  ProviderHandler::link_claude_account().await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn unlink_claude_account() -> TauriResult<()> {
  ProviderHandler::unlink_claude_account().await.into_tauri()
}
