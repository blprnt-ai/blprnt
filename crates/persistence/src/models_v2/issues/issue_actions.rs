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
use crate::prelude::IssueStatus;
use crate::prelude::RunId;

pub const ISSUE_ACTIONS_TABLE: &str = "issue_actions";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionId(SurrealId);

impl DbId for IssueActionId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for IssueActionId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_ACTIONS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueActionId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum IssueActionKind {
  Create,
  AddComment,
  AddAttachment,
  CheckOut,
  Release,
  Unassign,
  Assign { employee: EmployeeId },
  StatusChange { from: IssueStatus, to: IssueStatus },
  Update,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionModel {
  pub issue:       IssueId,
  pub action_kind: IssueActionKind,
  pub actor:       Option<EmployeeId>,
  pub source:      Option<RunId>,
  #[specta(type = i32)]
  pub created_at:  DateTime<Utc>,
}

impl IssueActionModel {
  pub fn new(issue: IssueId, action_kind: IssueActionKind, actor: Option<EmployeeId>, source: Option<RunId>) -> Self {
    Self { issue, action_kind, created_at: Utc::now(), actor, source }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionRecord {
  pub id:          IssueActionId,
  pub issue:       IssueId,
  pub action_kind: IssueActionKind,
  pub actor:       Option<EmployeeId>,
  pub source:      Option<RunId>,
  #[specta(type = i32)]
  pub created_at:  DateTime<Utc>,
}

impl From<IssueActionRecord> for IssueActionModel {
  fn from(record: IssueActionRecord) -> Self {
    Self {
      issue:       record.issue,
      action_kind: record.action_kind,
      actor:       record.actor,
      source:      record.source,
      created_at:  record.created_at,
    }
  }
}

impl IssueActionRecord {
  pub fn issue(&self) -> &IssueId {
    &self.issue
  }

  pub fn action_kind(&self) -> &IssueActionKind {
    &self.action_kind
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

impl IssueActionModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {ISSUE_ACTIONS_TABLE} SCHEMALESS;")).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS issue ON TABLE {ISSUE_ACTIONS_TABLE} TYPE option<record<{ISSUES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    Ok(())
  }
}
