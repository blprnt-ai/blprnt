mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use super::EmployeeRunTurnModel;
use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::Record;

pub const EMPLOYEE_RUNS_TABLE: &str = "employee_runs";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRunModel {
  pub employee:     SurrealId,
  pub status:       EmployeeRunStatus,
  pub trigger:      EmployeeRunTurnTrigger,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub started_at:   Option<DateTime<Utc>>,
  #[specta(type = i32)]
  pub completed_at: Option<DateTime<Utc>>,
}

impl EmployeeRunModel {
  pub fn new(employee: SurrealId) -> Self {
    Self {
      employee,
      status: EmployeeRunStatus::Pending,
      trigger: EmployeeRunTurnTrigger::Manual,
      created_at: Utc::now(),
      started_at: None,
      completed_at: None,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRunRecord {
  pub id:           SurrealId,
  pub employee:     SurrealId,
  pub status:       EmployeeRunStatus,
  pub trigger:      EmployeeRunTurnTrigger,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub started_at:   Option<DateTime<Utc>>,
  #[specta(type = i32)]
  pub completed_at: Option<DateTime<Utc>>,
}

impl From<EmployeeRunRecord> for EmployeeRunModel {
  fn from(record: EmployeeRunRecord) -> Self {
    Self {
      employee:     record.employee,
      status:       record.status,
      trigger:      record.trigger,
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

impl EmployeeRunRecord {
  pub fn employee(&self) -> &SurrealId {
    &self.employee
  }

  pub fn status(&self) -> &EmployeeRunStatus {
    &self.status
  }

  pub fn trigger(&self) -> &EmployeeRunTurnTrigger {
    &self.trigger
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn started_at(&self) -> &Option<DateTime<Utc>> {
    &self.started_at
  }

  pub fn completed_at(&self) -> &Option<DateTime<Utc>> {
    &self.completed_at
  }
}

impl EmployeeRunModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    EmployeeRunTurnModel::migrate(db).await?;

    db.query(
      r#"
      DEFINE TABLE IF NOT EXISTS employee_runs SCHEMALESS;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS employee ON TABLE employee_runs TYPE option<record<employees>> REFERENCE ON DELETE CASCADE;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS turns ON TABLE employee_runs COMPUTED <~employee_run_turns;
      "#,
    )
    .await?;

    Ok(())
  }
}

pub struct EmployeeRunRepository;

impl EmployeeRunRepository {
  pub async fn create(model: EmployeeRunModel) -> Result<EmployeeRunRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(EMPLOYEE_RUNS_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create employee run"))?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: SurrealId) -> Result<EmployeeRunRecord> {
    let db = SurrealConnection::db().await;
    let record: EmployeeRunRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Employee run not found"))?;
    Ok(record)
  }

  pub async fn list(employee: SurrealId) -> Result<Vec<EmployeeRunRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<EmployeeRunRecord> = db
      .query("SELECT * FROM employee_runs WHERE employee = $employee")
      .bind(("employee", employee.inner()))
      .await?
      .take(0)?;
    Ok(records)
  }

  pub async fn update(id: SurrealId, status: EmployeeRunStatus) -> Result<EmployeeRunRecord> {
    let db = SurrealConnection::db().await;
    let mut model: EmployeeRunModel = Self::get(id.clone()).await?.into();

    if matches!(status, EmployeeRunStatus::Running) {
      model.started_at = Some(Utc::now());
    } else if matches!(status, EmployeeRunStatus::Completed | EmployeeRunStatus::Failed) {
      model.completed_at = Some(Utc::now());
    }

    model.status = status;

    let _: Option<Record> = db.update(id.inner()).merge(model).await?;

    Self::get(id).await
  }
}
