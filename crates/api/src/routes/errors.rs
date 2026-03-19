#![allow(unused)]

use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde_json::Value;
use shared::errors::CoordinatorError;
use shared::errors::DatabaseError;

pub type AppResult<T> = Result<T, ApiError>;

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
      DatabaseError::FailedToBeginTransaction(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCommitTransaction(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueNotFoundAfterCreation => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Issue not found after creation".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: None,
      },
      DatabaseError::FailedToGetIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListIssues(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListChildrenIssues(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListComments(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListActions(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListAttachments(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToUpdateIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToDeleteIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCheckoutIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToReleaseIssue(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueAlreadyCheckedOutByAnotherEmployee => ApiError {
        status:  StatusCode::CONFLICT,
        message: "Issue already checked out by another employee".to_string(),
        code:    "ISSUE_ALREADY_CHECKED_OUT_BY_ANOTHER_EMPLOYEE".to_string(),
        details: None,
      },
      DatabaseError::IssueNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue not found".to_string(),
        code:    "ISSUE_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueComment(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueComment(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueCommentNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue comment not found".to_string(),
        code:    "ISSUE_COMMENT_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueAction(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueAction(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueActionNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue action not found".to_string(),
        code:    "ISSUE_ACTION_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueAttachment(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueAttachment(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueAttachmentNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue attachment not found".to_string(),
        code:    "ISSUE_ATTACHMENT_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateEmployee(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetEmployee(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::EmployeeNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Employee not found".to_string(),
        code:    "EMPLOYEE_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::EmployeeNotFoundAfterCreation => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Employee not found after creation".to_string(),
        code:    "EMPLOYEE_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToListEmployees(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToUpdateEmployee(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToDeleteEmployee(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateRun(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetRun(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListRuns(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::RunNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Run not found".to_string(),
        code:    "RUN_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::RunNotFoundAfterCreation => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Run not found after creation".to_string(),
        code:    "RUN_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToUpdateRun(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateTurn(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetTurn(e) => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::TurnNotFound => ApiError {
        status:  StatusCode::NOT_FOUND,
        message: "Turn not found".to_string(),
        code:    "TURN_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::TurnNotFoundAfterCreation => ApiError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Turn not found after creation".to_string(),
        code:    "TURN_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToUpdateTurn(e) => ApiError {
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
