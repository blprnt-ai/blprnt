use axum::Json;
use axum::Router;
use axum::routing::get;
use axum::routing::post;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePermissions;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;

use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::routes::v1::employees::Employee;

pub fn routes() -> Router {
  Router::new().route("/onboarding", post(owner_onboarding)).route("/owner", get(get_owner))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct OwnerOnboardingPayload {
  name:  String,
  icon:  String,
  color: String,
}

async fn owner_onboarding(Json(payload): Json<OwnerOnboardingPayload>) -> ApiResult<Json<Employee>> {
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

async fn get_owner() -> ApiResult<Json<Option<Employee>>> {
  let owner = EmployeeRepository::list().await?.into_iter().find(|e| e.role.is_owner()).map(Into::into);

  Ok(Json(owner))
}
