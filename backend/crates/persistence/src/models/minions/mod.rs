mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
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
pub struct MinionModel {
  pub slug:         String,
  pub display_name: String,
  pub description:  String,
  #[serde(default)]
  pub enabled:      bool,
  #[serde(default)]
  pub prompt:       Option<String>,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl MinionModel {
  pub fn new(slug: String, display_name: String, description: String, prompt: Option<String>) -> Self {
    Self { slug, display_name, description, enabled: true, prompt, created_at: Utc::now(), updated_at: Utc::now() }
  }

  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {MINIONS_TABLE} SCHEMALESS;")).await?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS idx_minions_slug_unique ON TABLE {MINIONS_TABLE} FIELDS slug UNIQUE;"
    ))
    .await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {SYSTEM_MINION_OVERRIDES_TABLE} SCHEMALESS;")).await?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS idx_system_minion_overrides_slug_unique ON TABLE {SYSTEM_MINION_OVERRIDES_TABLE} FIELDS slug UNIQUE;"
    ))
    .await?;
    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct MinionRecord {
  pub id:           MinionId,
  pub slug:         String,
  pub display_name: String,
  pub description:  String,
  pub enabled:      bool,
  pub prompt:       Option<String>,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl From<MinionRecord> for MinionModel {
  fn from(record: MinionRecord) -> Self {
    Self {
      slug:         record.slug,
      display_name: record.display_name,
      description:  record.description,
      enabled:      record.enabled,
      prompt:       record.prompt,
      created_at:   record.created_at,
      updated_at:   record.updated_at,
    }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export, optional_fields)]
pub struct MinionPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub slug:         Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub display_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:  Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled:      Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub prompt:       Option<Option<String>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct SystemMinionOverrideModel {
  pub slug:       String,
  pub enabled:    bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl SystemMinionOverrideModel {
  fn new(slug: String, enabled: bool) -> Self {
    Self { slug, enabled, created_at: Utc::now(), updated_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct SystemMinionOverrideRecord {
  pub id:         RecordId,
  pub slug:       String,
  pub enabled:    bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<SystemMinionOverrideRecord> for SystemMinionOverrideModel {
  fn from(record: SystemMinionOverrideRecord) -> Self {
    Self {
      slug:       record.slug,
      enabled:    record.enabled,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}

pub struct MinionRepository;
pub struct SystemMinionOverrideRepository;

impl MinionRepository {
  pub async fn create(model: MinionModel) -> DatabaseResult<MinionRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(MINIONS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Minion })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: MinionId) -> DatabaseResult<MinionRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Minion })
  }

  pub async fn list() -> DatabaseResult<Vec<MinionRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {MINIONS_TABLE} ORDER BY created_at ASC"))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })
  }

  pub async fn update(id: MinionId, patch: MinionPatch) -> DatabaseResult<MinionRecord> {
    let db = SurrealConnection::db().await;
    let mut model: MinionModel = Self::get(id.clone()).await?.into();

    if let Some(slug) = patch.slug {
      model.slug = slug;
    }
    if let Some(display_name) = patch.display_name {
      model.display_name = display_name;
    }
    if let Some(description) = patch.description {
      model.description = description;
    }
    if let Some(enabled) = patch.enabled {
      model.enabled = enabled;
    }
    if let Some(prompt) = patch.prompt {
      model.prompt = prompt;
    }
    model.updated_at = Utc::now();

    let _: Record = db
      .update(id.inner())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Minion })?;

    Self::get(id).await
  }

  pub async fn delete(id: MinionId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Minion })?;

    Ok(())
  }
}

impl SystemMinionOverrideRepository {
  pub async fn is_enabled(kind: SystemMinionKind) -> DatabaseResult<bool> {
    Ok(Self::get_by_slug(kind.slug()).await?.map(|record| record.enabled).unwrap_or(kind.default_enabled()))
  }

  pub async fn set_enabled(kind: SystemMinionKind, enabled: bool) -> DatabaseResult<()> {
    if enabled == kind.default_enabled() {
      return Self::clear(kind).await;
    }

    let db = SurrealConnection::db().await;
    let existing = Self::get_by_slug(kind.slug()).await?;
    let mut model = existing
      .clone()
      .map(Into::into)
      .unwrap_or_else(|| SystemMinionOverrideModel::new(kind.slug().to_string(), enabled));
    model.enabled = enabled;
    model.updated_at = Utc::now();

    match existing {
      Some(record) => {
        let _: Record = db
          .update(record.id)
          .content(model)
          .await
          .map_err(|e| DatabaseError::Operation {
            entity:    DatabaseEntity::Minion,
            operation: DatabaseOperation::Update,
            source:    e.into(),
          })?
          .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Minion })?;
      }
      None => {
        let record_id = RecordId::new(SYSTEM_MINION_OVERRIDES_TABLE, Uuid::new_v7());
        let _: Record = db
          .create(record_id)
          .content(model)
          .await
          .map_err(|e| DatabaseError::Operation {
            entity:    DatabaseEntity::Minion,
            operation: DatabaseOperation::Create,
            source:    e.into(),
          })?
          .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Minion })?;
      }
    }

    Ok(())
  }

  pub async fn clear(kind: SystemMinionKind) -> DatabaseResult<()> {
    let Some(record) = Self::get_by_slug(kind.slug()).await? else {
      return Ok(());
    };

    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(record.id)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Minion })?;

    Ok(())
  }

  async fn get_by_slug(slug: &str) -> DatabaseResult<Option<SystemMinionOverrideRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {SYSTEM_MINION_OVERRIDES_TABLE} WHERE slug = $slug LIMIT 1"))
      .bind(("slug", slug.to_string()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Minion,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })
  }
}
