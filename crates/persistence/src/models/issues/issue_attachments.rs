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
  pub issue_id:    IssueId,
  pub attachment:  IssueAttachment,
  pub employee_id: Option<EmployeeId>,
  pub source:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl Default for IssueAttachmentModel {
  fn default() -> Self {
    Self {
      issue_id:    IssueId(SurrealId::default()),
      attachment:  IssueAttachment::default(),
      employee_id: None,
      source:      None,
      created_at:  Utc::now(),
    }
  }
}

impl From<(IssueId, IssueAttachment)> for IssueAttachmentModel {
  fn from((issue_id, attachment): (IssueId, IssueAttachment)) -> Self {
    Self { issue_id, attachment: attachment, employee_id: None, source: None, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueAttachmentRecord {
  pub id:          IssueAttachmentId,
  pub issue_id:    IssueId,
  pub attachment:  IssueAttachment,
  pub employee_id: Option<EmployeeId>,
  pub source:      Option<RunId>,
  pub created_at:  DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentModel {
  fn from(record: IssueAttachmentRecord) -> Self {
    Self {
      issue_id:    record.issue_id,
      attachment:  record.attachment,
      employee_id: record.employee_id,
      source:      record.source,
      created_at:  record.created_at,
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
      format!("DEFINE FIELD IF NOT EXISTS employee_id ON TABLE {ISSUE_ATTACHMENTS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
