use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use events::API_EVENTS;
use events::ApiEvent;
use persistence::prelude::EmployeeId;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueActionModel;
use persistence::prelude::IssueAttachment;
use persistence::prelude::IssueAttachmentModel;
use persistence::prelude::IssueCommentModel;
use persistence::prelude::IssueId;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ListIssuesParams;
use persistence::prelude::ProjectId;
use persistence::prelude::RunTrigger;

use crate::dto::IssueAttachmentDto;
use crate::dto::IssueCommentDto;
use crate::dto::IssueDto;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/issues", post(create_issue))
    .route("/issues", get(list_issues))
    .route("/issues/{issue_id}", patch(update_issue))
    .route("/issues/{issue_id}", get(get_issue))
    .route("/issues/{issue_id}/comments", post(add_comment))
    .route("/issues/{issue_id}/attachments", post(add_attachment))
    .route("/issues/{issue_id}/assign", post(assign_issue))
    .route("/issues/{issue_id}/unassign", post(unassign_issue))
    .route("/issues/{issue_id}/checkout", post(checkout_issue))
    .route("/issues/{issue_id}/release", post(release_issue))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
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
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::create(payload.into()).await?;

  let model = IssueActionModel::new(issue.id.clone(), IssueActionKind::Create, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn get_issue(Path(issue_id): Path<IssueId>) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::get(issue_id.clone().into()).await?;
  let comments = IssueRepository::list_comments(issue_id.clone().into()).await?;
  let attachments = IssueRepository::list_attachments(issue_id.clone().into()).await?;
  let actions = IssueRepository::list_actions(issue_id.clone().into()).await?;

  let mut dto: IssueDto = issue.into();
  dto.comments = comments.into_iter().map(|c| c.into()).collect();
  dto.attachments = attachments.into_iter().map(|a| a.into()).collect();
  dto.actions = actions.into_iter().map(|a| a.into()).collect();

  Ok(Json(dto))
}

async fn list_issues(Query(mut params): Query<ListIssuesParams>) -> ApiResult<Json<Vec<IssueDto>>> {
  if params.expected_statuses.is_none() || params.expected_statuses.as_ref().unwrap().is_empty() {
    params.expected_statuses =
      Some(vec![IssueStatus::Todo, IssueStatus::InProgress, IssueStatus::InReview, IssueStatus::Blocked]);
  }

  let issues = IssueRepository::list(params).await?;
  let dto = issues.into_iter().map(|i| i.into()).collect();

  Ok(Json(dto))
}

async fn update_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<IssuePatch>,
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::update(issue_id.into(), payload).await?;

  let model = IssueActionModel::new(issue.id.clone(), IssueActionKind::Update, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct AddCommentPayload {
  pub comment: String,
}

async fn add_comment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<AddCommentPayload>,
) -> ApiResult<Json<IssueCommentDto>> {
  let mut model =
    IssueCommentModel::new(issue_id.clone(), payload.comment, extension.employee.id.clone(), extension.run_id.clone());

  if let Some(run) = &extension.run_id {
    model.run_id = Some(run.clone());
  }

  let comment = IssueRepository::add_comment(model).await?;

  let model =
    IssueActionModel::new(issue_id.clone(), IssueActionKind::AddComment, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(comment.into()))
}

async fn add_attachment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<IssueAttachment>,
) -> ApiResult<Json<IssueAttachmentDto>> {
  let model =
    IssueAttachmentModel::new(issue_id.clone(), payload, extension.employee.id.clone(), extension.run_id.clone());
  let attachment = IssueRepository::add_attachment(model).await?;

  let model =
    IssueActionModel::new(issue_id.clone(), IssueActionKind::AddAttachment, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(attachment.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct AssignIssuePayload {
  pub employee_id: EmployeeId,
}

async fn assign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
  Json(payload): Json<AssignIssuePayload>,
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::assign(issue_id.clone(), payload.employee_id.clone()).await?;

  let model = IssueActionModel::new(
    issue.id.clone(),
    IssueActionKind::Assign { employee: payload.employee_id.clone() },
    extension.employee.id,
    extension.run_id,
  );
  let _ = IssueRepository::add_action(model).await;

  API_EVENTS.emit(ApiEvent::StartRun {
    employee_id: payload.employee_id,
    trigger:     RunTrigger::IssueAssignment { issue_id },
    rx:          None,
  })?;

  Ok(Json(issue.into()))
}

async fn unassign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::unassign(issue_id).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Unassign, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn checkout_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::checkout(issue_id, extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::CheckOut, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn release_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<IssueId>,
) -> ApiResult<Json<IssueDto>> {
  let issue = IssueRepository::release(issue_id, extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Release, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}
