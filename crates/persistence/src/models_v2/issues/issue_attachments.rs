use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
use surrealdb_types::SurrealValue;

use crate::connection::DbConnection;

pub const ISSUE_ATTACHMENTS_TABLE: &str = "issue_attachments";

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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentModel {
  pub issue:      SurrealId,
  pub attachment: IssueAttachment,
  pub actor:      SurrealId,
  pub source:     Option<SurrealId>,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
}

impl Default for IssueAttachmentModel {
  fn default() -> Self {
    Self {
      issue:      SurrealId::default(),
      attachment: IssueAttachment::default(),
      actor:      SurrealId::default(),
      source:     None,
      created_at: Utc::now(),
    }
  }
}

impl IssueAttachmentModel {
  pub fn new(issue: SurrealId, attachment: IssueAttachment, actor: SurrealId, source: Option<SurrealId>) -> Self {
    Self { issue, attachment, actor, source, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentRecord {
  pub id:         SurrealId,
  pub issue:      SurrealId,
  pub attachment: IssueAttachment,
  pub actor:      SurrealId,
  pub source:     Option<SurrealId>,
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
  pub fn issue(&self) -> &SurrealId {
    &self.issue
  }

  pub fn attachment(&self) -> &IssueAttachment {
    &self.attachment
  }

  pub fn actor(&self) -> &SurrealId {
    &self.actor
  }

  pub fn source(&self) -> &Option<SurrealId> {
    &self.source
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }
}

impl IssueAttachmentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS issue ON TABLE issue_attachments TYPE option<record<issue>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS actor ON TABLE issue_attachments TYPE option<record<employees>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    Ok(())
  }
}
