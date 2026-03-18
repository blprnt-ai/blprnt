use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::PathList;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::COMPANIES_TABLE;
use crate::prelude::Record;
use crate::prelude::SESSIONS_TABLE;

pub const PROJECTS_TABLE: &str = "projects";

// TODO: Replace id with ProjectId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectId(SurrealId);

impl DbId for ProjectId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for ProjectId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(PROJECTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProjectId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectModelV2 {
  pub name:                String,
  pub working_directories: PathList,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_primer:        Option<String>,
  #[specta(type = i32)]
  pub created_at:          DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:          DateTime<Utc>,
}

impl ProjectModelV2 {
  pub fn new(name: String, working_directories: PathList, agent_primer: Option<String>) -> Self {
    Self {
      name:                name,
      working_directories: working_directories,
      agent_primer:        agent_primer,
      created_at:          Utc::now(),
      updated_at:          Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectRecord {
  pub id:              SurrealId,
  name:                String,
  working_directories: PathList,
  #[serde(skip_serializing_if = "Option::is_none")]
  agent_primer:        Option<String>,
  #[specta(type = i32)]
  created_at:          DateTime<Utc>,
  #[specta(type = i32)]
  updated_at:          DateTime<Utc>,
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

impl ProjectRecord {
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn working_directories(&self) -> &PathList {
    &self.working_directories
  }

  pub fn agent_primer(&self) -> &Option<String> {
    &self.agent_primer
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

impl ProjectModelV2 {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS child_sessions ON TABLE {PROJECTS_TABLE} COMPUTED <~{SESSIONS_TABLE};"
    ))
    .await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS company ON TABLE {PROJECTS_TABLE} TYPE option<record<{COMPANIES_TABLE}>> REFERENCE ON DELETE UNSET;")
    )
    .await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectPatchV2 {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:                Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_directories: Option<PathList>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_primer:        Option<Option<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at:          Option<DateTime<Utc>>,
}

pub struct ProjectRepositoryV2;

impl ProjectRepositoryV2 {
  pub async fn create(model: ProjectModelV2) -> Result<ProjectRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(PROJECTS_TABLE, Uuid::new_v7());

    db.create(record_id).content(model).await?.ok_or(anyhow::anyhow!("Failed to create project"))
  }

  pub async fn get(id: SurrealId) -> Result<ProjectRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Project not found"))
  }

  #[deprecated(since = "0.1.0", note = "Use list(company) instead")]
  #[allow(non_snake_case)]
  pub async fn list_LEGACY() -> Result<Vec<ProjectRecord>> {
    let db = SurrealConnection::db().await;

    db.query(format!("SELECT * FROM {}", PROJECTS_TABLE)).await?.take(0).context("Failed to list projects")
  }

  pub async fn list(company: SurrealId) -> Result<Vec<ProjectRecord>> {
    let db = SurrealConnection::db().await;
    db.query("SELECT * FROM $company_id.projects.*")
      .bind(("company_id", company.inner()))
      .await?
      .take(0)
      .context("Failed to list projects")
  }

  pub async fn update(id: SurrealId, patch: ProjectPatchV2) -> Result<ProjectRecord> {
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

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete project"))?;

    Ok(())
  }
}
