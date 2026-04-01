use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::ISSUES_TABLE;
use crate::prelude::IssueId;
use crate::prelude::RUNS_TABLE;
use crate::prelude::RunId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueActionModel {
  pub issue_id:    IssueId,
  pub action_kind: IssueActionKind,
  pub creator:     EmployeeId,
  pub run_id:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl IssueActionModel {
  pub fn new(issue_id: IssueId, action_kind: IssueActionKind, creator: EmployeeId, run_id: Option<RunId>) -> Self {
    Self { issue_id, action_kind, created_at: Utc::now(), creator, run_id }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueActionRecord {
  pub id:          IssueActionId,
  pub issue_id:    IssueId,
  pub action_kind: IssueActionKind,
  pub creator:     EmployeeId,
  pub run_id:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl From<IssueActionRecord> for IssueActionModel {
  fn from(record: IssueActionRecord) -> Self {
    Self {
      issue_id:    record.issue_id,
      action_kind: record.action_kind,
      creator:     record.creator,
      run_id:      record.run_id,
      created_at:  record.created_at,
    }
  }
}

impl IssueActionModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_ACTIONS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue_id ON TABLE {ISSUE_ACTIONS_TABLE} TYPE option<record<{ISSUES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    db.query(
    format!("DEFINE FIELD IF NOT EXISTS creator ON TABLE {ISSUE_ACTIONS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE UNSET;"),
  )
  .await?;

    db.query(
    format!("DEFINE FIELD IF NOT EXISTS run_id ON TABLE {ISSUE_ACTIONS_TABLE} TYPE option<record<{RUNS_TABLE}>> REFERENCE ON DELETE UNSET;"),
  )
  .await?;

    Ok(())
  }
}
