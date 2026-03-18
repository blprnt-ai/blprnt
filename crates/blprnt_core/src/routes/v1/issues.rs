use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use persistence::prelude::CompanyId;
use persistence::prelude::EmployeeId;
use persistence::prelude::IssueAttachment;
use persistence::prelude::IssueAttachmentRecord;
use persistence::prelude::IssueCommentModel;
use persistence::prelude::IssueCommentRecord;
use persistence::prelude::IssueId;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRecord;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ProjectId;

use crate::routes::types::AppErrorKind;
use crate::routes::types::AppResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/issues", post(create_issue))
    .route("/issues", get(list_issues))
    .route("/issues/:issue_id", patch(update_issue))
    .route("/issues/:issue_id", get(get_issue))
    .route("/issues/:issue_id/comments", post(add_comment))
    .route("/issues/:issue_id/attachments", post(add_attachment))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateIssuePayload {
  pub company:     CompanyId,
  pub title:       String,
  pub description: String,
  pub priority:    IssuePriority,
  pub project:     Option<ProjectId>,
  pub parent:      Option<IssueId>,
  pub assignee:    Option<EmployeeId>,
}

impl From<CreateIssuePayload> for IssueModel {
  fn from(payload: CreateIssuePayload) -> Self {
    IssueModel {
      project: payload.project.map(|p| p.into()),
      title: payload.title,
      description: payload.description,
      priority: payload.priority,
      parent: payload.parent.map(|p| p.into()),
      assignee: payload.assignee.map(|e| e.into()),
      ..Default::default()
    }
  }
}

async fn create_issue(Json(payload): Json<CreateIssuePayload>) -> AppResult<Json<IssueRecord>> {
  let issue = IssueRepository::create(payload.company.clone(), payload.into())
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;
  Ok(Json(issue))
}

#[derive(Debug, serde::Serialize)]
struct GetIssueResponse {
  #[serde(flatten)]
  pub issue:       IssueRecord,
  pub comments:    Vec<IssueCommentRecord>,
  pub attachments: Vec<IssueAttachmentRecord>,
}

async fn get_issue(Path(issue_id): Path<IssueId>) -> AppResult<Json<GetIssueResponse>> {
  let issue = IssueRepository::get(issue_id.clone().into())
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;

  let comments = IssueRepository::list_comments(issue_id.clone().into())
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;

  let attachments = IssueRepository::list_attachments(issue_id.clone().into())
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;

  Ok(Json(GetIssueResponse { issue, comments, attachments }))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ListIssuesQuery {
  pub expected_statuses: Vec<IssueStatus>,
}

async fn list_issues(
  Extension(extension): Extension<RequestExtension>,
  Query(query): Query<ListIssuesQuery>,
) -> AppResult<Json<Vec<IssueRecord>>> {
  let company = extension.company.ok_or(AppErrorKind::BadRequest(serde_json::json!("Company ID is required")))?;

  let mut issues =
    IssueRepository::list(company).await.map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;

  let expected_statuses = if query.expected_statuses.is_empty() {
    vec![IssueStatus::Todo, IssueStatus::InProgress, IssueStatus::InReview, IssueStatus::Blocked]
  } else {
    query.expected_statuses
  };

  issues.retain(|issue| expected_statuses.contains(&issue.status));

  Ok(Json(issues))
}

async fn update_issue(Path(issue_id): Path<IssueId>, Json(payload): Json<IssuePatch>) -> AppResult<Json<IssueRecord>> {
  let issue = IssueRepository::update(issue_id.into(), payload)
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;
  Ok(Json(issue))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddCommentPayload {
  pub comment: String,
}

async fn add_comment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<AddCommentPayload>,
) -> AppResult<Json<IssueCommentRecord>> {
  let mut model = IssueCommentModel::default();
  model.comment = payload.comment;
  model.issue = issue_id.into();

  if let Some(employee) = extension.employee {
    model.creator = Some(employee.into());
  }

  if let Some(run) = extension.run {
    model.run = Some(run.into());
  }

  let comment = IssueRepository::add_comment(model)
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;
  Ok(Json(comment))
}

async fn add_attachment(
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<IssueAttachment>,
) -> AppResult<Json<IssueAttachmentRecord>> {
  let attachment = IssueRepository::add_attachment((issue_id.into(), payload).into())
    .await
    .map_err(|e| AppErrorKind::IssueNotFound(serde_json::json!(e.to_string())))?;
  Ok(Json(attachment))
}
