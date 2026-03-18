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

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::RUNS_TABLE;
use crate::prelude::Record;
use crate::prelude::RunId;

pub const TURNS_TABLE: &str = "turns";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct TurnId(SurrealId);

impl DbId for TurnId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for TurnId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TURNS_TABLE, uuid).into())
  }
}

impl From<RecordId> for TurnId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct TurnModel {
  pub run:        RunId,
  pub steps:      Vec<TurnStep>,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at: DateTime<Utc>,
}

impl Default for TurnModel {
  fn default() -> Self {
    Self {
      run:        RunId(SurrealId::default()),
      steps:      Vec::new(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct TurnRecord {
  pub id:         TurnId,
  pub run:        RunId,
  pub steps:      Vec<TurnStep>,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at: DateTime<Utc>,
}

impl From<TurnRecord> for TurnModel {
  fn from(record: TurnRecord) -> Self {
    Self {
      run:        record.run,
      steps:      record.steps,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}

impl TurnRecord {
  pub fn run(&self) -> &RunId {
    &self.run
  }

  pub fn turns(&self) -> &Vec<TurnStep> {
    &self.steps
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

impl TurnModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TURNS_TABLE} SCHEMALESS;")).await?;

    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS run ON TABLE {TURNS_TABLE} TYPE record<{RUNS_TABLE}> REFERENCE ON DELETE CASCADE;"
    ))
    .await?;

    Ok(())
  }
}

pub struct TurnRepository;

impl TurnRepository {
  pub async fn create(model: TurnModel) -> Result<TurnRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TURNS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await?
      .ok_or(anyhow::anyhow!("Failed to create employee run turn"))?;

    Self::get(record_id.into()).await
  }

  pub async fn create_step(turn_id: TurnId, step: TurnStep) -> Result<TurnRecord> {
    let mut turn = Self::get(turn_id).await?;

    if let Some(steps) = turn.steps.last_mut() {
      steps.completed_at = Some(Utc::now());
    };

    turn.steps.push(step);

    Self::update(turn.id, turn.steps).await
  }

  pub async fn insert_step_content(
    turn_id: TurnId,
    role: TurnStepRole,
    content: TurnStepContent,
  ) -> Result<TurnRecord> {
    let mut turn = Self::get(turn_id).await?;

    if let Some(steps) = turn.steps.last_mut() {
      if steps.contents.role == role {
        steps.contents.contents.push(content);
      } else {
        steps.completed_at = Some(Utc::now());
        turn.steps.push(TurnStep {
          contents:     TurnStepContents { contents: vec![content], role: role },
          status:       TurnStepStatus::InProgress,
          created_at:   Utc::now(),
          completed_at: None,
        });
      }
    }

    Self::update(turn.id, turn.steps).await
  }

  pub async fn get(id: TurnId) -> Result<TurnRecord> {
    let db = SurrealConnection::db().await;
    let record: TurnRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Employee run turn not found"))?;
    Ok(record)
  }

  pub async fn update(id: TurnId, steps: Vec<TurnStep>) -> Result<TurnRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await?;

    let mut model: TurnModel =
      txn.select(id.clone().inner()).await?.ok_or(anyhow::anyhow!("Employee run turn not found"))?;

    model.steps = steps;
    model.updated_at = Utc::now();

    let _: Option<Record> = txn.update(id.clone().inner()).merge(model).await?;

    txn.commit().await?;

    Self::get(id).await
  }
}
