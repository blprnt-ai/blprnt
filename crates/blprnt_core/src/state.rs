use persistence::prelude::CompanyId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::RunId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestExtension {
  pub employee: Option<EmployeeId>,
  pub project:  Option<ProjectId>,
  pub company:  Option<CompanyId>,
  pub run:      Option<RunId>,
}
