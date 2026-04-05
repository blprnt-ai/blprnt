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
use crate::prelude::RUNS_TABLE;
use crate::prelude::ReasoningEffort;
use crate::prelude::Record;
use crate::prelude::RunId;
use crate::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TurnModel {
  pub run_id:           RunId,
  #[serde(default)]
  pub reasoning_effort: Option<ReasoningEffort>,
  pub steps:            Vec<TurnStep>,
  #[serde(default)]
  pub usage:            UsageMetrics,
  pub created_at:       DateTime<Utc>,
  pub updated_at:       DateTime<Utc>,
}

impl Default for TurnModel {
  fn default() -> Self {
    Self {
      run_id:           RunId(SurrealId::default()),
      reasoning_effort: None,
      steps:            Vec::new(),
      usage:            UsageMetrics::default(),
      created_at:       Utc::now(),
      updated_at:       Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TurnRecord {
  pub id:               TurnId,
  pub run_id:           RunId,
  #[serde(default)]
  pub reasoning_effort: Option<ReasoningEffort>,
  pub steps:            Vec<TurnStep>,
  #[serde(default)]
  pub usage:            UsageMetrics,
  pub created_at:       DateTime<Utc>,
  pub updated_at:       DateTime<Utc>,
}

impl From<TurnRecord> for TurnModel {
  fn from(record: TurnRecord) -> Self {
    Self {
      run_id:           record.run_id,
      reasoning_effort: record.reasoning_effort,
      steps:            record.steps,
      usage:            record.usage,
      created_at:       record.created_at,
      updated_at:       record.updated_at,
    }
  }
}

impl TurnModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TURNS_TABLE} SCHEMALESS;")).await?;

    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS run_id ON TABLE {TURNS_TABLE} TYPE record<{RUNS_TABLE}> REFERENCE ON DELETE CASCADE;"
    ))
    .await?;

    Ok(())
  }
}

pub struct TurnRepository;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnStepSide {
  Request,
  Response,
}

impl TurnRepository {
  pub async fn create(model: TurnModel) -> DatabaseResult<TurnRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TURNS_TABLE, Uuid::new_v7());
    let mut model = model;
    model.usage = rollup_usage(model.steps.as_slice());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Turn,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Turn })?;

    Self::get(record_id.into()).await
  }

  pub async fn create_step(turn_id: TurnId, step: TurnStep) -> DatabaseResult<TurnRecord> {
    let mut turn = Self::get(turn_id).await?;

    if let Some(steps) = turn.steps.last_mut() {
      steps.completed_at = Some(Utc::now());
    };

    turn.steps.push(step);

    Self::update(turn.id, turn.steps).await
  }

  pub async fn insert_step_content(
    turn_id: TurnId,
    side: TurnStepSide,
    content: TurnStepContent,
  ) -> DatabaseResult<TurnRecord> {
    let mut turn = Self::get(turn_id).await?;

    match side {
      TurnStepSide::Request => match turn.steps.last_mut() {
        Some(last_step) if last_step.response.contents.is_empty() => {
          last_step.request.contents.push(content);
        }
        _ => {
          turn.steps.push(TurnStep {
            request:      TurnStepContents { contents: vec![content], role: TurnStepRole::User },
            response:     TurnStepContents { contents: Vec::new(), role: TurnStepRole::Assistant },
            status:       TurnStepStatus::InProgress,
            usage:        UsageMetrics::default(),
            created_at:   Utc::now(),
            completed_at: None,
          });
        }
      },
      TurnStepSide::Response => {
        if let Some(last_step) = turn.steps.last_mut() {
          last_step.response.contents.push(content);
        } else {
          turn.steps.push(TurnStep {
            request:      TurnStepContents { contents: Vec::new(), role: TurnStepRole::User },
            response:     TurnStepContents { contents: vec![content], role: TurnStepRole::Assistant },
            status:       TurnStepStatus::InProgress,
            usage:        UsageMetrics::default(),
            created_at:   Utc::now(),
            completed_at: None,
          });
        }
      }
    }

    Self::update(turn.id, turn.steps).await
  }

  pub async fn get(id: TurnId) -> DatabaseResult<TurnRecord> {
    let db = SurrealConnection::db().await;
    let record: TurnRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Turn,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Turn })?;
    Ok(record)
  }

  pub async fn update(id: TurnId, steps: Vec<TurnStep>) -> DatabaseResult<TurnRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.clone().begin().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Turn,
      operation: DatabaseOperation::BeginTransaction,
      source:    e.into(),
    })?;

    let mut model: TurnModel = txn
      .select(id.clone().inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Turn,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Turn })?;

    model.steps = steps;
    model.usage = rollup_usage(model.steps.as_slice());
    model.updated_at = Utc::now();

    let _: Record = txn
      .update(id.clone().inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Turn,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Turn })?;

    txn.commit().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Turn,
      operation: DatabaseOperation::CommitTransaction,
      source:    e.into(),
    })?;

    Self::get(id).await
  }
}

pub fn rollup_usage(step_usages: &[TurnStep]) -> UsageMetrics {
  let mut usage = UsageMetrics::default();
  let mut input_tokens = 0u64;
  let mut output_tokens = 0u64;
  let mut total_tokens = 0u64;
  let mut estimated_cost_usd = 0.0f64;
  let mut has_input = false;
  let mut has_output = false;
  let mut has_total = false;
  let mut has_cost = false;

  for step in step_usages {
    let step_usage = &step.usage;
    if usage.provider.is_none() {
      usage.provider = step_usage.provider;
    }
    if usage.model.is_none() {
      usage.model = step_usage.model.clone();
    }

    if let Some(value) = step_usage.input_tokens {
      input_tokens = input_tokens.saturating_add(value);
      has_input = true;
    }
    if let Some(value) = step_usage.output_tokens {
      output_tokens = output_tokens.saturating_add(value);
      has_output = true;
    }
    if let Some(value) = step_usage.total_tokens {
      total_tokens = total_tokens.saturating_add(value);
      has_total = true;
    }
    if let Some(value) = step_usage.estimated_cost_usd {
      estimated_cost_usd += value;
      has_cost = true;
    }

    usage.has_unavailable_token_data |= step_usage.has_unavailable_token_data;
    usage.has_unavailable_cost_data |= step_usage.has_unavailable_cost_data;
  }

  usage.input_tokens = has_input.then_some(input_tokens);
  usage.output_tokens = has_output.then_some(output_tokens);
  usage.total_tokens = has_total.then_some(total_tokens);
  usage.estimated_cost_usd = has_cost.then_some(estimated_cost_usd);
  usage
}
