use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::ISSUES_TABLE;
use crate::prelude::IssueId;
use crate::prelude::RunId;

pub const ISSUE_ATTACHMENTS_TABLE: &str = "issue_attachments";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentId(SurrealId);

impl DbId for IssueAttachmentId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for IssueAttachmentId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_ATTACHMENTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueAttachmentId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum IssueAttachmentKind {
  #[default]
  Image,
  File,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachment {
  pub name:            String,
  pub attachment_kind: IssueAttachmentKind,
  pub attachment:      String,
  pub mime_kind:       String,
  pub size:            u64,
}

impl From<(IssueId, IssueAttachment)> for IssueAttachmentModel {
  fn from((issue, attachment): (IssueId, IssueAttachment)) -> Self {
    Self { issue: issue, attachment: attachment, actor: None, source: None, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentModel {
  pub issue:      IssueId,
  pub attachment: IssueAttachment,
  pub actor:      Option<EmployeeId>,
  pub source:     Option<RunId>,
  #[specta(type = i32)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentRecord {
  pub id:         IssueAttachmentId,
  pub issue:      IssueId,
  pub attachment: IssueAttachment,
  pub actor:      Option<EmployeeId>,
  pub source:     Option<RunId>,
  #[specta(type = i32)]
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

impl IssueAttachmentRecord {
  pub fn issue(&self) -> &IssueId {
    &self.issue
  }

  pub fn attachment(&self) -> &IssueAttachment {
    &self.attachment
  }

  pub fn actor(&self) -> &Option<EmployeeId> {
    &self.actor
  }

  pub fn source(&self) -> &Option<RunId> {
    &self.source
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
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
