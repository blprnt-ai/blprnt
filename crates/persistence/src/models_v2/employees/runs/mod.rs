mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use super::TurnModel;
use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::Record;
use crate::prelude::TURNS_TABLE;
use crate::prelude::errors::DatabaseError;
use crate::prelude::errors::DatabaseResult;

pub const RUNS_TABLE: &str = "runs";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct RunId(pub SurrealId);

impl DbId for RunId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for RunId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(RUNS_TABLE, uuid).into())
  }
}

impl From<RecordId> for RunId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct RunModel {
  pub employee:     EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub started_at:   Option<DateTime<Utc>>,
  #[specta(type = i32)]
  pub completed_at: Option<DateTime<Utc>>,
}

impl RunModel {
  pub fn new(employee: EmployeeId) -> Self {
    Self {
      employee,
      status: RunStatus::Pending,
      trigger: RunTrigger::Manual,
      created_at: Utc::now(),
      started_at: None,
      completed_at: None,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct RunRecord {
  pub id:           RunId,
  pub employee:     EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub started_at:   Option<DateTime<Utc>>,
  #[specta(type = i32)]
  pub completed_at: Option<DateTime<Utc>>,
}

impl From<RunRecord> for RunModel {
  fn from(record: RunRecord) -> Self {
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

impl RunRecord {
  pub fn employee(&self) -> &EmployeeId {
    &self.employee
  }

  pub fn status(&self) -> &RunStatus {
    &self.status
  }

  pub fn trigger(&self) -> &RunTrigger {
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

impl RunModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    TurnModel::migrate(db).await?;

    db.query(format!("DEFINE TABLE IF NOT EXISTS {RUNS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS employee ON TABLE {RUNS_TABLE} TYPE option<record<{EMPLOYEES_TABLE}>> REFERENCE ON DELETE CASCADE;"),
    )
    .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS turns ON TABLE {RUNS_TABLE} COMPUTED <~{TURNS_TABLE};")).await?;

    Ok(())
  }
}

pub struct RunRepository;

impl RunRepository {
  pub async fn create(model: RunModel) -> DatabaseResult<RunRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(RUNS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::FailedToCreateRun(e.into()))?
      .ok_or(DatabaseError::RunNotFoundAfterCreation)?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: RunId) -> DatabaseResult<RunRecord> {
    let db = SurrealConnection::db().await;
    let record: RunRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::FailedToGetRun(e.into()))?
      .ok_or(DatabaseError::RunNotFound)?;
    Ok(record)
  }

  pub async fn list(employee: EmployeeId) -> DatabaseResult<Vec<RunRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<RunRecord> = db
      .query(format!("SELECT * FROM {RUNS_TABLE} WHERE employee = $employee"))
      .bind(("employee", employee.inner()))
      .await
      .map_err(|e| DatabaseError::FailedToListRuns(e.into()))?
      .take(0)
      .map_err(|e| DatabaseError::FailedToListRuns(e.into()))?;
    Ok(records)
  }

  pub async fn update(id: RunId, status: RunStatus) -> DatabaseResult<RunRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await.map_err(|e| DatabaseError::FailedToBeginTransaction(e.into()))?;

    let mut model: RunRecord = txn
      .select(id.clone().inner())
      .await
      .map_err(|e| DatabaseError::FailedToGetRun(e.into()))?
      .ok_or(DatabaseError::RunNotFound)?;

    if matches!(status, RunStatus::Running) {
      model.started_at = Some(Utc::now());
    } else if matches!(status, RunStatus::Completed | RunStatus::Failed) {
      model.completed_at = Some(Utc::now());
    }

    model.status = status;

    let _: Option<Record> = txn
      .update(id.clone().inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::FailedToUpdateRun(e.into()))?
      .ok_or(DatabaseError::RunNotFound)?;

    txn.commit().await.map_err(|e| DatabaseError::FailedToCommitTransaction(e.into()))?;

    Self::get(id).await
  }
}
