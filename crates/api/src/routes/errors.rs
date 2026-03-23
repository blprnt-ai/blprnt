#![allow(unused)]

use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::Value;
use serde_json::json;
use shared::errors::CoordinatorError;
use shared::errors::DatabaseError;
use shared::errors::MemoryError;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, serde::Serialize)]
pub struct ApiError {
  #[serde(skip)]
  pub status:  StatusCode,
  pub message: String,
  pub code:    String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<Value>,
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    (self.status, Json(self)).into_response()
  }
}

impl From<anyhow::Error> for ApiError {
  fn from(error: anyhow::Error) -> Self {
    ApiError {
      status:  StatusCode::INTERNAL_SERVER_ERROR,
      message: "Internal server error".to_string(),
      code:    "INTERNAL_SERVER_ERROR".to_string(),
      details: Some(error.to_string().into()),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiErrorKind {
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

impl From<ApiErrorKind> for ApiError {
  fn from(kind: ApiErrorKind) -> Self {
    match kind {
      ApiErrorKind::BadRequest(e) => ApiError {
        status:  StatusCode::BAD_REQUEST,
        message: "Bad request".to_string(),
        code:    "BAD_REQUEST".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::Unauthorized(e) => ApiError {
        status:  StatusCode::UNAUTHORIZED,
        message: "Unauthorized".to_string(),
        code:    "UNAUTHORIZED".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::Forbidden(e) => ApiError {
        status:  StatusCode::FORBIDDEN,
        message: "Forbidden".to_string(),
        code:    "FORBIDDEN".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::IssueNotFound(e) => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue not found".to_string(),
        code:    "ISSUE_NOT_FOUND".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::ProjectNotFound(e) => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Project not found".to_string(),
        code:    "PROJECT_NOT_FOUND".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::UnprocessableEntity(e) => ApiError {
        status:  StatusCode::UNPROCESSABLE_ENTITY,
        message: "Unprocessable entity".to_string(),
        code:    "UNPROCESSABLE_ENTITY".to_string(),
        details: Some(e.clone()),
      },
      ApiErrorKind::InternalServerError(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.clone()),
      },
    }
  }
}

impl From<DatabaseError> for ApiError {
  fn from(error: DatabaseError) -> Self {
    let error = match error {
      DatabaseError::Operation { entity, operation, source } => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(
          json!({ "entity": entity.to_string(), "operation": operation.to_string(), "source": source.to_string() }),
        ),
      },
      DatabaseError::NotFound { entity } => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Not found".to_string(),
        code:    "NOT_FOUND".to_string(),
        details: Some(json!({ "entity": entity.to_string() })),
      },
      DatabaseError::NotFoundAfterCreate { entity } => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Not found after create".to_string(),
        code:    "NOT_FOUND_AFTER_CREATE".to_string(),
        details: Some(json!({ "entity": entity.to_string() })),
      },
      DatabaseError::Conflict { entity, reason } => ApiError {
        status:  StatusCode::CONFLICT,
        message: "Conflict".to_string(),
        code:    "CONFLICT".to_string(),
        details: Some(json!({ "entity": entity.to_string(), "reason": reason.to_string() })),
      },
    };

    if error.status == StatusCode::INTERNAL_SERVER_ERROR && error.details.is_some() {
      tracing::error!("Internal server error: {:?}", error.details.clone().unwrap());
    }

    error
  }
}

impl From<CoordinatorError> for ApiError {
  fn from(error: CoordinatorError) -> Self {
    if let CoordinatorError::DatabaseError(e) = error {
      return e.into();
    }

    let error = match error {
      CoordinatorError::DatabaseError(e) => e.into(),
      CoordinatorError::EmployeeNotManaged => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: None,
      },
      CoordinatorError::NoRunSlotsAvailable => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: None,
      },
      CoordinatorError::FailedToEmitCoordinatorEvent(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      CoordinatorError::FailedToAwaitOneshotChannel(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
    };

    if error.status == StatusCode::INTERNAL_SERVER_ERROR && error.details.is_some() {
      tracing::error!("Internal server error: {:?}", error.details.clone().unwrap());
    }

    error
  }
}

impl From<MemoryError> for ApiError {
  fn from(error: MemoryError) -> Self {
    match error {
      MemoryError::InvalidPath(path) => ApiErrorKind::BadRequest(json!({ "path": path })).into(),
      MemoryError::ProjectNotFound(project_id) => {
        ApiErrorKind::ProjectNotFound(json!({ "project_id": project_id })).into()
      }
      MemoryError::Io(source) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(source.to_string().into()),
      },
      MemoryError::ProjectLookupFailed(source) | MemoryError::QmdOperationFailed(source) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(source.into()),
      },
      MemoryError::QmdInstallationFailed | MemoryError::QmdCollectionInitializationFailed => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: None,
      },
    }
  }
}
