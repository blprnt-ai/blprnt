use axum::Extension;
use axum::Json;
use axum::Router;
use axum::routing::post;
use coordinator::Coordinator;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunRecord;
use persistence::prelude::RunTrigger;

use crate::routes::errors::ApiError;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::AppResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new().route("/runs/trigger", post(trigger_run))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TriggerRunPayload {
  employee_id: EmployeeId,
}

async fn trigger_run(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<TriggerRunPayload>,
) -> AppResult<Json<RunRecord>> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to trigger runs")).into());
  }

  let run =
    Coordinator::get().await.trigger_run_now(&payload.employee_id, RunTrigger::Manual).await.map_err(ApiError::from)?;

  match run {
    Some(run) => Ok(Json(run)),
    _ => unreachable!("None is only returned for RunTrigger::Event"),
  }
}
