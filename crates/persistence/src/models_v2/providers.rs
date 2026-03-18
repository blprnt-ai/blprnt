use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::Provider;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::Record;

pub const PROVIDERS_TABLE: &str = "providers";

// TODO: Replace id with ProviderId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProviderId(SurrealId);

impl DbId for ProviderId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for ProviderId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(PROVIDERS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProviderId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProviderModelV2 {
  pub provider:   Provider,
  pub base_url:   Option<String>,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at: DateTime<Utc>,
}

impl ProviderModelV2 {
  pub fn new(provider: Provider) -> Self {
    Self { provider: provider, base_url: None, created_at: Utc::now(), updated_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProviderRecord {
  pub id:         SurrealId,
  pub provider:   Provider,
  pub base_url:   Option<String>,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at: DateTime<Utc>,
}

impl ProviderModelV2 {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {PROVIDERS_TABLE} SCHEMALESS;")).await?;
    Ok(())
  }
}

impl From<ProviderRecord> for ProviderModelV2 {
  fn from(record: ProviderRecord) -> Self {
    Self {
      provider:   record.provider,
      base_url:   record.base_url,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}

impl ProviderRecord {
  pub fn provider(&self) -> Provider {
    self.provider
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  pub fn is_open_ai(&self) -> bool {
    self.provider() == Provider::OpenAi || self.provider() == Provider::OpenAiFnf
  }

  pub fn is_anthropic(&self) -> bool {
    self.provider() == Provider::Anthropic || self.provider() == Provider::AnthropicFnf
  }

  pub fn is_open_router(&self) -> bool {
    self.provider() == Provider::OpenRouter
  }

  pub fn is_blprnt(&self) -> bool {
    self.provider() == Provider::Blprnt
  }

  pub fn is_fnf(&self) -> bool {
    matches!(self.provider(), Provider::AnthropicFnf | Provider::OpenAiFnf)
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProviderPatchV2 {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_url:   Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at: Option<DateTime<Utc>>,
}

pub struct ProviderRepositoryV2;

impl ProviderRepositoryV2 {
  pub async fn create(model: ProviderModelV2) -> Result<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(PROVIDERS_TABLE, Uuid::new_v7());
    db.create(record_id).content(model).await?.ok_or(anyhow::anyhow!("Failed to create provider"))
  }

  pub async fn get(id: SurrealId) -> Result<ProviderRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Provider not found"))
  }

  pub async fn get_by_provider(provider: Provider) -> Option<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let result: Option<ProviderRecord> = db
      .query(format!("SELECT * FROM {PROVIDERS_TABLE} WHERE provider = $provider LIMIT 1"))
      .bind(("provider", provider))
      .await
      .ok()?
      .take(0)
      .ok()?;

    result
  }

  pub async fn list() -> Result<Vec<ProviderRecord>> {
    let db = SurrealConnection::db().await;
    Ok(db.query(format!("SELECT * FROM {PROVIDERS_TABLE}")).await?.take(0)?)
  }

  pub async fn update(id: SurrealId, patch: ProviderPatchV2) -> Result<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let mut provider_model: ProviderModelV2 = Self::get(id.clone()).await?.into();
    provider_model.updated_at = Utc::now();

    if let Some(base_url) = patch.base_url {
      provider_model.base_url = Some(base_url);
    }

    provider_model.updated_at = Utc::now();

    let _: Option<Record> = db.update(id.inner()).merge(provider_model).await?;

    Self::get(id).await
  }

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete provider"))?;

    Ok(())
  }
}
