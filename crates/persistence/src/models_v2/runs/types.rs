use surrealdb_types::SurrealValue;
use surrealdb_types::Value;

use crate::prelude::DbId;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub enum RunStatus {
  Pending,
  Running,
  Completed,
  Cancelled,
  Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
#[serde(rename_all = "snake_case")]
pub enum RunTrigger {
  Manual,
  Timer,
  Event { issue_id: IssueId },
}

#[derive(Clone, Debug)]
pub struct RunFilter {
  pub employee: Option<EmployeeId>,
  pub status:   Option<RunStatus>,
  pub trigger:  Option<RunTrigger>,
}

#[derive(Clone, Debug)]
pub enum RunBind {
  Employee(EmployeeId),
  Status(RunStatus),
  Trigger(RunTrigger),
}

impl RunBind {
  pub fn into_bind_value(&self) -> (String, Value) {
    match self {
      RunBind::Employee(employee) => ("employee".to_string(), employee.clone().inner().into_value()),
      RunBind::Status(status) => ("status".to_string(), status.clone().into_value()),
      RunBind::Trigger(trigger) => ("trigger".to_string(), trigger.clone().into_value()),
    }
  }
}
