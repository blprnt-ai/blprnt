mod types;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use shared::agent::Provider;
use shared::errors::DatabaseEntity;
use shared::errors::DatabaseError;
use shared::errors::DatabaseOperation;
use shared::errors::DatabaseResult;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::Record;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProviderModel {
  pub provider:   Provider,
  pub base_url:   Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ProviderModel {
  pub fn new(provider: Provider) -> Self {
    Self { provider: provider, base_url: None, created_at: Utc::now(), updated_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProviderRecord {
  pub id:         ProviderId,
  pub provider:   Provider,
  pub base_url:   Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ProviderModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {PROVIDERS_TABLE} SCHEMALESS;")).await?;
    Ok(())
  }
}

impl From<ProviderRecord> for ProviderModel {
  fn from(record: ProviderRecord) -> Self {
    Self {
      provider:   record.provider,
      base_url:   record.base_url,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProviderPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub base_url:   Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at: Option<DateTime<Utc>>,
}

pub struct ProviderRepository;

impl ProviderRepository {
  pub async fn create(model: ProviderModel) -> DatabaseResult<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(PROVIDERS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Provider })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: ProviderId) -> DatabaseResult<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let record: ProviderRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Provider })?;

    Ok(record)
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

  pub async fn list() -> DatabaseResult<Vec<ProviderRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<ProviderRecord> = db
      .query(format!("SELECT * FROM {PROVIDERS_TABLE}"))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;

    Ok(records)
  }

  pub async fn update(id: ProviderId, patch: ProviderPatch) -> DatabaseResult<ProviderRecord> {
    let db = SurrealConnection::db().await;
    let mut provider_model: ProviderModel = Self::get(id.clone()).await?.into();
    provider_model.updated_at = Utc::now();

    if let Some(base_url) = patch.base_url {
      provider_model.base_url = Some(base_url);
    }

    provider_model.updated_at = Utc::now();

    let _: Record = db
      .update(id.inner())
      .merge(provider_model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Provider })?;

    Self::get(id).await
  }

  pub async fn delete(id: ProviderId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Provider,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Provider })?;

    Ok(())
  }
}
