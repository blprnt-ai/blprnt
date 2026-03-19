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
pub struct ProjectModel {
  pub name:                String,
  pub working_directories: Vec<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl ProjectModel {
  pub fn new(name: String, working_directories: Vec<String>) -> Self {
    Self {
      name:                name,
      working_directories: working_directories,
      created_at:          Utc::now(),
      updated_at:          Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectRecord {
  pub id:                  ProjectId,
  pub name:                String,
  pub working_directories: Vec<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl From<ProjectRecord> for ProjectModel {
  fn from(record: ProjectRecord) -> Self {
    Self {
      name:                record.name,
      working_directories: record.working_directories,
      created_at:          record.created_at,
      updated_at:          record.updated_at,
    }
  }
}

impl ProjectModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {PROJECTS_TABLE} SCHEMALESS;")).await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:                Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_directories: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:          Option<DateTime<Utc>>,
}

pub struct ProjectRepository;

impl ProjectRepository {
  pub async fn create(model: ProjectModel) -> DatabaseResult<ProjectRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(PROJECTS_TABLE, Uuid::new_v7());

    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Project })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: ProjectId) -> DatabaseResult<ProjectRecord> {
    let db = SurrealConnection::db().await;
    let record: ProjectRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Project })?;

    Ok(record)
  }

  pub async fn list() -> DatabaseResult<Vec<ProjectRecord>> {
    let db = SurrealConnection::db().await;

    let records: Vec<ProjectRecord> = db
      .query(format!("SELECT * FROM {PROJECTS_TABLE}"))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;

    Ok(records)
  }

  pub async fn update(id: ProjectId, patch: ProjectPatch) -> DatabaseResult<ProjectRecord> {
    let db = SurrealConnection::db().await;

    let mut project_model: ProjectModel = Self::get(id.clone()).await?.into();

    if let Some(name) = patch.name {
      project_model.name = name;
    }

    if let Some(working_directories) = patch.working_directories {
      project_model.working_directories = working_directories;
    }

    project_model.updated_at = Utc::now();

    let _: Record = db
      .update(id.inner())
      .merge(project_model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Project })?;

    Self::get(id).await
  }

  pub async fn delete(id: ProjectId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Project,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Project })?;

    Ok(())
  }
}
