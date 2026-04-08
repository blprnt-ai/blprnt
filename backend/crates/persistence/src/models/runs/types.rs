use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use surrealdb_types::Value;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::EmployeeId;
use crate::prelude::IssueCommentId;
use crate::prelude::IssueId;
use crate::prelude::SurrealId;

pub const RUNS_TABLE: &str = "runs";

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunId(pub SurrealId);

impl DbId for RunId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for RunId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(RUNS_TABLE, uuid).into())
  }
}

impl From<Uuid> for RunId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(RUNS_TABLE, uuid).into())
  }
}

impl From<RecordId> for RunId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub enum RunStatus {
  Pending,
  Running,
  Completed,
  Cancelled,
  Failed(String),
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
  Manual,
  Conversation,
  Timer,
  Dreaming,
  IssueAssignment {
    #[schema(value_type = String)]
    issue_id: IssueId,
  },
  IssueMention {
    #[schema(value_type = String)]
    issue_id: IssueId,
    #[schema(value_type = String)]
    comment_id: IssueCommentId,
  },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunFilter {
  pub employee: Option<EmployeeId>,
  pub issue:    Option<IssueId>,
  pub status:   Option<RunStatus>,
  pub trigger:  Option<RunTrigger>,
}

#[derive(Clone, Debug)]
pub enum RunBind {
  Employee(EmployeeId),
  Issue(IssueId),
  Status(RunStatus),
  Trigger(RunTrigger),
}

impl RunBind {
  pub fn into_bind_value(&self) -> (String, Value) {
    match self {
      RunBind::Employee(employee) => ("employee".to_string(), employee.clone().inner().into_value()),
      RunBind::Issue(issue) => ("issue".to_string(), issue.clone().inner().into_value()),
      RunBind::Status(status) => ("status".to_string(), status.clone().into_value()),
      RunBind::Trigger(trigger) => ("trigger".to_string(), trigger.clone().into_value()),
    }
  }
}
