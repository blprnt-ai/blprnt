use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use persistence::prelude::EmployeeId;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueActionModel;
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
use persistence::prelude::ListIssuesParams;
use persistence::prelude::ProjectId;

use crate::routes::errors::ApiError;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/issues", post(create_issue))
    .route("/issues", get(list_issues))
    .route("/issues/:issue_id", patch(update_issue))
    .route("/issues/:issue_id", get(get_issue))
    .route("/issues/:issue_id/comments", post(add_comment))
    .route("/issues/:issue_id/attachments", post(add_attachment))
    .route("/issues/:issue_id/assign", post(assign_issue))
    .route("/issues/:issue_id/unassign", post(unassign_issue))
    .route("/issues/:issue_id/checkout", post(checkout_issue))
    .route("/issues/:issue_id/release", post(release_issue))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateIssuePayload {
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
      parent_id: payload.parent.map(|p| p.into()),
      assignee: payload.assignee.map(|e| e.into()),
      ..Default::default()
    }
  }
}

async fn create_issue(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<CreateIssuePayload>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::create(payload.into()).await.map_err(ApiError::from)?;

  let model = IssueActionModel::new(issue.id.clone(), IssueActionKind::Create, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}

#[derive(Debug, serde::Serialize)]
struct GetIssueResponse {
  #[serde(flatten)]
  pub issue:       IssueRecord,
  pub comments:    Vec<IssueCommentRecord>,
  pub attachments: Vec<IssueAttachmentRecord>,
}

async fn get_issue(Path(issue_id): Path<IssueId>) -> ApiResult<Json<GetIssueResponse>> {
  let issue = IssueRepository::get(issue_id.clone().into()).await.map_err(ApiError::from)?;
  let comments = IssueRepository::list_comments(issue_id.clone().into()).await.map_err(ApiError::from)?;
  let attachments = IssueRepository::list_attachments(issue_id.clone().into()).await.map_err(ApiError::from)?;

  Ok(Json(GetIssueResponse { issue, comments, attachments }))
}

async fn list_issues(Query(mut params): Query<ListIssuesParams>) -> ApiResult<Json<Vec<IssueRecord>>> {
  if params.expected_statuses.is_none() || params.expected_statuses.as_ref().unwrap().is_empty() {
    params.expected_statuses =
      Some(vec![IssueStatus::Todo, IssueStatus::InProgress, IssueStatus::InReview, IssueStatus::Blocked]);
  }

  Ok(Json(IssueRepository::list(params).await.map_err(ApiError::from)?))
}

async fn update_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<IssuePatch>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::update(issue_id.into(), payload).await.map_err(ApiError::from)?;

  let model = IssueActionModel::new(issue.id.clone(), IssueActionKind::Update, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AddCommentPayload {
  pub comment: String,
}

async fn add_comment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<AddCommentPayload>,
) -> ApiResult<Json<IssueCommentRecord>> {
  let mut model = IssueCommentModel::default();
  model.comment = payload.comment;
  model.issue_id = issue_id.clone();
  model.creator = Some(extension.employee.id.clone());

  if let Some(run) = &extension.run_id {
    model.run = Some(run.clone());
  }

  let comment = IssueRepository::add_comment(model).await.map_err(ApiError::from)?;

  let model =
    IssueActionModel::new(issue_id.clone(), IssueActionKind::AddComment, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(comment))
}

async fn add_attachment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<IssueAttachment>,
) -> ApiResult<Json<IssueAttachmentRecord>> {
  let attachment = IssueRepository::add_attachment((issue_id.clone(), payload).into()).await.map_err(ApiError::from)?;

  let model =
    IssueActionModel::new(issue_id.clone(), IssueActionKind::AddAttachment, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(attachment))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AssignIssuePayload {
  pub employee: EmployeeId,
}

async fn assign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<AssignIssuePayload>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::assign(issue_id.into(), payload.employee.clone()).await.map_err(ApiError::from)?;

  let model = IssueActionModel::new(
    issue.id.clone(),
    IssueActionKind::Assign { employee: payload.employee },
    extension.employee.id,
    extension.run_id,
  );
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}

async fn unassign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::unassign(issue_id).await.map_err(ApiError::from)?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Unassign, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}

async fn checkout_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::checkout(issue_id, extension.employee.id.clone()).await.map_err(ApiError::from)?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::CheckOut, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}

async fn release_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueRecord>> {
  let issue = IssueRepository::release(issue_id, extension.employee.id.clone()).await.map_err(ApiError::from)?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Release, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await.map_err(ApiError::from);

  Ok(Json(issue))
}
