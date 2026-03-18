use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::prelude::EmployeeId;
use crate::prelude::ISSUES_TABLE;
use crate::prelude::IssueId;
use crate::prelude::RunId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueActionModel {
  pub issue:       IssueId,
  pub action_kind: IssueActionKind,
  pub actor:       EmployeeId,
  pub source:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl IssueActionModel {
  pub fn new(issue: IssueId, action_kind: IssueActionKind, actor: EmployeeId, source: Option<RunId>) -> Self {
    Self { issue, action_kind, created_at: Utc::now(), actor, source }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueActionRecord {
  pub id:          IssueActionId,
  pub issue:       IssueId,
  pub action_kind: IssueActionKind,
  pub actor:       EmployeeId,
  pub source:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl From<IssueActionRecord> for IssueActionModel {
  fn from(record: IssueActionRecord) -> Self {
    Self {
      issue:       record.issue,
      action_kind: record.action_kind,
      actor:       record.actor,
      source:      record.source,
      created_at:  record.created_at,
    }
  }
}

impl IssueActionModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_ACTIONS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue ON TABLE {ISSUE_ACTIONS_TABLE} TYPE option<record<{ISSUES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
