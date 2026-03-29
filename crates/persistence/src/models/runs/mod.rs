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
use crate::prelude::TURNS_TABLE;
use crate::prelude::TurnRecord;

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunSummaryRecord {
  pub id:           RunId,
  pub employee_id:  EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub created_at:   DateTime<Utc>,
  pub started_at:   Option<DateTime<Utc>>,
  pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
struct RunDocument {
  pub id:           RunId,
  pub employee_id:  EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
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
    let record: RunDocument = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Run })?;

    let turns = load_turns_for_run(&db, &record.id).await?;

    Ok(RunRecord {
      id: record.id,
      employee_id: record.employee_id,
      status: record.status,
      trigger: record.trigger,
      turns,
      created_at: record.created_at,
      started_at: record.started_at,
      completed_at: record.completed_at,
    })
  }

  pub async fn list(filter: RunFilter) -> DatabaseResult<Vec<RunRecord>> {
    let db = SurrealConnection::db().await;

    let (query, binds) = build_run_query("SELECT *", filter, Some("ORDER BY created_at DESC"), None, None);

    let mut query = db.query(query);
    for bind in binds {
      query = query.bind(bind.into_bind_value());
    }

    let records: Vec<RunDocument> = query
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

    let mut hydrated = Vec::with_capacity(records.len());
    for record in records {
      let turns = load_turns_for_run(&db, &record.id).await?;
      hydrated.push(RunRecord {
        id: record.id,
        employee_id: record.employee_id,
        status: record.status,
        trigger: record.trigger,
        turns,
        created_at: record.created_at,
        started_at: record.started_at,
        completed_at: record.completed_at,
      });
    }

    Ok(hydrated)
  }

  pub async fn list_summaries(
    filter: RunFilter,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> DatabaseResult<Vec<RunSummaryRecord>> {
    let db = SurrealConnection::db().await;
    let (query, binds) = build_run_query("SELECT *", filter, Some("ORDER BY created_at DESC"), limit, offset);

    let mut query = db.query(query);
    for bind in binds {
      query = query.bind(bind.into_bind_value());
    }

    query
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
      })
  }

  pub async fn count(filter: RunFilter) -> DatabaseResult<u64> {
    let db = SurrealConnection::db().await;
    let (query, binds) = build_run_query("SELECT count() AS count", filter, None, None, None);

    let mut query = db.query(query);
    for bind in binds {
      query = query.bind(bind.into_bind_value());
    }

    let counts: Vec<CountRow> = query
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

    Ok(counts.first().map(|row| row.count).unwrap_or(0))
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

    let mut model: RunModel = txn
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
    } else if matches!(status, RunStatus::Completed | RunStatus::Cancelled | RunStatus::Failed(_)) {
      model.completed_at = Some(Utc::now());
    }

    model.status = status;

    let _: Option<Record> = txn
      .update(id.clone().inner())
      .content(model)
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
struct CountRow {
  count: u64,
}

fn build_run_query(
  select: &str,
  filter: RunFilter,
  order_clause: Option<&str>,
  limit: Option<usize>,
  offset: Option<usize>,
) -> (String, Vec<RunBind>) {
  let mut query = format!("{select} FROM {RUNS_TABLE}");
  let mut binds: Vec<RunBind> = vec![];

  if let Some(employee) = filter.employee {
    query.push_str(" WHERE employee_id = $employee");
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

  if let Some(order_clause) = order_clause {
    query.push(' ');
    query.push_str(order_clause);
  }

  if let Some(limit) = limit {
    query.push_str(&format!(" LIMIT {limit}"));
  }

  if let Some(offset) = offset {
    query.push_str(&format!(" START {offset}"));
  }

  (query, binds)
}

async fn load_turns_for_run(db: &DbConnection, run_id: &RunId) -> DatabaseResult<Vec<TurnRecord>> {
  db.query(format!("SELECT * FROM {TURNS_TABLE} WHERE run_id = $run_id ORDER BY created_at ASC"))
    .bind(("run_id", run_id.clone()))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Turn,
      operation: DatabaseOperation::List,
      source:    e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Turn,
      operation: DatabaseOperation::List,
      source:    e.into(),
    })
}
