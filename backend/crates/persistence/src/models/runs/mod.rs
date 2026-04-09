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
use crate::prelude::UsageMetrics;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunModel {
  pub employee_id:  EmployeeId,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  #[serde(default)]
  pub usage:        Option<UsageMetrics>,
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
      usage:        None,
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
  #[serde(default)]
  pub usage:        Option<UsageMetrics>,
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
  #[serde(default)]
  pub usage:        Option<UsageMetrics>,
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
  #[serde(default)]
  pub usage:        Option<UsageMetrics>,
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
      usage:        record.usage,
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
    let mut model = model;
    model.usage = None;
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
    let usage = rollup_turn_usage(turns.as_slice());

    Ok(RunRecord {
      id: record.id,
      employee_id: record.employee_id,
      status: record.status,
      trigger: record.trigger,
      turns,
      usage: Some(usage),
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
      let usage = rollup_turn_usage(turns.as_slice());
      hydrated.push(RunRecord {
        id: record.id,
        employee_id: record.employee_id,
        status: record.status,
        trigger: record.trigger,
        turns,
        usage: Some(usage),
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
    if filter.issue.is_some() && filter.employee.is_none() && filter.status.is_none() && filter.trigger.is_none() {
      return Self::list_issue_summaries(filter.issue.unwrap(), limit, offset).await;
    }

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

  async fn list_issue_summaries(
    issue_id: crate::prelude::IssueId,
    limit: Option<usize>,
    offset: Option<usize>,
  ) -> DatabaseResult<Vec<RunSummaryRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<RunSummaryRecord> = db
      .query(format!("SELECT * FROM {RUNS_TABLE} ORDER BY created_at DESC"))
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

    let filtered = records.into_iter().filter(|record| match &record.trigger {
      RunTrigger::IssueAssignment { issue_id: run_issue_id }
      | RunTrigger::IssueMention { issue_id: run_issue_id, .. } => run_issue_id == &issue_id,
      RunTrigger::Manual | RunTrigger::Conversation | RunTrigger::Timer | RunTrigger::Dreaming => false,
    });

    let mut filtered: Vec<RunSummaryRecord> = filtered.collect();

    if let Some(offset) = offset {
      filtered = filtered.into_iter().skip(offset).collect();
    }

    if let Some(limit) = limit {
      filtered.truncate(limit);
    }

    Ok(filtered)
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
    let txn = db.clone().begin().await.map_err(|e| DatabaseError::Operation {
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
    let txn = db.clone().begin().await.map_err(|e| DatabaseError::Operation {
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
    let turns = load_turns_for_run(&db, &id).await?;
    model.usage = Some(rollup_turn_usage(turns.as_slice()));

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

  pub async fn delete(id: RunId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Run,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Run })?;

    Ok(())
  }
}

fn rollup_turn_usage(turns: &[TurnRecord]) -> UsageMetrics {
  let mut usage = UsageMetrics::default();
  let mut input_tokens = 0u64;
  let mut output_tokens = 0u64;
  let mut total_tokens = 0u64;
  let mut estimated_cost_usd = 0.0f64;
  let mut has_input = false;
  let mut has_output = false;
  let mut has_total = false;
  let mut has_cost = false;

  for turn in turns {
    let turn_usage = &turn.usage;
    if usage.provider.is_none() {
      usage.provider = turn_usage.provider;
    }
    if usage.model.is_none() {
      usage.model = turn_usage.model.clone();
    }

    if let Some(value) = turn_usage.input_tokens {
      input_tokens = input_tokens.saturating_add(value);
      has_input = true;
    }
    if let Some(value) = turn_usage.output_tokens {
      output_tokens = output_tokens.saturating_add(value);
      has_output = true;
    }
    if let Some(value) = turn_usage.total_tokens {
      total_tokens = total_tokens.saturating_add(value);
      has_total = true;
    }
    if let Some(value) = turn_usage.estimated_cost_usd {
      estimated_cost_usd += value;
      has_cost = true;
    }

    usage.has_unavailable_token_data |= turn_usage.has_unavailable_token_data;
    usage.has_unavailable_cost_data |= turn_usage.has_unavailable_cost_data;
  }

  usage.input_tokens = has_input.then_some(input_tokens);
  usage.output_tokens = has_output.then_some(output_tokens);
  usage.total_tokens = has_total.then_some(total_tokens);
  usage.estimated_cost_usd = has_cost.then_some(estimated_cost_usd);
  usage
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

  if let Some(issue) = filter.issue {
    let verb = if query.contains("WHERE") { "AND" } else { "WHERE" };
    query.push_str(&format!(
      " {verb} (trigger.issue_assignment.issue_id = $issue OR trigger.issue_mention.issue_id = $issue)"
    ));
    binds.push(RunBind::Issue(issue));
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
