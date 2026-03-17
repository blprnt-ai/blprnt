mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::Record;

pub const EMPLOYEE_RUN_TURNS_TABLE: &str = "employee_run_turns";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRunTurnModel {
  pub employee_run: SurrealId,
  pub steps:        Vec<TurnStep>,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:   DateTime<Utc>,
}

impl Default for EmployeeRunTurnModel {
  fn default() -> Self {
    Self {
      employee_run: SurrealId::default(),
      steps:        Vec::new(),
      created_at:   Utc::now(),
      updated_at:   Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRunTurnRecord {
  pub id:           SurrealId,
  pub employee_run: SurrealId,
  pub steps:        Vec<TurnStep>,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:   DateTime<Utc>,
}

impl From<EmployeeRunTurnRecord> for EmployeeRunTurnModel {
  fn from(record: EmployeeRunTurnRecord) -> Self {
    Self {
      employee_run: record.employee_run,
      steps:        record.steps,
      created_at:   record.created_at,
      updated_at:   record.updated_at,
    }
  }
}

impl EmployeeRunTurnRecord {
  pub fn employee_run(&self) -> &SurrealId {
    &self.employee_run
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

impl EmployeeRunTurnModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE TABLE IF NOT EXISTS employee_run_turns SCHEMALESS;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS employee_run ON TABLE employee_run_turns TYPE record<employee_runs> REFERENCE ON DELETE CASCADE;
      "#,
    )
    .await?;

    Ok(())
  }
}

pub struct EmployeeRunTurnRepository;

impl EmployeeRunTurnRepository {
  pub async fn create(model: EmployeeRunTurnModel) -> Result<EmployeeRunTurnRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(EMPLOYEE_RUN_TURNS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await?
      .ok_or(anyhow::anyhow!("Failed to create employee run turn"))?;

    Self::get(record_id.into()).await
  }

  pub async fn create_step(employee_run: SurrealId, step: TurnStep) -> Result<EmployeeRunTurnRecord> {
    let mut turn = Self::get(employee_run).await?;

    if let Some(steps) = turn.steps.last_mut() {
      steps.completed_at = Some(Utc::now());
    };

    turn.steps.push(step);

    Self::update(turn.id, turn.steps).await
  }

  pub async fn insert_step_content(
    employee_run: SurrealId,
    role: TurnStepRole,
    content: TurnStepContent,
  ) -> Result<EmployeeRunTurnRecord> {
    let mut turn = Self::get(employee_run).await?;

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

  pub async fn get(id: SurrealId) -> Result<EmployeeRunTurnRecord> {
    let db = SurrealConnection::db().await;
    let record: EmployeeRunTurnRecord =
      db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Employee run turn not found"))?;
    Ok(record)
  }

  pub async fn list(employee: SurrealId) -> Result<Vec<EmployeeRunTurnRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<EmployeeRunTurnRecord> = db
      .query("SELECT * FROM employee_run_turns WHERE employee = $employee")
      .bind(("employee", employee.inner()))
      .await?
      .take(0)?;
    Ok(records)
  }

  pub async fn list_turns(employee_run: SurrealId) -> Result<Vec<EmployeeRunTurnRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<EmployeeRunTurnRecord> = db
      .query("SELECT * FROM employee_run_turns WHERE employee_run = $employee_run")
      .bind(("employee_run", employee_run.inner()))
      .await?
      .take(0)?;
    Ok(records)
  }

  pub async fn update(id: SurrealId, steps: Vec<TurnStep>) -> Result<EmployeeRunTurnRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.begin().await?;

    let mut model: EmployeeRunTurnModel =
      txn.select(id.inner()).await?.ok_or(anyhow::anyhow!("Employee run turn not found"))?;

    model.steps = steps;
    model.updated_at = Utc::now();

    let _: Option<Record> = txn.update(id.inner()).merge(model).await?;

    txn.commit().await?;

    Self::get(id).await
  }
}
