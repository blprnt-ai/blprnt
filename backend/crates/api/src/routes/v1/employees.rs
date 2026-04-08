use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::fs;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use chrono::DateTime;
use chrono::Utc;
use employee_import::DEFAULT_EMPLOYEES_REPO_URL;
use employee_import::EmployeeLibrarySource;
use employee_import::ImportEmployeeAction;
use employee_import::ImportEmployeeRequest;
use events::API_EVENTS;
use events::ApiEvent;
use events::EMPLOYEE_EVENTS;
use events::EmployeeEvent;
use events::EmployeeEventKind;
use memory::EmployeeMemoryService;
use memory::MemoryTreeNode;
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
    .route("/employees/stream", get(stream_employees))
    .route("/employees/me", get(get_me))
    .route("/employees/{employee_id}/life", get(get_employee_life))
    .route("/employees/{employee_id}/life/file", get(read_employee_life_file).patch(update_employee_life_file))
    .route("/employees/{employee_id}", get(get_employee))
    .route("/employees", get(list_employees))
    .route("/employees/org-chart", get(org_chart))
    .route("/employees/import", post(import_employee))
    .route("/employees", post(create_employee))
    .route("/employees/{employee_id}", patch(update_employee))
    .route("/employees/{employee_id}", delete(terminate_employee).route_layer(middleware::from_fn(owner_only)))
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Employee {
  id: Uuid,
  name: String,
  role: EmployeeRole,
  kind: EmployeeKind,
  icon: String,
  color: String,
  title: String,
  status: EmployeeStatus,
  capabilities: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  permissions: Option<EmployeePermissions>,
  reports_to: Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  provider_config: Option<EmployeeProviderConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  runtime_config: Option<EmployeeRuntimeConfig>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  #[schema(no_recursion)]
  chain_of_command: Vec<ThinEmployee>,
  created_at: DateTime<Utc>,
}

impl From<EmployeeRecord> for Employee {
  fn from(employee: EmployeeRecord) -> Self {
    Self {
      id: employee.id.uuid(),
      name: employee.name,
      role: employee.role,
      permissions: Some(employee.permissions),
      kind: employee.kind,
      icon: employee.icon,
      color: employee.color,
      title: employee.title,
      status: employee.status,
      provider_config: employee.provider_config,
      runtime_config: employee.runtime_config,
      reports_to: employee.reports_to.map(|id| id.uuid()),
      capabilities: employee.capabilities,
      chain_of_command: Vec::new(),
      created_at: employee.created_at,
    }
  }
}

impl Employee {
  async fn with_chain_of_command(employee_record: EmployeeRecord) -> ApiResult<Self> {
    let mut employee = Employee::from(employee_record.clone());
    let mut current_employee = employee_record.clone();

    while let Some(manager_id) = current_employee.reports_to {
      let manager = EmployeeRepository::get(manager_id).await?;
      employee.chain_of_command.push(ThinEmployee::from(manager.clone()));
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
      employee.chain_of_command.push(ThinEmployee::from(manager.clone()));
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ThinEmployee {
  id: Uuid,
  name: String,
  role: EmployeeRole,
}

impl From<EmployeeRecord> for ThinEmployee {
  fn from(employee: EmployeeRecord) -> Self {
    Self { id: employee.id.uuid(), name: employee.name, role: employee.role }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EmployeeStreamSnapshotDto {
  employees: Vec<Employee>,
}

const EDITABLE_LIFE_DOCS: [&str; 4] = ["HEARTBEAT.md", "SOUL.md", "AGENTS.md", "TOOLS.md"];

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeLifeFileKind {
  Memory,
  HomeDoc,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmployeeLifeTreeNode {
  Directory {
    name: String,
    path: String,
    #[schema(no_recursion)]
    children: Vec<EmployeeLifeTreeNode>,
  },
  File {
    name: String,
    path: String,
    kind: EmployeeLifeFileKind,
    editable: bool,
  },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EmployeeLifeTreeResult {
  root_path: String,
  nodes: Vec<EmployeeLifeTreeNode>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EmployeeLifeFileResult {
  path: String,
  content: String,
  kind: EmployeeLifeFileKind,
  editable: bool,
}

#[derive(Debug, Clone, serde::Deserialize, ts_rs::TS, utoipa::IntoParams, utoipa::ToSchema)]
#[ts(export)]
pub struct EmployeeLifeFileQuery {
  path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EmployeeLifeFilePatchPayload {
  path: String,
  content: String,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmployeeStreamMessageDto {
  Snapshot { snapshot: EmployeeStreamSnapshotDto },
  Upsert { employee: Employee },
  Delete { employee_id: Uuid },
}

async fn load_visible_employees(asking_employee: &EmployeeRecord) -> ApiResult<Vec<Employee>> {
  let employee_records = EmployeeRepository::list().await?;
  let mut employees: Vec<Employee> = Vec::new();

  let mut employees_by_id: HashMap<Uuid, EmployeeRecord> = HashMap::new();
  for employee in &employee_records {
    employees_by_id.insert(employee.id.clone().uuid(), employee.clone());
  }

  for employee in employees_by_id.values() {
    let mut employee = Employee::with_chain_of_command_from_hashmap(employee.clone(), &employees_by_id)?;
    employee.maybe_hide_sensitive_data(asking_employee);
    employees.push(employee);
  }

  Ok(employees)
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
  path = "/employees/{employee_id}/life",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  responses(
    (status = 200, description = "Fetch an employee life tree", body = EmployeeLifeTreeResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn get_employee_life(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<Uuid>,
) -> ApiResult<Json<EmployeeLifeTreeResult>> {
  let target = EmployeeRepository::get(employee_id.into()).await?;
  let can_edit_docs = can_write_employee_life_doc(&extension.employee, &target).await?;
  let memory = EmployeeMemoryService::new(target.id.clone()).await?.list().await?;

  let mut nodes = EDITABLE_LIFE_DOCS
    .iter()
    .map(|file_name| EmployeeLifeTreeNode::File {
      name: file_name.to_string(),
      path: file_name.to_string(),
      kind: EmployeeLifeFileKind::HomeDoc,
      editable: can_edit_docs,
    })
    .collect::<Vec<_>>();
  nodes.push(EmployeeLifeTreeNode::Directory {
    name: "memory".to_string(),
    path: "memory".to_string(),
    children: prefix_memory_nodes(memory.nodes, "memory"),
  });

  Ok(Json(EmployeeLifeTreeResult { root_path: "$AGENT_HOME".to_string(), nodes }))
}

#[utoipa::path(
  get,
  path = "/employees/{employee_id}/life/file",
  security(("blprnt_employee_id" = [])),
  params(
    ("employee_id" = Uuid, Path, description = "Employee id"),
    EmployeeLifeFileQuery
  ),
  responses(
    (status = 200, description = "Read an employee life file", body = EmployeeLifeFileResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn read_employee_life_file(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<Uuid>,
  Query(query): Query<EmployeeLifeFileQuery>,
) -> ApiResult<Json<EmployeeLifeFileResult>> {
  let target = EmployeeRepository::get(employee_id.into()).await?;
  let can_edit_docs = can_write_employee_life_doc(&extension.employee, &target).await?;
  let (kind, relative_path) = parse_life_file_path(&query.path)?;

  let content = match kind {
    EmployeeLifeFileKind::HomeDoc => read_employee_doc(target.id.uuid(), relative_path)?,
    EmployeeLifeFileKind::Memory => {
      EmployeeMemoryService::new(target.id.clone()).await?.read(relative_path).await?.content
    }
  };

  Ok(Json(EmployeeLifeFileResult {
    path: query.path,
    content,
    kind,
    editable: matches!(kind, EmployeeLifeFileKind::HomeDoc) && can_edit_docs,
  }))
}

#[utoipa::path(
  patch,
  path = "/employees/{employee_id}/life/file",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  request_body = EmployeeLifeFilePatchPayload,
  responses(
    (status = 200, description = "Update an employee life file", body = EmployeeLifeFileResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Forbidden", body = crate::routes::errors::ApiError),
    (status = 404, description = "Employee not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "employees"
)]
pub(super) async fn update_employee_life_file(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<Uuid>,
  Json(payload): Json<EmployeeLifeFilePatchPayload>,
) -> ApiResult<Json<EmployeeLifeFileResult>> {
  let target = EmployeeRepository::get(employee_id.into()).await?;
  if !can_write_employee_life_doc(&extension.employee, &target).await? {
    return Err(
      ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to update this employee file")).into(),
    );
  }

  let (kind, relative_path) = parse_life_file_path(&payload.path)?;
  if !matches!(kind, EmployeeLifeFileKind::HomeDoc) {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Memory files are read-only from this endpoint")).into());
  }

  write_employee_doc(target.id.uuid(), relative_path, &payload.content)?;

  Ok(Json(EmployeeLifeFileResult { path: payload.path, content: payload.content, kind, editable: true }))
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
  Ok(Json(load_visible_employees(&extension.employee).await?))
}

#[derive(Debug, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct OrgChart {
  id: Uuid,
  name: String,
  role: EmployeeRole,
  title: String,
  status: EmployeeStatus,
  capabilities: Vec<String>,
  #[schema(no_recursion)]
  reports: Vec<OrgChart>,
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
      id: employee.id.uuid(),
      name: employee.name,
      role: employee.role,
      title: employee.title,
      status: employee.status,
      capabilities: employee.capabilities,
      reports: reports,
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
  name: String,
  kind: EmployeeKind,
  role: EmployeeRole,
  title: String,
  icon: String,
  color: String,
  capabilities: Vec<String>,
  provider_config: Option<EmployeeProviderConfig>,
  runtime_config: Option<EmployeeRuntimeConfig>,
  #[serde(default)]
  heartbeat_md: Option<String>,
  #[serde(default)]
  soul_md: Option<String>,
  #[serde(default)]
  agents_md: Option<String>,
  #[serde(default)]
  tools_md: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct ImportEmployeePayload {
  #[serde(default)]
  base_url: Option<String>,
  slug: String,
  #[serde(default)]
  force: bool,
  #[serde(default)]
  skip_duplicate_skills: bool,
  #[serde(default)]
  force_skills: bool,
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

impl CreateEmployeePayload {
  fn docs(&self) -> [(&'static str, Option<&str>); 4] {
    [
      ("HEARTBEAT.md", self.heartbeat_md.as_deref()),
      ("SOUL.md", self.soul_md.as_deref()),
      ("AGENTS.md", self.agents_md.as_deref()),
      ("TOOLS.md", self.tools_md.as_deref()),
    ]
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
  emit_employee_event(imported.employee.id.clone(), EmployeeEventKind::Upsert);

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

  let docs = payload.docs().map(|(name, contents)| (name, contents.map(str::to_owned)));

  let mut employee: EmployeeModel = payload.into();
  employee.reports_to = Some(extension.employee.id.clone());

  let employee = EmployeeRepository::create(employee).await?;
  write_employee_docs(employee.id.uuid(), &docs)?;
  if employee.kind.is_agent() {
    API_EVENTS.emit(ApiEvent::AddEmployee { employee_id: employee.id.clone() })?;
  }
  emit_employee_event(employee.id.clone(), EmployeeEventKind::Upsert);

  let mut employee = Employee::with_chain_of_command(employee).await?;
  employee.maybe_hide_sensitive_data(&extension.employee);

  Ok(Json(employee))
}

fn write_employee_docs(employee_id: Uuid, docs: &[(&'static str, Option<String>); 4]) -> ApiResult<()> {
  let employee_home = shared::paths::employee_home(&employee_id.to_string());
  fs::create_dir_all(&employee_home).map_err(|err| {
    ApiErrorKind::InternalServerError(serde_json::json!(format!(
      "failed to create employee home {}: {err}",
      employee_home.display()
    )))
  })?;

  for (file_name, contents) in docs {
    let Some(contents) = contents else {
      continue;
    };

    fs::write(employee_home.join(file_name), contents).map_err(|err| {
      ApiErrorKind::InternalServerError(serde_json::json!(format!(
        "failed to write employee file {}: {err}",
        employee_home.join(file_name).display()
      )))
    })?;
  }

  Ok(())
}

fn read_employee_doc(employee_id: Uuid, file_name: &str) -> ApiResult<String> {
  let employee_home = shared::paths::employee_home(&employee_id.to_string());

  let file_path = employee_home.join(file_name);
  if !file_path.exists() {
    return Ok(String::new());
  }

  fs::read_to_string(&file_path).map_err(|err| {
    ApiErrorKind::InternalServerError(serde_json::json!(format!(
      "failed to read employee file {}: {err}",
      file_path.display()
    )))
    .into()
  })
}

fn write_employee_doc(employee_id: Uuid, file_name: &str, contents: &str) -> ApiResult<()> {
  let employee_home = shared::paths::employee_home(&employee_id.to_string());
  fs::create_dir_all(&employee_home).map_err(|err| {
    ApiErrorKind::InternalServerError(serde_json::json!(format!(
      "failed to create employee home {}: {err}",
      employee_home.display()
    )))
  })?;

  let file_path = employee_home.join(file_name);
  fs::write(&file_path, contents).map_err(|err| {
    ApiErrorKind::InternalServerError(serde_json::json!(format!(
      "failed to write employee file {}: {err}",
      file_path.display()
    )))
    .into()
  })
}

fn parse_life_file_path(path: &str) -> ApiResult<(EmployeeLifeFileKind, &str)> {
  if EDITABLE_LIFE_DOCS.contains(&path) {
    return Ok((EmployeeLifeFileKind::HomeDoc, path));
  }

  let Some(relative_path) = path.strip_prefix("memory/") else {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Unsupported life file path")).into());
  };

  Ok((EmployeeLifeFileKind::Memory, relative_path))
}

fn prefix_memory_nodes(nodes: Vec<MemoryTreeNode>, parent_path: &str) -> Vec<EmployeeLifeTreeNode> {
  nodes
    .into_iter()
    .map(|node| match node {
      MemoryTreeNode::Directory { name, path, children } => EmployeeLifeTreeNode::Directory {
        name,
        path: format!("{parent_path}/{path}"),
        children: prefix_memory_nodes(children, parent_path),
      },
      MemoryTreeNode::File { name, path } => EmployeeLifeTreeNode::File {
        name,
        path: format!("{parent_path}/{path}"),
        kind: EmployeeLifeFileKind::Memory,
        editable: false,
      },
    })
    .collect()
}

async fn can_write_employee_life_doc(actor: &EmployeeRecord, target: &EmployeeRecord) -> ApiResult<bool> {
  if actor.is_owner() || actor.is_ceo() {
    return Ok(true);
  }

  if actor.id == target.id {
    return Ok(true);
  }

  if !matches!(actor.role, EmployeeRole::Manager) {
    return Ok(false);
  }

  let employees = EmployeeRepository::list().await?;
  Ok(reporting_tree_employee_ids(actor, &employees).contains(&target.id))
}

fn reporting_tree_employee_ids(manager: &EmployeeRecord, employees: &[EmployeeRecord]) -> HashSet<EmployeeId> {
  let reports_by_manager = employees.iter().fold(HashMap::<EmployeeId, Vec<EmployeeId>>::new(), |mut acc, employee| {
    if let Some(manager_id) = &employee.reports_to {
      acc.entry(manager_id.clone()).or_default().push(employee.id.clone());
    }
    acc
  });

  let mut descendants = HashSet::new();
  let mut queue = VecDeque::from([manager.id.clone()]);
  let mut visited = HashSet::new();

  while let Some(manager_id) = queue.pop_front() {
    if !visited.insert(manager_id.clone()) {
      continue;
    }

    if let Some(reports) = reports_by_manager.get(&manager_id) {
      for report_id in reports {
        descendants.insert(report_id.clone());
        queue.push_back(report_id.clone());
      }
    }
  }

  descendants
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct EmployeePatchPayload {
  name: Option<String>,
  title: Option<String>,
  status: Option<EmployeeStatus>,
  icon: Option<String>,
  color: Option<String>,
  reports_to: Option<Option<Uuid>>,
  capabilities: Option<Vec<String>>,
  provider_config: Option<EmployeeProviderConfig>,
  runtime_config: Option<EmployeeRuntimeConfig>,
}

impl From<EmployeePatchPayload> for EmployeePatch {
  fn from(payload: EmployeePatchPayload) -> Self {
    Self {
      name: payload.name,
      title: payload.title,
      status: payload.status,
      icon: payload.icon,
      color: payload.color,
      capabilities: payload.capabilities,
      provider_config: payload.provider_config,
      runtime_config: payload.runtime_config,
      reports_to: payload.reports_to.map(|id| id.map(|id| id.into())),
      last_run_at: None,
      role: None,
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
    emit_employee_event(employee.id.clone(), EmployeeEventKind::Upsert);

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
  if skill_stack.len() > 2 {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Skill stack supports at most 2 skills")).into());
  }

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
    API_EVENTS.emit(ApiEvent::DeleteEmployee { employee_id: employee_id.clone() })?;
  }
  emit_employee_event(employee_id, EmployeeEventKind::Delete);

  Ok(StatusCode::NO_CONTENT)
}

async fn stream_employees(
  Extension(extension): Extension<RequestExtension>,
  ws: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
  ws.on_upgrade(move |socket| handle_employee_socket(socket, extension.employee))
}

async fn handle_employee_socket(mut socket: WebSocket, asking_employee: EmployeeRecord) {
  if send_employee_snapshot(&mut socket, &asking_employee).await.is_err() {
    return;
  }

  let mut employee_events = EMPLOYEE_EVENTS.subscribe();

  loop {
    tokio::select! {
      event = employee_events.recv() => {
        let Ok(event) = event else {
          break;
        };

        if send_employee_event_message(&mut socket, &asking_employee, event).await.is_err() {
          break;
        }
      }
      message = socket.recv() => {
        match message {
          Some(Ok(Message::Close(_))) | None => break,
          Some(Ok(Message::Ping(payload))) => {
            if socket.send(Message::Pong(payload)).await.is_err() {
              break;
            }
          }
          Some(Ok(_)) => {}
          Some(Err(_)) => break,
        }
      }
    }
  }
}

async fn send_employee_snapshot(socket: &mut WebSocket, asking_employee: &EmployeeRecord) -> anyhow::Result<()> {
  let employees = load_visible_employees(asking_employee)
    .await
    .map_err(|error| anyhow::anyhow!("{} ({})", error.message, error.code))?;
  let message = EmployeeStreamMessageDto::Snapshot { snapshot: EmployeeStreamSnapshotDto { employees } };
  socket.send(Message::Text(serde_json::to_string(&message)?.into())).await?;
  Ok(())
}

async fn send_employee_event_message(
  socket: &mut WebSocket,
  asking_employee: &EmployeeRecord,
  event: EmployeeEvent,
) -> anyhow::Result<()> {
  let message = match event.kind {
    EmployeeEventKind::Upsert => {
      let employee = EmployeeRepository::get(event.employee_id).await?;
      let mut employee = Employee::with_chain_of_command(employee)
        .await
        .map_err(|error| anyhow::anyhow!("{} ({})", error.message, error.code))?;
      employee.maybe_hide_sensitive_data(asking_employee);
      EmployeeStreamMessageDto::Upsert { employee }
    }
    EmployeeEventKind::Delete => EmployeeStreamMessageDto::Delete { employee_id: event.employee_id.uuid() },
  };

  socket.send(Message::Text(serde_json::to_string(&message)?.into())).await?;
  Ok(())
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

fn emit_employee_event(employee_id: EmployeeId, kind: EmployeeEventKind) {
  if let Err(error) = EMPLOYEE_EVENTS.emit(EmployeeEvent { employee_id, kind }) {
    tracing::debug!(?error, "dropping employee event without subscribers");
  }
}
