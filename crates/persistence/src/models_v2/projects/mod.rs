mod types;
use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::Record;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectModelV2 {
  pub name:                String,
  pub working_directories: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_primer:        Option<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl ProjectModelV2 {
  pub fn new(name: String, working_directories: Vec<String>, agent_primer: Option<String>) -> Self {
    Self {
      name:                name,
      working_directories: working_directories,
      agent_primer:        agent_primer,
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_primer:        Option<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl From<ProjectRecord> for ProjectModelV2 {
  fn from(record: ProjectRecord) -> Self {
    Self {
      name:                record.name,
      working_directories: record.working_directories,
      agent_primer:        record.agent_primer,
      created_at:          record.created_at,
      updated_at:          record.updated_at,
    }
  }
}

impl ProjectModelV2 {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {PROJECTS_TABLE} SCHEMALESS;")).await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectPatchV2 {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:                Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_directories: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_primer:        Option<Option<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:          Option<DateTime<Utc>>,
}

pub struct ProjectRepositoryV2;

impl ProjectRepositoryV2 {
  pub async fn create(model: ProjectModelV2) -> Result<ProjectRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(PROJECTS_TABLE, Uuid::new_v7());

    db.create(record_id).content(model).await?.ok_or(anyhow::anyhow!("Failed to create project"))
  }

  pub async fn get(id: ProjectId) -> Result<ProjectRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Project not found"))
  }

  pub async fn list() -> Result<Vec<ProjectRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {PROJECTS_TABLE}.*")).await?.take(0).context("Failed to list projects")
  }

  pub async fn update(id: ProjectId, patch: ProjectPatchV2) -> Result<ProjectRecord> {
    let db = SurrealConnection::db().await;

    let mut project_model: ProjectModelV2 = Self::get(id.clone()).await?.into();
    project_model.updated_at = Utc::now();

    if let Some(name) = patch.name {
      project_model.name = name;
    }

    if let Some(working_directories) = patch.working_directories {
      project_model.working_directories = working_directories;
    }

    if let Some(agent_primer) = patch.agent_primer {
      project_model.agent_primer = agent_primer;
    }

    project_model.updated_at = Utc::now();

    let _: Option<Record> = db.update(id.inner()).merge(project_model).await?;

    Self::get(id).await
  }

  pub async fn delete(id: ProjectId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete project"))?;

    Ok(())
  }
}
