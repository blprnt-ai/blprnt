use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
use surrealdb_types::SurrealValue;

use crate::connection::DbConnection;
use crate::prelude::IssueStatus;

pub const ISSUE_ACTIONS_TABLE: &str = "issue_actions";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum IssueActionType {
  Comment,
  Assign,
  Unassign,
  StatusChange { from: IssueStatus, to: IssueStatus },
  Update,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionModel {
  pub issue:        SurrealId,
  pub action:       String,
  pub action_types: Vec<IssueActionType>,
  pub actor:        SurrealId,
  pub source:       Option<SurrealId>,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
}

impl IssueActionModel {
  pub fn new(issue: SurrealId, action: String, actor: SurrealId, source: Option<SurrealId>) -> Self {
    Self { issue, action, action_types: vec![], created_at: Utc::now(), actor, source }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionRecord {
  pub id:           SurrealId,
  pub issue:        SurrealId,
  pub action:       String,
  pub action_types: Vec<IssueActionType>,
  pub actor:        SurrealId,
  pub source:       Option<SurrealId>,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
}

impl From<IssueActionRecord> for IssueActionModel {
  fn from(record: IssueActionRecord) -> Self {
    Self {
      issue:        record.issue,
      action:       record.action,
      action_types: record.action_types,
      actor:        record.actor,
      source:       record.source,
      created_at:   record.created_at,
    }
  }
}

impl IssueActionRecord {
  pub fn issue(&self) -> &SurrealId {
    &self.issue
  }

  pub fn action(&self) -> &String {
    &self.action
  }

  pub fn action_types(&self) -> &Vec<IssueActionType> {
    &self.action_types
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

impl IssueActionModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS issue ON TABLE issue_actions TYPE option<record<issue>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    Ok(())
  }
}
