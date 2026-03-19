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

use super::TurnModel;
use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::Record;
use crate::prelude::SurrealId;
use crate::prelude::TURNS_TABLE;
use crate::prelude::TurnRecord;

pub const RUNS_TABLE: &str = "runs";

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunId(pub SurrealId);

impl DbId for RunId {
  fn id(&self) -> SurrealId {
    self.0.clone()
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunModel {
  pub employee_id:  EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub created_at:   DateTime<Utc>,
  pub started_at:   Option<DateTime<Utc>>,
  pub completed_at: Option<DateTime<Utc>>,
}

impl RunModel {
  pub fn new(employee: EmployeeId, trigger: RunTrigger) -> Self {
    Self {
      employee_id:  employee,
      status:       RunStatus::Pending,
      trigger:      trigger,
      created_at:   Utc::now(),
      started_at:   None,
      completed_at: None,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunRecord {
  pub id:           RunId,
  pub employee_id:  EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub turns:        Vec<TurnRecord>,
  pub created_at:   DateTime<Utc>,
  pub started_at:   Option<DateTime<Utc>>,
  pub completed_at: Option<DateTime<Utc>>,
}

impl From<RunRecord> for RunModel {
  fn from(record: RunRecord) -> Self {
    Self {
      employee_id:  record.employee_id,
      status:       record.status,
      trigger:      record.trigger,
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

impl RunModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    TurnModel::migrate(db).await?;

    db.query(format!("DEFINE TABLE IF NOT EXISTS {RUNS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS employee_id ON TABLE {RUNS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE CASCADE;"),
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
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Run })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: RunId) -> DatabaseResult<RunRecord> {
    let db = SurrealConnection::db().await;
    let record: RunRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Run })?;
    Ok(record)
  }

  pub async fn list(filter: RunFilter) -> DatabaseResult<Vec<RunRecord>> {
    let db = SurrealConnection::db().await;

    let mut query = format!("SELECT * FROM {RUNS_TABLE}");

    let mut binds: Vec<RunBind> = vec![];

    if let Some(employee) = filter.employee {
      query.push_str(&format!(" WHERE employee = $employee"));
      binds.push(RunBind::Employee(employee));
    }

    if let Some(status) = filter.status {
      let verb = if query.contains("WHERE") { "AND" } else { "WHERE" };
      query.push_str(&format!(" {verb} status = $status"));
      binds.push(RunBind::Status(status));
    }

    if let Some(trigger) = filter.trigger {
      let verb = if query.contains("WHERE") { "AND" } else { "WHERE" };
      query.push_str(&format!(" {verb} trigger = $trigger"));
      binds.push(RunBind::Trigger(trigger));
    }

    let mut query = db.query(query);
    for bind in binds {
      query = query.bind(bind.into_bind_value());
    }

    let records: Vec<RunRecord> = query
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;

    Ok(records)
  }

  pub async fn mark_all_pending_as_failed(failure_reason: String) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Run,
      operation: DatabaseOperation::BeginTransaction,
      source:    e.into(),
    })?;

    let _ = txn
      .query(format!("UPDATE {RUNS_TABLE} SET status = $failed, completed_at = $completed_at WHERE status = $pending"))
      .bind(("failed", RunStatus::Failed(failure_reason)))
      .bind(("completed_at", Utc::now()))
      .bind(("pending", RunStatus::Pending))
      .await;

    txn.commit().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Run,
      operation: DatabaseOperation::CommitTransaction,
      source:    e.into(),
    })?;

    Ok(())
  }

  pub async fn update(id: RunId, status: RunStatus) -> DatabaseResult<RunRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Run,
      operation: DatabaseOperation::BeginTransaction,
      source:    e.into(),
    })?;

    let mut model: RunRecord = txn
      .select(id.clone().inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Run })?;

    if matches!(status, RunStatus::Running) {
      model.started_at = Some(Utc::now());
    } else if matches!(status, RunStatus::Completed | RunStatus::Failed(_)) {
      model.completed_at = Some(Utc::now());
    }

    model.status = status;

    let _: Option<Record> = txn
      .update(id.clone().inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Run })?;

    txn.commit().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Run,
      operation: DatabaseOperation::CommitTransaction,
      source:    e.into(),
    })?;

    Self::get(id).await
  }
}
