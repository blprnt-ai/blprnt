use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use events::API_EVENTS;
use events::ApiEvent;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunFilter;
use persistence::prelude::RunId;
use persistence::prelude::RunRepository;
use persistence::prelude::RunTrigger;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

use crate::dto::RunDto;
use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/runs", get(list_runs))
    .route("/runs/{run_id}", get(get_run))
    .route("/runs", post(trigger_run))
    .route("/runs/{run_id}/cancel", delete(cancel_run))
    .layer(middleware::from_fn(owner_only))
}

async fn list_runs(Json(payload): Json<RunFilter>) -> ApiResult<Json<Vec<RunDto>>> {
  Ok(Json(RunRepository::list(payload).await?.into_iter().map(|r| r.into()).collect()))
}

async fn get_run(Path(run_id): Path<RunId>) -> ApiResult<Json<RunDto>> {
  Ok(Json(RunRepository::get(run_id).await?.into()))
}

async fn cancel_run(Path(run_id): Path<RunId>) -> ApiResult<StatusCode> {
  let run = RunRepository::get(run_id).await?;

  API_EVENTS.emit(ApiEvent::CancelRun { employee_id: run.employee_id, run_id: run.id })?;

  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct TriggerRunPayload {
  employee_id: EmployeeId,
}

async fn trigger_run(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<TriggerRunPayload>,
) -> ApiResult<Json<RunDto>> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to trigger runs")).into());
  }

  let (tx, rx) = oneshot::channel();
  API_EVENTS.emit(ApiEvent::StartRun {
    employee_id: payload.employee_id,
    trigger:     RunTrigger::Manual,
    rx:          Some(Arc::new(Mutex::new(Some(tx)))),
  })?;

  let run =
    rx.await.map_err(|_| ApiErrorKind::InternalServerError(serde_json::json!("Failed to receive run result")))??;

  match run {
    Some(run) => Ok(Json(run.into())),
    None => unreachable!("only RunTrigger::IssueAssignment can return None"),
  }
}
