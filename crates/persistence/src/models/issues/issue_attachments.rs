use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::models::RUNS_TABLE;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::RunId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueAttachmentModel {
  pub issue_id:   IssueId,
  pub attachment: IssueAttachment,
  pub creator:    EmployeeId,
  pub run_id:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl IssueAttachmentModel {
  pub fn new(issue_id: IssueId, attachment: IssueAttachment, creator: EmployeeId, run_id: Option<RunId>) -> Self {
    Self { issue_id, attachment, creator, run_id, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueAttachmentRecord {
  pub id:         IssueAttachmentId,
  pub issue_id:   IssueId,
  pub attachment: IssueAttachment,
  pub creator:    EmployeeId,
  pub run_id:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentModel {
  fn from(record: IssueAttachmentRecord) -> Self {
    Self {
      issue_id:   record.issue_id,
      attachment: record.attachment,
      creator:    record.creator,
      run_id:     record.run_id,
      created_at: record.created_at,
    }
  }
}

impl IssueAttachmentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_ATTACHMENTS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue_id ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE record<{ISSUES_TABLE}> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS creator ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS run_id ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE option<record<{RUNS_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
