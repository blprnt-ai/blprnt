use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::prelude::EmployeeId;
use crate::prelude::ISSUES_TABLE;
use crate::prelude::IssueId;
use crate::prelude::RunId;

pub const ISSUE_COMMENTS_TABLE: &str = "issue_comments";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentId(SurrealId);

impl DbId for IssueCommentId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for IssueCommentId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_COMMENTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueCommentId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentModel {
  pub issue:      IssueId,
  pub comment:    String,
  pub creator:    Option<EmployeeId>,
  pub run:        Option<RunId>,
  #[specta(type = i32)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentRecord {
  pub id:         IssueCommentId,
  pub issue:      IssueId,
  pub comment:    String,
  pub creator:    Option<EmployeeId>,
  pub run:        Option<RunId>,
  #[specta(type = i32)]
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

impl IssueCommentRecord {
  pub fn issue(&self) -> &IssueId {
    &self.issue
  }

  pub fn comment(&self) -> &String {
    &self.comment
  }

  pub fn creator(&self) -> &Option<EmployeeId> {
    &self.creator
  }

  pub fn run(&self) -> &Option<RunId> {
    &self.run
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
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
