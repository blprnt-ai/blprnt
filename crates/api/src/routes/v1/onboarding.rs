use axum::Json;
use axum::Router;
use axum::routing::post;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;

use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new().route("/onboarding", post(owner_onboarding))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct OwnerOnboardingPayload {
  name:  String,
  icon:  String,
  color: String,
}

async fn owner_onboarding(Json(payload): Json<OwnerOnboardingPayload>) -> ApiResult<Json<EmployeeRecord>> {
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
    ..Default::default()
  };

  let owner = EmployeeRepository::create(owner).await?;

  Ok(Json(owner))
}
