use std::collections::HashMap;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeProviderConfig;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::EmployeeRuntimeConfig;
use persistence::prelude::EmployeeStatus;

use crate::routes::errors::AppError;
use crate::routes::errors::AppErrorKind;
use crate::routes::errors::AppResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/employees/me", get(get_me))
    .route("/employees/:employee_id", get(get_employee))
    .route("/employees", get(list_employees))
    .route("/employees", get(org_chart))
    .route("/employees", post(create_employee))
    .route("/employees/:employee_id", patch(update_employee))
}

async fn get_me(Extension(extension): Extension<RequestExtension>) -> AppResult<Json<EmployeeRecord>> {
  Ok(Json(EmployeeRepository::get(extension.employee.id).await.map_err(AppError::from)?))
}

async fn get_employee(Path(employee_id): Path<EmployeeId>) -> AppResult<Json<EmployeeRecord>> {
  let employee = EmployeeRepository::get(employee_id).await.map_err(AppError::from)?;

  Ok(Json(employee))
}

async fn list_employees() -> AppResult<Json<Vec<EmployeeRecord>>> {
  let employees = EmployeeRepository::list().await.map_err(AppError::from)?;

  Ok(Json(employees))
}

#[derive(Debug, serde::Serialize)]
struct OrgChart {
  id:           EmployeeId,
  name:         String,
  role:         EmployeeRole,
  title:        String,
  status:       EmployeeStatus,
  capabilities: Vec<String>,
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
      id:           employee.id,
      name:         employee.name,
      role:         employee.role,
      title:        employee.title,
      status:       employee.status,
      capabilities: employee.capabilities,
      reports:      reports,
    }
  }
}

async fn org_chart() -> AppResult<Json<Vec<OrgChart>>> {
  let employees = EmployeeRepository::list().await.map_err(AppError::from)?;

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateEmployeePayload {
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

async fn create_employee(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<CreateEmployeePayload>,
) -> AppResult<Json<EmployeeRecord>> {
  if payload.role.is_owner() {
    return Err(AppErrorKind::BadRequest(serde_json::json!("Owner role is not allowed to be created")).into());
  }

  if payload.role.is_ceo() && !extension.employee.is_owner() {
    return Err(AppErrorKind::Forbidden(serde_json::json!("You are not authorized to hire a CEO employee")).into());
  }

  let has_configs = payload.provider_config.is_some() && payload.runtime_config.is_some();

  if !extension.employee.can_hire() {
    return Err(AppErrorKind::Forbidden(serde_json::json!("You are not authorized to hire employees")).into());
  }

  if extension.employee.kind.is_agent() && payload.kind.is_person() {
    return Err(AppErrorKind::Forbidden(serde_json::json!("You are not authorized to hire person employees")).into());
  }

  if payload.kind.is_agent() && !has_configs {
    return Err(
      AppErrorKind::BadRequest(serde_json::json!(format!(
        "Provider config and runtime config are required for agent employees"
      )))
      .into(),
    );
  }

  let mut employee: EmployeeModel = payload.into();
  employee.reports_to = Some(extension.employee.id.clone());

  let employee = EmployeeRepository::create(employee).await.map_err(AppError::from)?;

  Ok(Json(employee))
}

async fn update_employee(
  Extension(extension): Extension<RequestExtension>,
  Path(employee_id): Path<EmployeeId>,
  Json(payload): Json<EmployeePatch>,
) -> AppResult<Json<EmployeeRecord>> {
  if !extension.employee.can_update_employee() {
    Err(AppErrorKind::Forbidden(serde_json::json!("You are not authorized to update employees")).into())
  } else {
    Ok(Json(EmployeeRepository::update(employee_id, payload).await.map_err(AppError::from)?))
  }
}
