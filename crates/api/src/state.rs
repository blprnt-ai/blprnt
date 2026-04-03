use persistence::prelude::EmployeeRecord;
use persistence::prelude::ProjectId;
use persistence::prelude::RunId;
use persistence::prelude::AuthSessionId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum RequestAuth {
  Header,
  Session { session_id: AuthSessionId },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestExtension {
  pub employee:   EmployeeRecord,
  pub project_id: Option<ProjectId>,
  pub run_id:     Option<RunId>,
  pub auth:       RequestAuth,
}
