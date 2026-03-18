#![allow(unused)]

use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::Value;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, serde::Serialize)]
pub struct AppError {
  #[serde(skip)]
  pub status:  StatusCode,
  pub message: String,
  pub code:    String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<Value>,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (self.status, Json(self)).into_response()
  }
}

#[derive(Debug, thiserror::Error)]
pub enum AppErrorKind {
  // 400
  #[error("bad request")]
  BadRequest(Value),
  // 401
  #[error("unauthorized")]
  Unauthorized(Value),
  // 403
  #[error("forbidden")]
  Forbidden(Value),
  // 404
  #[error("issue not found")]
  IssueNotFound(Value),
  #[error("project not found")]
  ProjectNotFound(Value),
  // 422
  #[error("unprocessable entity")]
  UnprocessableEntity(Value),
  // 500
  #[error("internal server error")]
  InternalServerError(Value),
}

impl From<AppErrorKind> for AppError {
  fn from(kind: AppErrorKind) -> Self {
    match kind {
      AppErrorKind::BadRequest(e) => AppError {
        status:  StatusCode::BAD_REQUEST,
        message: "Bad request".to_string(),
        code:    "BAD_REQUEST".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::Unauthorized(e) => AppError {
        status:  StatusCode::UNAUTHORIZED,
        message: "Unauthorized".to_string(),
        code:    "UNAUTHORIZED".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::Forbidden(e) => AppError {
        status:  StatusCode::FORBIDDEN,
        message: "Forbidden".to_string(),
        code:    "FORBIDDEN".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::IssueNotFound(e) => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue not found".to_string(),
        code:    "ISSUE_NOT_FOUND".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::ProjectNotFound(e) => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Project not found".to_string(),
        code:    "PROJECT_NOT_FOUND".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::UnprocessableEntity(e) => AppError {
        status:  StatusCode::UNPROCESSABLE_ENTITY,
        message: "Unprocessable entity".to_string(),
        code:    "UNPROCESSABLE_ENTITY".to_string(),
        details: Some(e.clone()),
      },
      AppErrorKind::InternalServerError(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.clone()),
      },
    }
  }
}
