#![allow(unused)]

use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use persistence::prelude::errors::DatabaseError;
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

impl From<DatabaseError> for AppError {
  fn from(error: DatabaseError) -> Self {
    let error = match error {
      DatabaseError::FailedToBeginTransaction(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCommitTransaction(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueNotFoundAfterCreation => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Issue not found after creation".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: None,
      },
      DatabaseError::FailedToGetIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListIssues(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListChildrenIssues(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListComments(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListActions(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListAttachments(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToUpdateIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToDeleteIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCheckoutIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToReleaseIssue(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueAlreadyCheckedOutByAnotherEmployee => AppError {
        status:  StatusCode::CONFLICT,
        message: "Issue already checked out by another employee".to_string(),
        code:    "ISSUE_ALREADY_CHECKED_OUT_BY_ANOTHER_EMPLOYEE".to_string(),
        details: None,
      },
      DatabaseError::IssueNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue not found".to_string(),
        code:    "ISSUE_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueComment(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueComment(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueCommentNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue comment not found".to_string(),
        code:    "ISSUE_COMMENT_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueAction(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueAction(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueActionNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue action not found".to_string(),
        code:    "ISSUE_ACTION_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateIssueAttachment(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetIssueAttachment(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::IssueAttachmentNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Issue attachment not found".to_string(),
        code:    "ISSUE_ATTACHMENT_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::FailedToCreateEmployee(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetEmployee(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::EmployeeNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Employee not found".to_string(),
        code:    "EMPLOYEE_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::EmployeeNotFoundAfterCreation => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Employee not found after creation".to_string(),
        code:    "EMPLOYEE_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToListEmployees(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToUpdateEmployee(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToDeleteEmployee(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateRun(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetRun(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToListRuns(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::RunNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Run not found".to_string(),
        code:    "RUN_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::RunNotFoundAfterCreation => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Run not found after creation".to_string(),
        code:    "RUN_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToUpdateRun(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToCreateTurn(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::FailedToGetTurn(e) => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Internal server error".to_string(),
        code:    "INTERNAL_SERVER_ERROR".to_string(),
        details: Some(e.to_string().into()),
      },
      DatabaseError::TurnNotFound => AppError {
        status:  StatusCode::NOT_FOUND,
        message: "Turn not found".to_string(),
        code:    "TURN_NOT_FOUND".to_string(),
        details: None,
      },
      DatabaseError::TurnNotFoundAfterCreation => AppError {
        status:  StatusCode::INTERNAL_SERVER_ERROR,
        message: "Turn not found after creation".to_string(),
        code:    "TURN_NOT_FOUND_AFTER_CREATION".to_string(),
        details: None,
      },
      DatabaseError::FailedToUpdateTurn(e) => AppError {
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
