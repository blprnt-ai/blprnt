use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::RunId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueCommentModel {
  pub issue_id:   IssueId,
  pub comment:    String,
  pub creator:    EmployeeId,
  pub run_id:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl IssueCommentModel {
  pub fn new(issue_id: IssueId, comment: String, creator: EmployeeId, run_id: Option<RunId>) -> Self {
    Self { issue_id, comment, creator, run_id, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueCommentRecord {
  pub id:         IssueCommentId,
  pub issue_id:   IssueId,
  pub comment:    String,
  pub creator:    EmployeeId,
  pub run_id:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueCommentRecord> for IssueCommentModel {
  fn from(record: IssueCommentRecord) -> Self {
    Self {
      issue_id:   record.issue_id,
      comment:    record.comment,
      creator:    record.creator,
      run_id:     record.run_id,
      created_at: record.created_at,
    }
  }
}

impl IssueCommentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_COMMENTS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue_id ON TABLE {ISSUE_COMMENTS_TABLE} TYPE record<{ISSUES_TABLE}> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
