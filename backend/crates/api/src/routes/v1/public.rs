use axum::Json;
use axum::Router;
use axum::routing::get;
use axum::routing::post;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePermissions;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;

use crate::config::deployed_mode;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::routes::v1::employees::Employee;

pub fn routes() -> Router {
  Router::new().route("/onboarding", post(owner_onboarding)).route("/owner", get(get_owner))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct OwnerOnboardingPayload {
  name:  String,
  icon:  String,
  color: String,
}

#[utoipa::path(
  post,
  path = "/onboarding",
  request_body = OwnerOnboardingPayload,
  responses(
    (status = 200, description = "Create the initial owner account", body = Employee),
    (status = 400, description = "Owner already exists", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "public"
)]
pub(super) async fn owner_onboarding(Json(payload): Json<OwnerOnboardingPayload>) -> ApiResult<Json<Employee>> {
  if deployed_mode() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!(
      "Public owner onboarding is disabled in deployed mode. Use /api/v1/auth/bootstrap-owner instead."
    ))
    .into());
  }

  let employee = EmployeeRepository::list().await?.into_iter().find(|e| e.role.is_owner());

  if employee.is_some() {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Owner already exists")).into());
  }

  let owner = EmployeeModel {
    name: payload.name,
    kind: EmployeeKind::Person,
    role: EmployeeRole::Owner,
    title: "Owner".to_string(),
    icon: payload.icon,
    color: payload.color,
    permissions: EmployeePermissions::new(true, true),
    ..Default::default()
  };

  let owner = EmployeeRepository::create(owner).await?;

  Ok(Json(owner.into()))
}

#[utoipa::path(
  get,
  path = "/owner",
  responses(
    (status = 200, description = "Fetch the owner account if it exists", body = Option<Employee>),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "public"
)]
pub(super) async fn get_owner() -> ApiResult<Json<Option<Employee>>> {
  let owner = EmployeeRepository::list().await?.into_iter().find(|e| e.role.is_owner()).map(Into::into);

  Ok(Json(owner))
}
