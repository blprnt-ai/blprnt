use axum::Extension;
use axum::Json;
use axum::Router;
use axum::routing::post;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunRecord;

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
  todo!()
}
