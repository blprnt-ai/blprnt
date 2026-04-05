use std::str::FromStr;

use axum::body::Body;
use axum::extract::Request;
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use persistence::Uuid;
use persistence::prelude::AuthSessionRepository;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeRepository;
use sha2::Digest;
use sha2::Sha256;

use crate::routes::errors::ApiError;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestAuth;
use crate::state::RequestExtension;

const EMPLOYEE_ID: &str = "x-blprnt-employee-id";
const PROJECT_ID: &str = "x-blprnt-project-id";
const RUN_ID: &str = "x-blprnt-run-id";
pub(crate) const SESSION_COOKIE_NAME: &str = "blprnt_session";

pub async fn api_middleware(mut request: Request, next: Next) -> ApiResult<Response<Body>> {
  let headers = request.headers();
  let (employee, auth) = if let Some(session_token) = session_token_from_headers(headers) {
    let session = AuthSessionRepository::find_active_by_token_hash(&hash_session_token(&session_token))
      .await
      .map_err(ApiError::from)?;
    match session {
      Some(session) => {
        let employee = EmployeeRepository::get(session.employee_id.clone()).await.map_err(ApiError::from)?;
        let _ = AuthSessionRepository::touch(session.id.clone()).await;
        (employee, RequestAuth::Session { session_id: session.id })
      }
      None => {
        let employee_id: EmployeeId = header_employee_id(headers, request.uri().query())?;
        let employee = EmployeeRepository::get(employee_id).await.map_err(ApiError::from)?;
        (employee, RequestAuth::Header)
      }
    }
  } else {
    let employee_id: EmployeeId = header_employee_id(headers, request.uri().query())?;
    let employee = EmployeeRepository::get(employee_id).await.map_err(ApiError::from)?;
    (employee, RequestAuth::Header)
  };

  let project_id =
    headers.get(PROJECT_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);
  let run_id = headers.get(RUN_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);

  let extension = RequestExtension { employee, project_id, run_id, auth };
  request.extensions_mut().insert(extension);

  Ok(next.run(request).await)
}

fn header_employee_id(headers: &axum::http::HeaderMap, query: Option<&str>) -> ApiResult<EmployeeId> {
  headers
    .get(EMPLOYEE_ID)
    .and_then(|value| value.to_str().ok().map(ToOwned::to_owned))
    .or_else(|| employee_id_from_query(query))
    .and_then(|v| Uuid::from_str(&v).ok())
    .map(Into::into)
    .ok_or(ApiErrorKind::BadRequest(serde_json::json!(format!(
      "Employee header ({EMPLOYEE_ID}) or employee_id query param is required and must be valid"
    )))
    .into())
}

fn session_token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
  let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
  cookie_header.split(';').find_map(|segment| {
    let mut parts = segment.trim().splitn(2, '=');
    match (parts.next(), parts.next()) {
      (Some(name), Some(value)) if name.trim() == SESSION_COOKIE_NAME => Some(value.trim().to_string()),
      _ => None,
    }
  })
}

fn hash_session_token(token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  hex::encode(hasher.finalize())
}

fn employee_id_from_query(query: Option<&str>) -> Option<String> {
  query?.split('&').find_map(|pair| {
    let mut parts = pair.splitn(2, '=');
    match (parts.next(), parts.next()) {
      (Some("employee_id"), Some(value)) if !value.is_empty() => Some(value.to_string()),
      _ => None,
    }
  })
}

pub async fn owner_only(request: Request, next: Next) -> ApiResult<Response<Body>> {
  let extension = request.extensions().get::<RequestExtension>().expect("RequestExtension should be set already");
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("Forbidden")).into());
  }

  Ok(next.run(request).await)
}
