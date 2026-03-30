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
use persistence::Uuid;
use persistence::prelude::DbId;
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
use persistence::prelude::RunTrigger;

use crate::dto::IssueAttachmentDto;
use crate::dto::IssueCommentDto;
use crate::dto::IssueDto;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

fn deserialize_nullable_patch_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

pub fn routes() -> Router {
  Router::new()
    .route("/issues", post(create_issue))
    .route("/issues", get(list_issues))
    .route("/issues/{issue_id}", patch(update_issue))
    .route("/issues/{issue_id}", get(get_issue))
    .route("/issues/{issue_id}/children", get(list_issue_children))
    .route("/issues/{issue_id}/comments", post(add_comment))
    .route("/issues/{issue_id}/attachments", post(add_attachment))
    .route("/issues/{issue_id}/assign", post(assign_issue))
    .route("/issues/{issue_id}/unassign", post(unassign_issue))
    .route("/issues/{issue_id}/checkout", post(checkout_issue))
    .route("/issues/{issue_id}/release", post(release_issue))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, optional_fields = nullable)]
struct CreateIssuePayload {
  pub title:       String,
  pub description: String,
  pub priority:    IssuePriority,
  pub project:     Option<Uuid>,
  pub parent:      Option<Uuid>,
  pub assignee:    Option<Uuid>,
}

impl From<CreateIssuePayload> for IssueModel {
  fn from(payload: CreateIssuePayload) -> Self {
    IssueModel {
      project: payload.project.map(Into::into),
      title: payload.title,
      description: payload.description,
      priority: payload.priority,
      parent_id: payload.parent.map(Into::into),
      assignee: payload.assignee.map(Into::into),
      ..Default::default()
    }
  }
}

async fn create_issue(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<CreateIssuePayload>,
) -> ApiResult<Json<IssueDto>> {
  let mut model: IssueModel = payload.into();
  model.creator = Some(extension.employee.id.clone());
  let issue = IssueRepository::create(model).await?;

  let model = IssueActionModel::new(issue.id.clone(), IssueActionKind::Create, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn get_issue(Path(issue_id): Path<Uuid>) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::get(issue_id.clone()).await?;
  let comments = IssueRepository::list_comments(issue_id.clone()).await?;
  let attachments = IssueRepository::list_attachments(issue_id.clone()).await?;
  let actions = IssueRepository::list_actions(issue_id.clone()).await?;

  let mut dto: IssueDto = issue.into();
  dto.comments = comments.into_iter().map(|c| c.into()).collect();
  dto.attachments = attachments.into_iter().map(|a| a.into()).collect();
  dto.actions = actions.into_iter().map(|a| a.into()).collect();

  Ok(Json(dto))
}

async fn list_issues(Query(mut params): Query<ListIssuesParams>) -> ApiResult<Json<Vec<IssueDto>>> {
  if params.expected_statuses.is_none() || params.expected_statuses.as_ref().unwrap().is_empty() {
    params.expected_statuses = Some(vec![
      IssueStatus::Backlog,
      IssueStatus::Todo,
      IssueStatus::InProgress,
      IssueStatus::Blocked,
      IssueStatus::Done,
      IssueStatus::Cancelled,
    ]);
  }

  let issues = IssueRepository::list(params).await?;
  let dto = issues.into_iter().map(|i| i.into()).collect();

  Ok(Json(dto))
}

async fn list_issue_children(Path(issue_id): Path<Uuid>) -> ApiResult<Json<Vec<IssueDto>>> {
  let issue_id: IssueId = issue_id.into();
  let children = IssueRepository::list_children(issue_id).await?;
  let dto = children.into_iter().map(Into::into).collect();

  Ok(Json(dto))
}

#[derive(Debug, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct IssuePatchPayload {
  #[serde(default)]
  #[ts(optional)]
  pub title:       Option<String>,
  #[serde(default)]
  #[ts(optional)]
  pub description: Option<String>,
  #[serde(default)]
  #[ts(optional)]
  pub status:      Option<IssueStatus>,
  #[serde(default, deserialize_with = "deserialize_nullable_patch_field")]
  #[ts(as = "Option<Uuid>", optional = nullable)]
  pub project:     Option<Option<Uuid>>,
  #[serde(default)]
  #[serde(deserialize_with = "deserialize_nullable_patch_field")]
  #[ts(as = "Option<Uuid>", optional = nullable)]
  pub assignee:    Option<Option<Uuid>>,
  #[serde(default)]
  #[serde(deserialize_with = "deserialize_nullable_patch_field")]
  #[ts(as = "Option<Uuid>", optional = nullable)]
  pub blocked_by:  Option<Option<Uuid>>,
  #[serde(default)]
  #[ts(optional)]
  pub priority:    Option<IssuePriority>,
  #[serde(default)]
  #[ts(optional)]
  pub updated_at:  Option<chrono::DateTime<chrono::Utc>>,
}

impl From<IssuePatchPayload> for IssuePatch {
  fn from(payload: IssuePatchPayload) -> Self {
    Self {
      title:       payload.title,
      description: payload.description,
      status:      payload.status,
      project:     payload.project.map(|project| project.map(Into::into)),
      assignee:    payload.assignee.map(|assignee| assignee.map(Into::into)),
      blocked_by:  payload.blocked_by.map(|blocked_by| blocked_by.map(Into::into)),
      priority:    payload.priority,
      updated_at:  payload.updated_at,
    }
  }
}

async fn update_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
  Json(payload): Json<IssuePatchPayload>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let employee_id = extension.employee.id;
  let run_id = extension.run_id;

  let old_issue = IssueRepository::get(issue_id.clone()).await?;
  let issue = IssueRepository::update(issue_id.clone(), payload.into()).await?;

  let metadata_changed = old_issue.title != issue.title
    || old_issue.description != issue.description
    || old_issue.project.as_ref().map(DbId::uuid) != issue.project.as_ref().map(DbId::uuid)
    || old_issue.blocked_by.as_ref().map(DbId::uuid) != issue.blocked_by.as_ref().map(DbId::uuid)
    || old_issue.priority != issue.priority;
  let mut should_add_update_action = true;
  let mut assignee_id: Option<EmployeeId> = None;

  if old_issue.status != issue.status {
    let model = IssueActionModel::new(
      issue_id.clone(),
      IssueActionKind::StatusChange { from: old_issue.status, to: issue.status.clone() },
      employee_id.clone(),
      run_id.clone(),
    );
    let _ = IssueRepository::add_action(model).await;
    should_add_update_action = false;

    if issue.status.active() && issue.assignee.is_some() {
      assignee_id = issue.assignee.clone();
    }
  }

  if old_issue.assignee != issue.assignee {
    let kind = if issue.assignee.is_some() {
      IssueActionKind::Assign { employee: issue.assignee.clone().unwrap() }
    } else {
      IssueActionKind::Unassign
    };
    let model = IssueActionModel::new(issue_id.clone(), kind, employee_id.clone(), run_id.clone());
    let _ = IssueRepository::add_action(model).await;
    should_add_update_action = false;

    if issue.assignee.is_some() {
      should_add_update_action = false;
      assignee_id = issue.assignee.clone();
    }
  }

  if should_add_update_action || metadata_changed {
    let model = IssueActionModel::new(issue_id.clone(), IssueActionKind::Update, employee_id.clone(), run_id.clone());
    let _ = IssueRepository::add_action(model).await;
  }

  if assignee_id.is_some() {
    API_EVENTS.emit(ApiEvent::StartRun {
      employee_id: assignee_id.unwrap(),
      trigger:     RunTrigger::IssueAssignment { issue_id: issue_id.clone() },
      rx:          None,
    })?;
  }

  let comments = IssueRepository::list_comments(issue_id.clone()).await?;
  let attachments = IssueRepository::list_attachments(issue_id.clone()).await?;
  let actions = IssueRepository::list_actions(issue_id.clone()).await?;

  let mut dto: IssueDto = issue.into();
  dto.comments = comments.into_iter().map(|c| c.into()).collect();
  dto.attachments = attachments.into_iter().map(|a| a.into()).collect();
  dto.actions = actions.into_iter().map(|a| a.into()).collect();

  Ok(Json(dto))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct AddCommentPayload {
  pub comment: String,
}

async fn add_comment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
  Json(payload): Json<AddCommentPayload>,
) -> ApiResult<Json<IssueCommentDto>> {
  let issue_id: IssueId = issue_id.into();
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
  Path(issue_id): Path<Uuid>,
  Json(payload): Json<IssueAttachment>,
) -> ApiResult<Json<IssueAttachmentDto>> {
  let issue_id: IssueId = issue_id.into();
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
  pub employee_id: Uuid,
}

async fn assign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
  Json(payload): Json<AssignIssuePayload>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let employee_id: EmployeeId = payload.employee_id.into();
  let issue = IssueRepository::assign(issue_id.clone(), employee_id.clone()).await?;

  let model = IssueActionModel::new(
    issue.id.clone(),
    IssueActionKind::Assign { employee: employee_id.clone() },
    extension.employee.id,
    extension.run_id,
  );
  let _ = IssueRepository::add_action(model).await;

  API_EVENTS.emit(ApiEvent::StartRun { employee_id, trigger: RunTrigger::IssueAssignment { issue_id }, rx: None })?;

  Ok(Json(issue.into()))
}

async fn unassign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::unassign(issue_id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Unassign, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn checkout_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::checkout(issue_id.clone(), extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::CheckOut, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

async fn release_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::release(issue_id.clone(), extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Release, extension.employee.id, extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  Ok(Json(issue.into()))
}

#[cfg(test)]
mod tests {
  use serde_json::json;
  use ts_rs::TS;

  use super::CreateIssuePayload;
  use super::IssuePatchPayload;

  #[test]
  fn create_issue_payload_binding_keeps_optional_relationship_ids_optional() {
    let binding = CreateIssuePayload::decl(&ts_rs::Config::default());

    assert!(binding.contains("project?: string | null"), "{binding}");
    assert!(binding.contains("parent?: string | null"), "{binding}");
    assert!(binding.contains("assignee?: string | null"), "{binding}");
  }

  #[test]
  fn issue_patch_payload_binding_matches_sparse_http_patch_contract() {
    let binding = IssuePatchPayload::decl(&ts_rs::Config::default());

    assert!(binding.contains("title?: string"), "{binding}");
    assert!(binding.contains("description?: string"), "{binding}");
    assert!(binding.contains("status?: IssueStatus"), "{binding}");
    assert!(binding.contains("project?: string | null"), "{binding}");
    assert!(binding.contains("assignee?: string | null"), "{binding}");
    assert!(binding.contains("blocked_by?: string | null"), "{binding}");
    assert!(binding.contains("priority?: IssuePriority"), "{binding}");
    assert!(binding.contains("updated_at?: string"), "{binding}");
  }

  #[test]
  fn issue_patch_payload_preserves_explicit_null_for_project() {
    let payload: IssuePatchPayload = serde_json::from_value(json!({ "project": null })).unwrap();

    assert_eq!(payload.project, Some(None));
  }
}
