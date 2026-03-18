use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::RunId;
use crate::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueCommentModel {
  pub issue:      IssueId,
  pub comment:    String,
  pub creator:    Option<EmployeeId>,
  pub run:        Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl Default for IssueCommentModel {
  fn default() -> Self {
    Self {
      issue:      IssueId(SurrealId::default()),
      comment:    String::new(),
      creator:    None,
      run:        None,
      created_at: Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueCommentRecord {
  pub id:         IssueCommentId,
  pub issue:      IssueId,
  pub comment:    String,
  pub creator:    Option<EmployeeId>,
  pub run:        Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueCommentRecord> for IssueCommentModel {
  fn from(record: IssueCommentRecord) -> Self {
    Self {
      issue:      record.issue,
      comment:    record.comment,
      creator:    record.creator,
      run:        record.run,
      created_at: record.created_at,
    }
  }
}

impl IssueCommentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_COMMENTS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue ON TABLE {ISSUE_COMMENTS_TABLE} TYPE record<{ISSUES_TABLE}> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
