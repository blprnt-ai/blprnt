use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
use surrealdb_types::SurrealValue;

use crate::connection::DbConnection;

pub const ISSUE_COMMENTS_TABLE: &str = "issue_comments";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentModel {
  pub issue:      SurrealId,
  pub comment:    String,
  pub creator:    SurrealId,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
}

impl IssueCommentModel {
  pub fn new(issue: SurrealId, comment: String, creator: SurrealId) -> Self {
    Self { issue, comment, creator, created_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentRecord {
  pub id:         SurrealId,
  pub issue:      SurrealId,
  pub comment:    String,
  pub creator:    SurrealId,
  #[specta(type = i32)]
  pub created_at: DateTime<Utc>,
}

impl From<IssueCommentRecord> for IssueCommentModel {
  fn from(record: IssueCommentRecord) -> Self {
    Self {
      issue:      record.issue,
      comment:    record.comment,
      creator:    record.creator,
      created_at: record.created_at,
    }
  }
}

impl IssueCommentRecord {
  pub fn issue(&self) -> &SurrealId {
    &self.issue
  }

  pub fn comment(&self) -> &String {
    &self.comment
  }

  pub fn creator(&self) -> &SurrealId {
    &self.creator
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }
}

impl IssueCommentModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS issue ON TABLE issue_comments TYPE record<issue> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    Ok(())
  }
}
