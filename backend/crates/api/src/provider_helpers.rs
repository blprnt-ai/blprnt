#![allow(unused)]

use anyhow::Result;
use persistence::prelude::DbId;
use persistence::prelude::ProviderId;
use persistence::prelude::ProviderModel;
use persistence::prelude::ProviderPatch;
use persistence::prelude::ProviderRecord;
use persistence::prelude::ProviderRepository;
use shared::agent::AnthropicOauthToken;
use shared::agent::BlprntCredentials;
use shared::agent::OauthToken;
use shared::agent::OpenAiOauthToken;
use shared::agent::Provider;
use shared::credentials;
use shared::credentials::ClaudeCredential;
use shared::errors::CredentialsError;

pub async fn upsert_provider(provider: ProviderModel) -> Result<ProviderRecord> {
  let record = match ProviderRepository::get_by_provider(provider.provider).await {
    Some(existing) => {
      let provider_patch = ProviderPatch { base_url: provider.base_url, ..Default::default() };
      ProviderRepository::update(existing.id.clone(), provider_patch).await
    }
    None => ProviderRepository::create(provider).await,
  };

  Ok(record?)
}

pub async fn upsert_provider_with_api_key(provider: ProviderModel, api_key: String) -> Result<ProviderRecord> {
  let provider_model = upsert_provider(provider).await?;

  let key = provider_model.id.clone().uuid();

  vault::set_stronghold_secret(vault::Vault::Key, key, &api_key).await?;

  Ok(provider_model)
}

pub async fn create_provider_fnf(provider: ProviderModel) -> Result<ProviderRecord> {
  Ok(ProviderRepository::create(provider).await?)
}

pub async fn update_provider_credential(provider_id: ProviderId, credentials: OauthToken) -> Result<()> {
  let credentials: BlprntCredentials = credentials.into();
  vault::set_stronghold_secret(
    vault::Vault::Key,
    provider_id.uuid(),
    &serde_json::to_string(&credentials).unwrap_or_default(),
  )
  .await?;
  Ok(())
}

pub async fn providers_list() -> Result<Vec<ProviderRecord>> {
  Ok(ProviderRepository::list().await?)
}

pub async fn delete_provider(provider_id: ProviderId) -> Result<()> {
  Ok(ProviderRepository::delete(provider_id).await?)
}

pub async fn link_codex_account() -> Result<ProviderRecord> {
  let codex_credentials = credentials::read_codex_credentials().ok_or(CredentialsError::CodexCredentialsNotFound)?;

  let oauth_token = OauthToken::OpenAi(OpenAiOauthToken {
    access_token:  codex_credentials.access,
    refresh_token: codex_credentials.refresh,
    expires_at_ms: codex_credentials.expires as u64,
    account_id:    codex_credentials.account_id,
  });

  let provider = ProviderModel::new(Provider::Codex);
  let provider = upsert_provider(provider).await?;

  update_provider_credential(provider.id.clone(), oauth_token).await?;

  Ok(provider)
}

pub async fn unlink_codex_account() -> Result<()> {
  let provider =
    ProviderRepository::get_by_provider(Provider::Codex).await.ok_or_else(|| anyhow::anyhow!("Provider not found"))?;
  delete_provider(provider.id.clone()).await?;

  Ok(())
}

pub async fn link_claude_account() -> Result<ProviderRecord> {
  let claude_credentials =
    credentials::read_claude_credentials(true, None).ok_or(CredentialsError::ClaudeCredentialsNotFound)?;

  let provider = ProviderModel::new(Provider::ClaudeCode);
  let provider = upsert_provider(provider).await?;

  match claude_credentials {
    ClaudeCredential::Oauth { access, refresh, expires, .. } => {
      let oauth_token = OauthToken::Anthropic(AnthropicOauthToken {
        access_token:  access,
        refresh_token: refresh,
        expires_at_ms: expires as u64,
      });

      update_provider_credential(provider.id.clone(), oauth_token).await?;
    }
    ClaudeCredential::Token { token, .. } => {
      upsert_provider_with_api_key(provider.clone().into(), token).await?;
    }
  }

  Ok(provider)
}

pub async fn unlink_claude_account() -> Result<()> {
  let provider = ProviderRepository::get_by_provider(Provider::ClaudeCode)
    .await
    .ok_or_else(|| anyhow::anyhow!("Provider not found"))?;
  delete_provider(provider.id.clone()).await?;
  Ok(())
}
