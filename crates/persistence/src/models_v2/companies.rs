use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::ISSUES_TABLE;
use crate::prelude::PROJECTS_TABLE;
use crate::prelude::ProjectId;
use crate::prelude::Record;

pub const COMPANIES_TABLE: &str = "companies";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct CompanyId(SurrealId);

impl DbId for CompanyId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for CompanyId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(COMPANIES_TABLE, uuid).into())
  }
}

impl From<RecordId> for CompanyId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct CompanyModel {
  pub name:                  String,
  pub description:           String,
  pub issue_prefix:          String,
  pub issue_count:           i32,
  pub require_hire_approval: bool,
  #[specta(type = i32)]
  pub created_at:            DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:            DateTime<Utc>,
}

impl Default for CompanyModel {
  fn default() -> Self {
    Self {
      name:                  String::new(),
      description:           String::new(),
      issue_prefix:          String::new(),
      issue_count:           0,
      require_hire_approval: false,
      created_at:            Utc::now(),
      updated_at:            Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct CompanyRecord {
  pub id:                    CompanyId,
  pub name:                  String,
  pub description:           String,
  pub issue_prefix:          String,
  pub issue_count:           i32,
  pub require_hire_approval: bool,
  #[specta(type = i32)]
  pub created_at:            DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:            DateTime<Utc>,
  pub employees:             Vec<EmployeeId>,
  pub projects:              Vec<ProjectId>,
}

impl From<CompanyRecord> for CompanyModel {
  fn from(record: CompanyRecord) -> Self {
    Self {
      name:                  record.name,
      description:           record.description,
      issue_prefix:          record.issue_prefix,
      issue_count:           record.issue_count,
      require_hire_approval: record.require_hire_approval,
      created_at:            record.created_at,
      updated_at:            record.updated_at,
    }
  }
}

impl CompanyRecord {
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn description(&self) -> &String {
    &self.description
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }

  pub fn employees(&self) -> &Vec<EmployeeId> {
    &self.employees
  }

  pub fn projects(&self) -> &Vec<ProjectId> {
    &self.projects
  }
}

impl CompanyModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {COMPANIES_TABLE} SCHEMALESS;")).await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS employees ON TABLE {COMPANIES_TABLE} COMPUTED <~{EMPLOYEES_TABLE};"))
      .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS projects ON TABLE {COMPANIES_TABLE} COMPUTED <~{PROJECTS_TABLE};"))
      .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS issues ON TABLE {COMPANIES_TABLE} COMPUTED <~{ISSUES_TABLE};"))
      .await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct CompanyPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:                  Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:           Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub issue_count:           Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub require_hire_approval: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at:            Option<DateTime<Utc>>,
}

pub struct CompanyRepository;

impl CompanyRepository {
  pub async fn create(model: CompanyModel) -> Result<CompanyRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(COMPANIES_TABLE, Uuid::new_v7());
    let record: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create company"))?;

    Self::get(record.id.into()).await
  }

  pub async fn get(id: CompanyId) -> Result<CompanyRecord> {
    let db = SurrealConnection::db().await;
    let record: CompanyRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Company not found"))?;
    Ok(record)
  }

  pub async fn list() -> Result<Vec<CompanyRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<CompanyRecord> = db.query(format!("SELECT * FROM {COMPANIES_TABLE}")).await?.take(0)?;
    Ok(records)
  }

  pub async fn update(id: CompanyId, patch: CompanyPatch) -> Result<CompanyRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await?;
    let mut model: CompanyRecord = txn.select(id.clone().inner()).await?.ok_or(anyhow::anyhow!("Company not found"))?;

    if let Some(name) = patch.name {
      model.name = name;
    }

    if let Some(description) = patch.description {
      model.description = description;
    }

    if let Some(issue_count) = patch.issue_count {
      model.issue_count = issue_count;
    }

    if let Some(require_hire_approval) = patch.require_hire_approval {
      model.require_hire_approval = require_hire_approval;
    }

    model.updated_at = Utc::now();

    let _: Record =
      txn.update(id.clone().inner()).merge(model).await?.ok_or(anyhow::anyhow!("Failed to update company"))?;

    txn.commit().await?;

    Self::get(id).await
  }

  pub async fn delete(id: CompanyId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete company"))?;
    Ok(())
  }
}
