use std::str::FromStr;

use anyhow::Result;
use common::credentials::ClaudeCredential;
use common::errors::AppCoreError;
use common::shared::prelude::AnthropicOauthToken;
use common::shared::prelude::BlprntCredentials;
use common::shared::prelude::OauthToken;
use common::shared::prelude::OpenAiOauthToken;
use common::shared::prelude::Provider;
use persistence::prelude::ProviderModelV2;
use persistence::prelude::ProviderPatchV2;
use persistence::prelude::ProviderRecord;
use persistence::prelude::ProviderRepositoryV2;
use persistence::prelude::SurrealId;
use surrealdb::types::Uuid;

pub struct ProviderHandler;

impl ProviderHandler {
  pub async fn upsert_provider(provider: ProviderModelV2) -> Result<ProviderRecord> {
    match ProviderRepositoryV2::get_by_provider(provider.provider).await {
      Some(existing) => {
        let provider_patch = ProviderPatchV2 { base_url: provider.base_url, ..Default::default() };
        ProviderRepositoryV2::update(existing.id.clone(), provider_patch).await
      }
      None => ProviderRepositoryV2::create(provider).await,
    }
  }

  pub async fn upsert_provider_with_api_key(provider: ProviderModelV2, api_key: String) -> Result<ProviderRecord> {
    let provider_model = Self::upsert_provider(provider).await?;

    let key = provider_model.id.clone().key().to_string();
    let key = Uuid::from_str(&key).unwrap_or_default();
    vault::set_stronghold_secret(vault::Vault::Key, key, &api_key).await?;

    Ok(provider_model)
  }

  pub async fn create_provider_fnf(provider: ProviderModelV2) -> Result<ProviderRecord> {
    ProviderRepositoryV2::create(provider).await
  }

  pub async fn update_provider_credential(provider_id: SurrealId, credentials: OauthToken) -> Result<()> {
    let credentials: BlprntCredentials = credentials.into();
    vault::set_stronghold_secret(
      vault::Vault::Key,
      provider_id.key(),
      &serde_json::to_string(&credentials).unwrap_or_default(),
    )
    .await?;
    Ok(())
  }

  pub async fn providers_list() -> Result<Vec<ProviderRecord>> {
    ProviderRepositoryV2::list().await
  }

  pub async fn delete_provider(provider_id: SurrealId) -> Result<()> {
    ProviderRepositoryV2::delete(provider_id).await
  }

  pub async fn link_codex_account() -> Result<()> {
    let codex_credentials =
      common::credentials::read_codex_credentials().ok_or(AppCoreError::CodexCredentialsNotFound)?;

    let oauth_token = OauthToken::OpenAi(OpenAiOauthToken {
      access_token:  codex_credentials.access,
      refresh_token: codex_credentials.refresh,
      expires_at_ms: codex_credentials.expires as u64,
      account_id:    codex_credentials.account_id,
    });

    let provider = ProviderModelV2::new(Provider::OpenAiFnf);
    let provider = Self::upsert_provider(provider).await?;

    Self::update_provider_credential(provider.id.clone(), oauth_token).await?;

    Ok(())
  }

  pub async fn unlink_codex_account() -> Result<()> {
    let provider = ProviderRepositoryV2::get_by_provider(Provider::OpenAiFnf)
      .await
      .ok_or_else(|| anyhow::anyhow!("Provider not found"))?;
    Self::delete_provider(provider.id.clone()).await?;

    Ok(())
  }

  pub async fn link_claude_account() -> Result<()> {
    let claude_credentials =
      common::credentials::read_claude_credentials(true, None).ok_or(AppCoreError::ClaudeCredentialsNotFound)?;

    let provider = ProviderModelV2::new(Provider::AnthropicFnf);
    let provider = Self::upsert_provider(provider).await?;

    match claude_credentials {
      ClaudeCredential::Oauth { access, refresh, expires, .. } => {
        let oauth_token = OauthToken::Anthropic(AnthropicOauthToken {
          access_token:  access,
          refresh_token: refresh,
          expires_at_ms: expires as u64,
        });

        Self::update_provider_credential(provider.id.clone(), oauth_token).await?;
      }
      ClaudeCredential::Token { token, .. } => {
        Self::upsert_provider_with_api_key(provider.into(), token).await?;
      }
    }

    Ok(())
  }

  pub async fn unlink_claude_account() -> Result<()> {
    let provider = ProviderRepositoryV2::get_by_provider(Provider::AnthropicFnf)
      .await
      .ok_or_else(|| anyhow::anyhow!("Provider not found"))?;
    Self::delete_provider(provider.id.clone()).await?;
    Ok(())
  }
}
