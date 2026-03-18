use persistence::prelude::CompanyId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::RunId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestExtension {
  pub company:  CompanyId,
  pub employee: Option<EmployeeId>,
  pub project:  Option<ProjectId>,
  pub run:      Option<RunId>,
}
