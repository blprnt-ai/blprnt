use std::collections::HashMap;
use std::env;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use employee_import::DEFAULT_EMPLOYEES_REPO_URL;
use employee_import::EmployeeLibrarySource;
use employee_import::ImportEmployeeAction;
use employee_import::ImportEmployeeRequest;
use events::API_EVENTS;
use events::ApiEvent;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeePermissions;
use persistence::prelude::EmployeeProviderConfig;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::EmployeeRuntimeConfig;
use persistence::prelude::EmployeeSkillRef;
use persistence::prelude::EmployeeStatus;

use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/employees/me", get(get_me))
    .route("/employees/{employee_id}", get(get_employee))
    .route("/employees", get(list_employees))
    .route("/employees/org-chart", get(org_chart))
    .route("/employees/import", post(import_employee).route_layer(middleware::from_fn(owner_only)))
    .route("/employees", post(create_employee))
    .route("/employees/{employee_id}", patch(update_employee))
    .route("/employees/{employee_id}", delete(terminate_employee).route_layer(middleware::from_fn(owner_only)))
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Employee {
  id:               Uuid,
  name:             String,
  role:             EmployeeRole,
  kind:             EmployeeKind,
  icon:             String,
  color:            String,
  title:            String,
  status:           EmployeeStatus,
  capabilities:     Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  permissions:      Option<EmployeePermissions>,
  reports_to:       Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  provider_config:  Option<EmployeeProviderConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  runtime_config:   Option<EmployeeRuntimeConfig>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[schema(no_recursion)]
  chain_of_command: Vec<Employee>,
}

impl From<EmployeeRecord> for Employee {
  fn from(employee: EmployeeRecord) -> Self {
    Self {
      id:               employee.id.uuid(),
      name:             employee.name,
      role:             employee.role,
      permissions:      Some(employee.permissions),
      kind:             employee.kind,
      icon:             employee.icon,
      color:            employee.color,
      title:            employee.title,
      status:           employee.status,
      provider_config:  employee.provider_config,
      runtime_config:   employee.runtime_config,
      reports_to:       employee.reports_to.map(|id| id.uuid()),
      capabilities:     employee.capabilities,
      chain_of_command: Vec::new(),
    }
  }
}

impl Employee {
  async fn with_chain_of_command(employee_record: EmployeeRecord) -> ApiResult<Self> {
    let mut employee = Employee::from(employee_record.clone());
    let mut current_employee = employee_record.clone();

    while let Some(manager_id) = current_employee.reports_to {
      let manager = EmployeeRepository::get(manager_id).await?;
      employee.chain_of_command.push(Employee::from(manager.clone()));
      current_employee = manager;
    }

    Ok(employee)
  }

  fn with_chain_of_command_from_hashmap(
    employee_record: EmployeeRecord,
    reports_by_manager: &HashMap<Uuid, EmployeeRecord>,
  ) -> ApiResult<Self> {
    let mut employee = Employee::from(employee_record.clone());
    let mut current_employee = employee_record.clone();

    while let Some(manager_id) = current_employee.reports_to
      && reports_by_manager.get(&manager_id.clone().uuid()).is_some()
    {
      let manager = reports_by_manager.get(&manager_id.uuid()).unwrap().clone();
      employee.chain_of_command.push(Employee::from(manager.clone()));
      current_employee = manager;
    }

    Ok(employee)
  }

  fn maybe_hide_sensitive_data(&mut self, asking_employee: &EmployeeRecord) {
    if !asking_employee.is_owner() {
      self.runtime_config = None;
      self.permissions = None;
    }
  }
}

#[utoipa::path(
  get,
  path = "/employees/me",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "Fetch the authenticated employee", body = Employee),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn get_me(Extension(extension): Extension<RequestExtension>) -> ApiResult<Json<Employee>> {
  let employee = EmployeeRepository::get(extension.employee.id.clone()).await?;
  let mut employee = Employee::with_chain_of_command(employee).await?;
  employee.maybe_hide_sensitive_data(&extension.employee);

  Ok(Json(employee))
}

#[utoipa::path(
  get,
  path = "/employees/{employee_id}",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  responses(
    (status = 200, description = "Fetch an employee", body = Employee),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn get_employee(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<Uuid>,
) -> ApiResult<Json<Employee>> {
  let employee = EmployeeRepository::get(employee_id.into()).await?;
  let mut employee = Employee::with_chain_of_command(employee).await?;
  employee.maybe_hide_sensitive_data(&extension.employee);

  Ok(Json(employee))
}

#[utoipa::path(
  get,
  path = "/employees",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List employees", body = [Employee]),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn list_employees(
  Extension(extension): Extension<RequestExtension>,
) -> ApiResult<Json<Vec<Employee>>> {
  let employee_records = EmployeeRepository::list().await?;
  let mut employees: Vec<Employee> = Vec::new();

  let mut employees_by_id: HashMap<Uuid, EmployeeRecord> = HashMap::new();
  for employee in &employee_records {
    employees_by_id.insert(employee.id.clone().uuid(), employee.clone());
  }

  for employee in employees_by_id.values() {
    let mut employee = Employee::with_chain_of_command_from_hashmap(employee.clone(), &employees_by_id)?;
    employee.maybe_hide_sensitive_data(&extension.employee);
    employees.push(employee);
  }

  Ok(Json(employees))
}

#[derive(Debug, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct OrgChart {
  id:           Uuid,
  name:         String,
  role:         EmployeeRole,
  title:        String,
  status:       EmployeeStatus,
  capabilities: Vec<String>,
  #[schema(no_recursion)]
  reports:      Vec<OrgChart>,
}

impl OrgChart {
  fn from_employee(
    employee: EmployeeRecord,
    reports_by_manager: &mut HashMap<EmployeeId, Vec<EmployeeRecord>>,
  ) -> Self {
    let reports = reports_by_manager
      .remove(&employee.id)
      .unwrap_or_default()
      .into_iter()
      .map(|report| Self::from_employee(report, reports_by_manager))
      .collect();

    Self {
      id:           employee.id.uuid(),
      name:         employee.name,
      role:         employee.role,
      title:        employee.title,
      status:       employee.status,
      capabilities: employee.capabilities,
      reports:      reports,
    }
  }
}

#[utoipa::path(
  get,
  path = "/employees/org-chart",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "Fetch the organization chart", body = [OrgChart]),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn org_chart() -> ApiResult<Json<Vec<OrgChart>>> {
  let employees = EmployeeRepository::list().await?;

  let mut root_employees = Vec::new();
  let mut reports_by_manager: HashMap<EmployeeId, Vec<EmployeeRecord>> = HashMap::new();

  for employee in employees {
    match &employee.reports_to {
      Some(manager_id) => reports_by_manager.entry(manager_id.clone()).or_default().push(employee),
      None => root_employees.push(employee),
    }
  }
  let org_chart =
    root_employees.into_iter().map(|employee| OrgChart::from_employee(employee, &mut reports_by_manager)).collect();

  Ok(Json(org_chart))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct CreateEmployeePayload {
  name:            String,
  kind:            EmployeeKind,
  role:            EmployeeRole,
  title:           String,
  icon:            String,
  color:           String,
  capabilities:    Vec<String>,
  provider_config: Option<EmployeeProviderConfig>,
  runtime_config:  Option<EmployeeRuntimeConfig>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct ImportEmployeePayload {
  #[serde(default)]
  base_url:              Option<String>,
  slug:                  String,
  #[serde(default)]
  force:                 bool,
  #[serde(default)]
  skip_duplicate_skills: bool,
  #[serde(default)]
  force_skills:          bool,
}

impl From<CreateEmployeePayload> for EmployeeModel {
  fn from(payload: CreateEmployeePayload) -> Self {
    Self {
      name: payload.name,
      kind: payload.kind,
      role: payload.role,
      title: payload.title,
      icon: payload.icon,
      color: payload.color,
      capabilities: payload.capabilities,
      provider_config: payload.provider_config,
      runtime_config: payload.runtime_config,
      ..Default::default()
    }
  }
}

#[utoipa::path(
  post,
  path = "/employees/import",
  security(("blprnt_employee_id" = [])),
  request_body = ImportEmployeePayload,
  responses(
    (status = 200, description = "Import an employee definition", body = Employee),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can import employees", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn import_employee(Json(payload): Json<ImportEmployeePayload>) -> ApiResult<Json<Employee>> {
  let workspace_root = env::current_dir().map_err(|err| {
    ApiErrorKind::InternalServerError(serde_json::json!(format!("failed to resolve working directory: {err}")))
  })?;
  let source = employee_library_source(payload.base_url.as_deref());
  let imported = employee_import::import_employee(ImportEmployeeRequest {
    slug: payload.slug,
    source,
    workspace_root,
    reports_to: None,
    force: payload.force,
    skip_duplicate_skills: payload.skip_duplicate_skills,
    force_skills: payload.force_skills,
  })
  .await
  .map_err(|err| ApiErrorKind::BadRequest(serde_json::json!(err.to_string())))?;

  match imported.action {
    ImportEmployeeAction::Created => {
      API_EVENTS.emit(ApiEvent::AddEmployee { employee_id: imported.employee.id.clone() })?;
    }
    ImportEmployeeAction::Updated => {
      API_EVENTS.emit(ApiEvent::UpdateEmployee { employee_id: imported.employee.id.clone() })?;
    }
  }

  Ok(Json(imported.employee.into()))
}

#[utoipa::path(
  post,
  path = "/employees",
  security(("blprnt_employee_id" = [])),
  request_body = CreateEmployeePayload,
  responses(
    (status = 200, description = "Create an employee", body = Employee),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Forbidden", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn create_employee(
  Extension(extension): Extension<RequestExtension>,
  Json(mut payload): Json<CreateEmployeePayload>,
) -> ApiResult<Json<Employee>> {
  if payload.role.is_owner() {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Owner role is not allowed to be created")).into());
  }

  if payload.role.is_ceo() && !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to hire a CEO employee")).into());
  }

  if !extension.employee.can_hire() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to hire employees")).into());
  }

  if !extension.employee.role.can_hire_role(&payload.role) {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to hire that role")).into());
  }

  if extension.employee.kind.is_agent() && payload.kind.is_person() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to hire person employees")).into());
  }

  let has_configs = payload.provider_config.is_some() && payload.runtime_config.is_some();
  if payload.kind.is_agent() && !has_configs {
    return Err(
      ApiErrorKind::BadRequest(serde_json::json!(format!(
        "Provider config and runtime config are required for agent employees"
      )))
      .into(),
    );
  }

  normalize_skill_stack(payload.runtime_config.as_mut())?;

  let mut employee: EmployeeModel = payload.into();
  employee.reports_to = Some(extension.employee.id.clone());

  let employee = EmployeeRepository::create(employee).await?;
  if employee.kind.is_agent() {
    API_EVENTS.emit(ApiEvent::AddEmployee { employee_id: employee.id.clone() })?;
  }

  let mut employee = Employee::with_chain_of_command(employee).await?;
  employee.maybe_hide_sensitive_data(&extension.employee);

  Ok(Json(employee))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct EmployeePatchPayload {
  name:            Option<String>,
  title:           Option<String>,
  status:          Option<EmployeeStatus>,
  icon:            Option<String>,
  color:           Option<String>,
  reports_to:      Option<Option<Uuid>>,
  capabilities:    Option<Vec<String>>,
  provider_config: Option<EmployeeProviderConfig>,
  runtime_config:  Option<EmployeeRuntimeConfig>,
}

impl From<EmployeePatchPayload> for EmployeePatch {
  fn from(payload: EmployeePatchPayload) -> Self {
    Self {
      name:            payload.name,
      title:           payload.title,
      status:          payload.status,
      icon:            payload.icon,
      color:           payload.color,
      capabilities:    payload.capabilities,
      provider_config: payload.provider_config,
      runtime_config:  payload.runtime_config,
      reports_to:      payload.reports_to.map(|id| id.map(|id| id.into())),
      last_run_at:     None,
      role:            None,
    }
  }
}

#[utoipa::path(
  patch,
  path = "/employees/{employee_id}",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  request_body = EmployeePatchPayload,
  responses(
    (status = 200, description = "Update an employee", body = Employee),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Forbidden", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn update_employee(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<Uuid>,
  Json(mut payload): Json<EmployeePatchPayload>,
) -> ApiResult<Json<Employee>> {
  if !extension.employee.can_update_employee() {
    Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to update employees")).into())
  } else {
    normalize_skill_stack(payload.runtime_config.as_mut())?;
    let employee = EmployeeRepository::update(employee_id.into(), payload.into()).await?;
    if employee.kind.is_agent() {
      API_EVENTS.emit(ApiEvent::UpdateEmployee { employee_id: employee.id.clone() })?;
    }

    let mut employee = Employee::with_chain_of_command(employee).await?;
    employee.maybe_hide_sensitive_data(&extension.employee);

    Ok(Json(employee))
  }
}

fn normalize_skill_stack(runtime_config: Option<&mut EmployeeRuntimeConfig>) -> ApiResult<()> {
  let Some(runtime_config) = runtime_config else {
    return Ok(());
  };

  let skill_stack = runtime_config.skill_stack.clone().unwrap_or_default();

  let mut normalized = Vec::with_capacity(skill_stack.len());
  for skill in &skill_stack {
    let metadata = skills::validate_skill_path(std::path::Path::new(&skill.path), Some(&skill.name))
      .map_err(|err| ApiErrorKind::BadRequest(serde_json::json!(err.to_string())))?;
    normalized.push(EmployeeSkillRef { name: metadata.name, path: metadata.path.to_string_lossy().to_string() });
  }

  runtime_config.skill_stack = if normalized.is_empty() { None } else { Some(normalized) };

  Ok(())
}

#[utoipa::path(
  delete,
  path = "/employees/{employee_id}",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  responses(
    (status = 204, description = "Terminate an employee"),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can terminate employees", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn terminate_employee(Path(employee_id): Path<Uuid>) -> ApiResult<StatusCode> {
  let employee_id: EmployeeId = employee_id.into();
  let employee = EmployeeRepository::get(employee_id.clone()).await?;

  if employee.role.is_owner() {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Owner role is not allowed to be terminated")).into());
  }

  EmployeeRepository::delete(employee_id.clone()).await?;
  if employee.kind.is_agent() {
    API_EVENTS.emit(ApiEvent::DeleteEmployee { employee_id })?;
  }

  Ok(StatusCode::NO_CONTENT)
}

fn employee_library_source(base_url: Option<&str>) -> EmployeeLibrarySource {
  match base_url.map(str::trim).filter(|value| !value.is_empty()) {
    Some(value) => employee_library_source_from_value(value),
    None => match env::var("BLPRNT_EMPLOYEES_REPO") {
      Ok(value) => employee_library_source_from_value(&value),
      Err(_) => EmployeeLibrarySource::GitUrl(DEFAULT_EMPLOYEES_REPO_URL.to_string()),
    },
  }
}

fn employee_library_source_from_value(value: &str) -> EmployeeLibrarySource {
  let path = std::path::PathBuf::from(value);
  if path.exists() { EmployeeLibrarySource::Local(path) } else { EmployeeLibrarySource::GitUrl(value.to_string()) }
}
