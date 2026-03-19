use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use coordinator::Coordinator;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunFilter;
use persistence::prelude::RunId;
use persistence::prelude::RunRecord;
use persistence::prelude::RunRepository;
use persistence::prelude::RunTrigger;

use crate::events::API_EVENTS;
use crate::events::ApiEvent;
use crate::middleware::owner_only;
use crate::routes::errors::ApiError;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/runs", get(list_runs))
    .route("/runs/:run_id", get(get_run))
    .route("/runs/:run_id/cancel", delete(cancel_run))
    .route("/runs/trigger", post(trigger_run))
    .layer(middleware::from_fn(owner_only))
}

async fn list_runs(Json(payload): Json<RunFilter>) -> ApiResult<Json<Vec<RunRecord>>> {
  Ok(Json(RunRepository::list(payload).await.map_err(ApiError::from)?))
}

async fn get_run(Path(run_id): Path<RunId>) -> ApiResult<Json<RunRecord>> {
  Ok(Json(RunRepository::get(run_id).await.map_err(ApiError::from)?))
}

async fn cancel_run(Path(run_id): Path<RunId>) -> ApiResult<StatusCode> {
  let run = RunRepository::get(run_id).await.map_err(ApiError::from)?;

  API_EVENTS
    .emit(ApiEvent::CancelRun { employee_id: run.employee_id, run_id: run.id })
    .map_err(|e| ApiErrorKind::InternalServerError(serde_json::json!(e.to_string())))
    .map_err(ApiError::from)?;

  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TriggerRunPayload {
  employee_id: EmployeeId,
}

async fn trigger_run(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<TriggerRunPayload>,
) -> ApiResult<Json<RunRecord>> {
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
