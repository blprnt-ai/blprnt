use std::collections::HashSet;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use events::API_EVENTS;
use events::ApiEvent;
use events::ISSUE_EVENTS;
use events::IssueEvent;
use events::IssueEventKind;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeStatus;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueActionModel;
use persistence::prelude::IssueAttachment;
use persistence::prelude::IssueAttachmentId;
use persistence::prelude::IssueAttachmentModel;
use persistence::prelude::IssueCommentMention;
use persistence::prelude::IssueCommentModel;
use persistence::prelude::IssueId;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ListIssuesParams;
use persistence::prelude::RunFilter;
use persistence::prelude::RunId;
use persistence::prelude::RunRepository;
use persistence::prelude::RunTrigger;

use crate::dto::IssueAttachmentDetailDto;
use crate::dto::IssueAttachmentDto;
use crate::dto::IssueCommentDto;
use crate::dto::IssueDto;
use crate::dto::IssueEventKindDto;
use crate::dto::IssueStreamMessageDto;
use crate::dto::IssueStreamSnapshotDto;
use crate::dto::RunSummaryDto;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

fn deserialize_nullable_patch_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

pub(crate) async fn load_issue_dto(issue_id: IssueId, for_owner: bool) -> anyhow::Result<IssueDto> {
  let issue = IssueRepository::get(issue_id.clone()).await?;
  let comments = IssueRepository::list_comments(issue_id.clone()).await?;

  let mut dto: IssueDto = issue.into();
  dto.comments = comments.into_iter().map(IssueCommentDto::from).collect();

  let attachments = IssueRepository::list_attachments(issue_id.clone()).await?;
  dto.attachments = attachments.into_iter().map(IssueAttachmentDto::from).collect();

  if for_owner {
    let actions = IssueRepository::list_actions(issue_id.clone()).await?;
    dto.actions = actions.into_iter().map(Into::into).collect();
  }

  Ok(dto)
}

fn emit_issue_event(event: IssueEvent) {
  let _ = ISSUE_EVENTS.emit(event);
}

async fn was_employee_assigned_in_same_run(
  issue_id: &IssueId,
  employee_id: &EmployeeId,
  run_id: Option<&RunId>,
) -> bool {
  let Some(run_id) = run_id else {
    return false;
  };

  let Ok(actions) = IssueRepository::list_actions(issue_id.clone()).await else {
    return false;
  };

  actions.into_iter().rev().any(|action| {
    action.run_id.as_ref() == Some(run_id)
      && matches!(action.action_kind, IssueActionKind::Assign { employee: ref assigned_employee } if assigned_employee == employee_id)
  })
}

pub fn routes() -> Router {
  Router::new()
    .route("/issues", post(create_issue))
    .route("/issues", get(list_issues))
    .route("/issues/{issue_id}", patch(update_issue))
    .route("/issues/{issue_id}", get(get_issue))
    .route("/issues/{issue_id}/runs", get(list_issue_runs))
    .route("/issues/{issue_id}/children", get(list_issue_children))
    .route("/issues/{issue_id}/comments", get(get_comments))
    .route("/issues/{issue_id}/comments", post(add_comment))
    .route("/issues/{issue_id}/attachments/{attachment_id}", get(get_attachment))
    .route("/issues/{issue_id}/attachments", post(add_attachment))
    .route("/issues/{issue_id}/assign", post(assign_issue))
    .route("/issues/{issue_id}/unassign", post(unassign_issue))
    .route("/issues/{issue_id}/checkout", post(checkout_issue))
    .route("/issues/{issue_id}/release", post(release_issue))
    .route("/issues/stream", get(stream_issues))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, optional_fields = nullable)]
pub(crate) struct CreateIssuePayload {
  pub title:       String,
  pub description: String,
  pub status:      IssueStatus,
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
      status: payload.status,
      priority: payload.priority,
      parent_id: payload.parent.map(Into::into),
      assignee: payload.assignee.map(Into::into),
      ..Default::default()
    }
  }
}

#[utoipa::path(
  post,
  path = "/issues",
  security(("blprnt_employee_id" = [])),
  request_body = CreateIssuePayload,
  responses(
    (status = 200, description = "Create an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(crate) async fn create_issue(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<CreateIssuePayload>,
) -> ApiResult<Json<IssueDto>> {
  let mut model: IssueModel = payload.into();
  model.creator = Some(extension.employee.id.clone());
  let issue = IssueRepository::create(model).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Create, extension.employee.id.clone(), extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  if issue.status.active() && issue.assignee.is_some() {
    API_EVENTS.emit(ApiEvent::StartRun {
      employee_id: issue.assignee.clone().unwrap(),
      run_id:      None,
      trigger:     RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      rx:          None,
    })?;
  }

  let dto = load_issue_dto(issue.id.clone(), extension.employee.is_owner()).await?;
  emit_issue_event(IssueEvent { issue_id: issue.id, kind: IssueEventKind::Created });

  Ok(Json(dto))
}

#[utoipa::path(
  get,
  path = "/issues/{issue_id}",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "Fetch an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn get_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  Ok(Json(load_issue_dto(issue_id.into(), extension.employee.is_owner()).await?))
}

#[utoipa::path(
  get,
  path = "/issues/{issue_id}/runs",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "List issue-associated run summaries", body = [RunSummaryDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn list_issue_runs(Path(issue_id): Path<Uuid>) -> ApiResult<Json<Vec<RunSummaryDto>>> {
  let issue_id: IssueId = issue_id.into();
  let _ = IssueRepository::get(issue_id.clone()).await?;

  let runs = RunRepository::list_summaries(
    RunFilter {
      employee: None,
      issue: Some(issue_id),
      status: None,
      trigger: None,
    },
    None,
    None,
  )
  .await?;

  Ok(Json(runs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
  get,
  path = "/issues",
  security(("blprnt_employee_id" = [])),
  params(ListIssuesParams),
  responses(
    (status = 200, description = "List issues", body = [IssueDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn list_issues(Query(mut params): Query<ListIssuesParams>) -> ApiResult<Json<Vec<IssueDto>>> {
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

#[utoipa::path(
  get,
  path = "/issues/{issue_id}/children",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "List child issues", body = [IssueDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn list_issue_children(Path(issue_id): Path<Uuid>) -> ApiResult<Json<Vec<IssueDto>>> {
  let issue_id: IssueId = issue_id.into();
  let children = IssueRepository::list_children(issue_id).await?;
  let dto = children.into_iter().map(Into::into).collect();

  Ok(Json(dto))
}

#[derive(Debug, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct IssuePatchPayload {
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

#[utoipa::path(
  patch,
  path = "/issues/{issue_id}",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  request_body = IssuePatchPayload,
  responses(
    (status = 200, description = "Update an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn update_issue(
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
      run_id:      None,
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

  emit_issue_event(IssueEvent { issue_id, kind: IssueEventKind::Updated });

  Ok(Json(dto))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(crate) struct AddCommentPayload {
  pub comment:      String,
  pub reopen_issue: Option<bool>,
  #[serde(default)]
  pub mentions:     Vec<MentionPayload>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(crate) struct MentionPayload {
  pub employee_id: Uuid,
  pub label:       String,
}

impl From<MentionPayload> for IssueCommentMention {
  fn from(payload: MentionPayload) -> Self {
    Self { employee_id: payload.employee_id.into(), label: payload.label }
  }
}

#[utoipa::path(
  get,
  path = "/issues/{issue_id}/comments",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "List issue comments", body = [IssueCommentDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn get_comments(Path(issue_id): Path<Uuid>) -> ApiResult<Json<Vec<IssueCommentDto>>> {
  let issue_id: IssueId = issue_id.into();
  let comments = IssueRepository::list_comments(issue_id.clone()).await?;
  let dto = comments.into_iter().map(IssueCommentDto::from).collect();

  Ok(Json(dto))
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/comments",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  request_body = AddCommentPayload,
  responses(
    (status = 200, description = "Add an issue comment", body = IssueCommentDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(crate) async fn add_comment(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
  Json(payload): Json<AddCommentPayload>,
) -> ApiResult<Json<IssueCommentDto>> {
  let issue_id: IssueId = issue_id.into();
  let should_reopen_issue = payload.reopen_issue.unwrap_or(false);
  let mentions = payload.mentions.into_iter().map(Into::into).collect::<Vec<IssueCommentMention>>();
  let mut model = IssueCommentModel::new(
    issue_id.clone(),
    payload.comment,
    mentions,
    extension.employee.id.clone(),
    extension.run_id.clone(),
  );

  if let Some(run) = &extension.run_id {
    model.run_id = Some(run.clone());
  }

  let comment = IssueRepository::add_comment(model).await?;

  let model = IssueActionModel::new(
    issue_id.clone(),
    IssueActionKind::AddComment,
    extension.employee.id.clone(),
    extension.run_id.clone(),
  );
  let _ = IssueRepository::add_action(model).await;

  if should_reopen_issue {
    let issue = IssueRepository::get(issue_id.clone()).await?;
    if issue.status == IssueStatus::Done {
      IssueRepository::update(issue_id.clone(), IssuePatch { status: Some(IssueStatus::Todo), ..Default::default() })
        .await?;
      let model = IssueActionModel::new(
        issue_id.clone(),
        IssueActionKind::Update,
        extension.employee.id.clone(),
        extension.run_id.clone(),
      );
      let _ = IssueRepository::add_action(model).await;
      emit_issue_event(IssueEvent { issue_id: issue_id.clone(), kind: IssueEventKind::Updated });
    }
  }

  let mut triggered_employees = HashSet::new();
  for mention in &comment.mentions.clone().unwrap_or_default() {
    if mention.employee_id == extension.employee.id || !triggered_employees.insert(mention.employee_id.clone()) {
      continue;
    }

    if was_employee_assigned_in_same_run(&issue_id, &mention.employee_id, extension.run_id.as_ref()).await {
      continue;
    }

    let Ok(employee) = EmployeeRepository::get(mention.employee_id.clone()).await else {
      continue;
    };

    let wake_on_demand = employee.runtime_config.as_ref().map(|config| config.wake_on_demand).unwrap_or(false);
    if employee.status == EmployeeStatus::Paused || !wake_on_demand {
      continue;
    }

    let _ = API_EVENTS.emit(ApiEvent::StartRun {
      employee_id: mention.employee_id.clone(),
      run_id:      None,
      trigger:     RunTrigger::IssueMention { issue_id: issue_id.clone(), comment_id: comment.id.clone() },
      rx:          None,
    });
  }

  emit_issue_event(IssueEvent { issue_id, kind: IssueEventKind::CommentAdded });

  Ok(Json(comment.into()))
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/attachments",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  request_body = IssueAttachment,
  responses(
    (status = 200, description = "Add an issue attachment", body = IssueAttachmentDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn add_attachment(
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

  emit_issue_event(IssueEvent { issue_id, kind: IssueEventKind::AttachmentAdded });

  Ok(Json(attachment.into()))
}

#[utoipa::path(
  get,
  path = "/issues/{issue_id}/attachments/{attachment_id}",
  security(("blprnt_employee_id" = [])),
  params(
    ("issue_id" = Uuid, Path, description = "Issue id"),
    ("attachment_id" = Uuid, Path, description = "Attachment id")
  ),
  responses(
    (status = 200, description = "Fetch one issue attachment", body = IssueAttachmentDetailDto),
    (status = 404, description = "Attachment not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn get_attachment(
  Path((issue_id, attachment_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IssueAttachmentDetailDto>> {
  let issue_id: IssueId = issue_id.into();
  let attachment = IssueRepository::get_attachment(IssueAttachmentId::from(attachment_id)).await?;

  if attachment.issue_id != issue_id {
    return Err(
      crate::routes::errors::ApiErrorKind::IssueNotFound(serde_json::json!(
        "Attachment does not belong to the requested issue"
      ))
      .into(),
    );
  }

  Ok(Json(attachment.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct AssignIssuePayload {
  pub employee_id: Uuid,
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/assign",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  request_body = AssignIssuePayload,
  responses(
    (status = 200, description = "Assign an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn assign_issue(
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
    extension.employee.id.clone(),
    extension.run_id,
  );
  let _ = IssueRepository::add_action(model).await;

  API_EVENTS.emit(ApiEvent::StartRun {
    employee_id,
    run_id: None,
    trigger: RunTrigger::IssueAssignment { issue_id },
    rx: None,
  })?;

  emit_issue_event(IssueEvent { issue_id: issue.id.clone(), kind: IssueEventKind::Assigned });

  Ok(Json(load_issue_dto(issue.id, extension.employee.is_owner()).await?))
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/unassign",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "Unassign an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn unassign_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::unassign(issue_id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Unassign, extension.employee.id.clone(), extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  emit_issue_event(IssueEvent { issue_id: issue.id.clone(), kind: IssueEventKind::Unassigned });

  Ok(Json(load_issue_dto(issue.id, extension.employee.is_owner()).await?))
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/checkout",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "Checkout an issue", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn checkout_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::checkout(issue_id.clone(), extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::CheckOut, extension.employee.id.clone(), extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  emit_issue_event(IssueEvent { issue_id: issue.id.clone(), kind: IssueEventKind::CheckedOut });

  Ok(Json(load_issue_dto(issue.id, extension.employee.is_owner()).await?))
}

#[utoipa::path(
  post,
  path = "/issues/{issue_id}/release",
  security(("blprnt_employee_id" = [])),
  params(("issue_id" = Uuid, Path, description = "Issue id")),
  responses(
    (status = 200, description = "Release an issue checkout", body = IssueDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Issue not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn release_issue(
  Extension(extension): Extension<RequestExtension>,
  Path(issue_id): Path<Uuid>,
) -> ApiResult<Json<IssueDto>> {
  let issue_id: IssueId = issue_id.into();
  let issue = IssueRepository::release(issue_id.clone(), extension.employee.id.clone()).await?;

  let model =
    IssueActionModel::new(issue.id.clone(), IssueActionKind::Release, extension.employee.id.clone(), extension.run_id);
  let _ = IssueRepository::add_action(model).await;

  emit_issue_event(IssueEvent { issue_id: issue.id.clone(), kind: IssueEventKind::Released });

  Ok(Json(load_issue_dto(issue.id, extension.employee.is_owner()).await?))
}

async fn stream_issues(ws: WebSocketUpgrade) -> impl IntoResponse {
  ws.on_upgrade(handle_issue_socket)
}

async fn handle_issue_socket(mut socket: WebSocket) {
  if send_issue_snapshot(&mut socket).await.is_err() {
    return;
  }

  let mut issue_events = ISSUE_EVENTS.subscribe();

  loop {
    tokio::select! {
      event = issue_events.recv() => {
        let Ok(event) = event else {
          break;
        };

        if send_issue_event_message(&mut socket, event).await.is_err() {
          break;
        }
      }
      message = socket.recv() => {
        match message {
          Some(Ok(Message::Close(_))) | None => break,
          Some(Ok(Message::Ping(payload))) => {
            if socket.send(Message::Pong(payload)).await.is_err() {
              break;
            }
          }
          Some(Ok(_)) => {}
          Some(Err(_)) => break,
        }
      }
    }
  }
}

async fn send_issue_snapshot(socket: &mut WebSocket) -> anyhow::Result<()> {
  let issues = IssueRepository::list(ListIssuesParams::default()).await?.into_iter().map(Into::into).collect();
  let message = IssueStreamMessageDto::Snapshot { snapshot: IssueStreamSnapshotDto { issues } };
  socket.send(Message::Text(serde_json::to_string(&message)?.into())).await?;
  Ok(())
}

async fn send_issue_event_message(socket: &mut WebSocket, event: IssueEvent) -> anyhow::Result<()> {
  let issue = load_issue_dto(event.issue_id, true).await?;
  let message = IssueStreamMessageDto::Upsert { kind: IssueEventKindDto::from(event.kind), issue };
  socket.send(Message::Text(serde_json::to_string(&message)?.into())).await?;
  Ok(())
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

    assert!(binding.contains("status: IssueStatus"), "{binding}");
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
