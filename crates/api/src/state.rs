use persistence::prelude::EmployeeRecord;
use persistence::prelude::ProjectId;
use persistence::prelude::RunId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestExtension {
  pub employee:   EmployeeRecord,
  pub project_id: Option<ProjectId>,
  pub run_id:     Option<RunId>,
}
