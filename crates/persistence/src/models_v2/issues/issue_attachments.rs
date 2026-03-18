use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::SurrealValue;

pub use super::types::*;
use crate::connection::DbConnection;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::RunId;
use crate::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueAttachmentModel {
  pub issue:      IssueId,
  pub attachment: IssueAttachment,
  pub actor:      Option<EmployeeId>,
  pub source:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl Default for IssueAttachmentModel {
  fn default() -> Self {
    Self {
      issue:      IssueId(SurrealId::default()),
      attachment: IssueAttachment::default(),
      actor:      None,
      source:     None,
      created_at: Utc::now(),
    }
  }
}

impl From<(IssueId, IssueAttachment)> for IssueAttachmentModel {
  fn from((issue, attachment): (IssueId, IssueAttachment)) -> Self {
    Self { issue: issue, attachment: attachment, actor: None, source: None, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueAttachmentRecord {
  pub id:         IssueAttachmentId,
  pub issue:      IssueId,
  pub attachment: IssueAttachment,
  pub actor:      Option<EmployeeId>,
  pub source:     Option<RunId>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentModel {
  fn from(record: IssueAttachmentRecord) -> Self {
    Self {
      issue:      record.issue,
      attachment: record.attachment,
      actor:      record.actor,
      source:     record.source,
      created_at: record.created_at,
    }
  }
}

impl IssueAttachmentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_ATTACHMENTS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE option<record<{ISSUES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS actor ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE option<record<{EMPLOYEES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
