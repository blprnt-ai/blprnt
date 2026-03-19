use std::str::FromStr;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use persistence::Uuid;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeRepository;

use crate::routes::errors::ApiError;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::AppResult;
use crate::state::RequestExtension;

const EMPLOYEE_ID: &str = "x-blprnt-employee-id";
const PROJECT_ID: &str = "x-blprnt-project-id";
const RUN_ID: &str = "x-blprnt-run-id";

pub async fn api_middleware(mut request: Request, next: Next) -> AppResult<Response<Body>> {
  let headers = request.headers();
  let employee_id: EmployeeId = headers
    .get(EMPLOYEE_ID)
    .ok_or(ApiErrorKind::BadRequest(serde_json::json!(format!("Employee header ({EMPLOYEE_ID}) is required"))))?
    .to_str()
    .ok()
    .and_then(|v| Uuid::from_str(v).ok())
    .map(Into::into)
    .ok_or(ApiErrorKind::BadRequest(serde_json::json!(format!("Employee header ({EMPLOYEE_ID}) is invalid"))))?;

  let employee = EmployeeRepository::get(employee_id).await.map_err(ApiError::from)?;

  let project_id =
    headers.get(PROJECT_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);
  let run_id = headers.get(RUN_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);

  let extension = RequestExtension { employee, project_id, run_id };
  request.extensions_mut().insert(extension);

  Ok(next.run(request).await)
}
