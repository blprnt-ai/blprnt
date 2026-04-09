use std::collections::HashMap;
use std::collections::HashSet;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::RawQuery;
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
use persistence::prelude::IssueLabel;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ListIssuesParams;
use persistence::prelude::ListIssuesSortBy;
use persistence::prelude::ListIssuesSortOrder;
use persistence::prelude::ProjectRepository;
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
use crate::dto::MyWorkItemDto;
use crate::dto::MyWorkReasonDto;
use crate::dto::MyWorkResponseDto;
use crate::dto::RunSummaryDto;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

fn deserialize_nullable_patch_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

fn parse_list_issues_sort_by(value: &str) -> Option<ListIssuesSortBy> {
  match value {
    "priority" => Some(ListIssuesSortBy::Priority),
    "created_at" => Some(ListIssuesSortBy::CreatedAt),
    "updated_at" => Some(ListIssuesSortBy::UpdatedAt),
    "title" => Some(ListIssuesSortBy::Title),
    "status" => Some(ListIssuesSortBy::Status),
    _ => None,
  }
}

fn parse_list_issues_sort_order(value: &str) -> Option<ListIssuesSortOrder> {
  match value {
    "asc" => Some(ListIssuesSortOrder::Asc),
    "desc" => Some(ListIssuesSortOrder::Desc),
    _ => None,
  }
}

fn parse_list_issues_params(raw_query: Option<&str>) -> ApiResult<ListIssuesParams> {
  let mut params = ListIssuesParams::default();

  for (key, value) in url::form_urlencoded::parse(raw_query.unwrap_or_default().as_bytes()) {
    match key.as_ref() {
      "expected_statuses" | "expected_statuses[]" => {
        let status = value.parse().map_err(|_| {
          ApiErrorKind::BadRequest(serde_json::json!({
            "parameter": key.as_ref(),
            "value": value.as_ref(),
            "message": format!("Invalid issue status: {}", value),
          }))
        })?;
        params.expected_statuses.get_or_insert_with(Vec::new).push(status);
      }
      "assignee" => {
        let assignee = match value.parse::<Uuid>() {
          Ok(assignee) => assignee,
          Err(error) => {
            return Err(
              ApiErrorKind::BadRequest(serde_json::json!({
                "parameter": "assignee",
                "value": value.as_ref(),
                "message": error.to_string(),
              }))
              .into(),
            );
          }
        };
        params.assignee = Some(assignee);
      }
      "label" => {
        params.label = Some(value.into_owned());
      }
      "page" => {
        params.page = Some(value.parse().map_err(|error: std::num::ParseIntError| {
          ApiErrorKind::BadRequest(serde_json::json!({
            "parameter": "page",
            "value": value.as_ref(),
            "message": error.to_string(),
          }))
        })?);
      }
      "page_size" => {
        params.page_size = Some(value.parse().map_err(|error: std::num::ParseIntError| {
          ApiErrorKind::BadRequest(serde_json::json!({
            "parameter": "page_size",
            "value": value.as_ref(),
            "message": error.to_string(),
          }))
        })?);
      }
      "sort_by" => {
        params.sort_by = Some(parse_list_issues_sort_by(value.as_ref()).ok_or_else(|| {
          ApiErrorKind::BadRequest(serde_json::json!({
            "parameter": "sort_by",
            "value": value.as_ref(),
            "message": format!("Invalid issue sort field: {}", value),
          }))
        })?);
      }
      "sort_order" => {
        params.sort_order = Some(parse_list_issues_sort_order(value.as_ref()).ok_or_else(|| {
          ApiErrorKind::BadRequest(serde_json::json!({
            "parameter": "sort_order",
            "value": value.as_ref(),
            "message": format!("Invalid issue sort order: {}", value),
          }))
        })?);
      }
      _ => {}
    }
  }

  Ok(params)
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
    .route("/issues/my-work", get(get_my_work))
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

fn is_my_work_visible_status(status: &IssueStatus) -> bool {
  !matches!(status, IssueStatus::Done | IssueStatus::Archived | IssueStatus::Cancelled)
}

pub(crate) fn build_comment_snippet_for_labels(comment: &str, labels: &[&str]) -> String {
  const MAX_CHARS: usize = 110;
  let normalized = comment.split_whitespace().collect::<Vec<_>>().join(" ");
  if normalized.is_empty() {
    return normalized;
  }

  let mention_index = labels.iter().filter_map(|label| normalized.find(&format!("@{label}"))).min();

  let total_chars = normalized.chars().count();
  let start_char_index =
    mention_index.map(|byte_index| normalized[..byte_index].chars().count().saturating_sub(MAX_CHARS / 3)).unwrap_or(0);
  let end_char_index = (start_char_index + MAX_CHARS).min(total_chars);

  let snippet: String = normalized.chars().skip(start_char_index).take(end_char_index - start_char_index).collect();

  let mut decorated = snippet;
  if start_char_index > 0 {
    decorated = format!("…{decorated}");
  }
  if end_char_index < total_chars {
    decorated.push('…');
  }

  decorated
}

fn mention_boundary_before(text: &str, start: usize) -> bool {
  if start == 0 {
    return true;
  }

  matches!(text[..start].chars().next_back(), Some(ch) if ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '-'))
}

fn mention_boundary_after(text: &str, end: usize) -> bool {
  if end >= text.len() {
    return true;
  }

  matches!(
    text[end..].chars().next(),
    Some(ch) if ch.is_whitespace() || matches!(ch, ')' | ']' | '}' | '.' | '!' | '?' | ',' | ':' | ';' | '-')
  )
}

fn contains_employee_mention(text: &str, label: &str) -> bool {
  let token = format!("@{label}");
  let mut cursor = 0;

  while let Some(offset) = text[cursor..].find(&token) {
    let start = cursor + offset;
    let end = start + token.len();

    if mention_boundary_before(text, start) && mention_boundary_after(text, end) {
      return true;
    }

    cursor = end;
  }

  false
}

async fn merge_inferred_comment_mentions(
  comment: &str,
  mentions: Vec<IssueCommentMention>,
) -> anyhow::Result<Vec<IssueCommentMention>> {
  let employees = EmployeeRepository::list().await?;
  let mut seen_employee_ids = mentions.iter().map(|mention| mention.employee_id.clone()).collect::<HashSet<_>>();
  let mut merged_mentions = mentions;

  for employee in employees {
    if seen_employee_ids.contains(&employee.id) {
      continue;
    }

    if contains_employee_mention(comment, &employee.name) {
      seen_employee_ids.insert(employee.id.clone());
      merged_mentions.push(IssueCommentMention { employee_id: employee.id, label: employee.name });
    }
  }

  Ok(merged_mentions)
}

#[utoipa::path(
  get,
  path = "/issues/my-work",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List current-user My Work items", body = MyWorkResponseDto),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "issues"
)]
pub(super) async fn get_my_work(
  Extension(extension): Extension<RequestExtension>,
) -> ApiResult<Json<MyWorkResponseDto>> {
  let employee_id = extension.employee.id.clone();
  let projects = ProjectRepository::list().await?;
  let project_names: HashMap<_, _> = projects.into_iter().map(|project| (project.id.uuid(), project.name)).collect();

  let assigned_issues = IssueRepository::list(ListIssuesParams {
    assignee: Some(employee_id.uuid()),
    expected_statuses: Some(vec![
      IssueStatus::Backlog,
      IssueStatus::Todo,
      IssueStatus::InProgress,
      IssueStatus::Blocked,
    ]),
    sort_by: Some(ListIssuesSortBy::UpdatedAt),
    sort_order: Some(ListIssuesSortOrder::Desc),
    ..Default::default()
  })
  .await?;

  let assigned = assigned_issues
    .into_iter()
    .filter(|issue| is_my_work_visible_status(&issue.status))
    .map(|issue| MyWorkItemDto {
      issue_id:         issue.id.uuid(),
      issue_identifier: format!("{}-{}", issue.identifier, issue.issue_number),
      title:            issue.title,
      project_id:       issue.project.as_ref().map(|project| project.uuid()),
      project_name:     issue.project.as_ref().and_then(|project| project_names.get(&project.uuid()).cloned()),
      status:           issue.status,
      priority:         issue.priority,
      reason:           MyWorkReasonDto::Assigned,
      relevant_at:      issue.updated_at,
      comment_id:       None,
      comment_snippet:  None,
    })
    .collect::<Vec<_>>();

  let mut assigned = assigned;
  assigned.sort_by(|left, right| right.relevant_at.cmp(&left.relevant_at));

  let all_issues = IssueRepository::list(ListIssuesParams {
    expected_statuses: Some(vec![
      IssueStatus::Backlog,
      IssueStatus::Todo,
      IssueStatus::InProgress,
      IssueStatus::Blocked,
    ]),
    sort_by: Some(ListIssuesSortBy::UpdatedAt),
    sort_order: Some(ListIssuesSortOrder::Desc),
    ..Default::default()
  })
  .await?;

  let mut newest_mentions_by_issue: HashMap<
    _,
    (persistence::prelude::IssueRecord, persistence::prelude::IssueCommentRecord),
  > = HashMap::new();

  for issue in all_issues {
    if !is_my_work_visible_status(&issue.status) {
      continue;
    }

    let comments = IssueRepository::list_comments(issue.id.clone()).await?;
    for comment in comments {
      let is_direct_mention = comment
        .mentions
        .as_ref()
        .map(|mentions| mentions.iter().any(|mention| mention.employee_id == employee_id))
        .unwrap_or(false);

      if !is_direct_mention {
        continue;
      }

      match newest_mentions_by_issue.get(&issue.id.uuid()) {
        Some((_, existing_comment)) if existing_comment.created_at >= comment.created_at => {}
        _ => {
          newest_mentions_by_issue.insert(issue.id.uuid(), (issue.clone(), comment));
        }
      }
    }
  }

  let mut mentioned = newest_mentions_by_issue
    .into_values()
    .map(|(issue, comment)| MyWorkItemDto {
      issue_id:         issue.id.uuid(),
      issue_identifier: format!("{}-{}", issue.identifier, issue.issue_number),
      title:            issue.title,
      project_id:       issue.project.as_ref().map(|project| project.uuid()),
      project_name:     issue.project.as_ref().and_then(|project| project_names.get(&project.uuid()).cloned()),
      status:           issue.status,
      priority:         issue.priority,
      reason:           MyWorkReasonDto::Mentioned,
      relevant_at:      comment.created_at,
      comment_id:       Some(comment.id.uuid()),
      comment_snippet:  Some(build_comment_snippet_for_labels(&comment.comment, &[&extension.employee.name])),
    })
    .collect::<Vec<_>>();

  mentioned.sort_by(|left, right| right.relevant_at.cmp(&left.relevant_at));

  Ok(Json(MyWorkResponseDto { assigned, mentioned }))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, optional_fields = nullable)]
pub(crate) struct CreateIssuePayload {
  pub title:       String,
  pub description: String,
  pub labels:      Option<Vec<IssueLabel>>,
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
      labels: payload.labels,
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
    RunFilter { employee: None, issue: Some(issue_id), status: None, trigger: None },
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
pub(super) async fn list_issues(RawQuery(raw_query): RawQuery) -> ApiResult<Json<Vec<IssueDto>>> {
  let mut params = parse_list_issues_params(raw_query.as_deref())?;

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
  pub labels:      Option<Option<Vec<IssueLabel>>>,
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
      labels:      payload.labels,
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
  let mentions = merge_inferred_comment_mentions(
    &payload.comment,
    payload.mentions.into_iter().map(Into::into).collect::<Vec<IssueCommentMention>>(),
  )
  .await?;
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

  let mut reopened_assignee_id: Option<EmployeeId> = None;

  if should_reopen_issue {
    let issue = IssueRepository::get(issue_id.clone()).await?;
    if issue.status == IssueStatus::Done {
      let reopened_issue =
        IssueRepository::update(issue_id.clone(), IssuePatch { status: Some(IssueStatus::Todo), ..Default::default() })
          .await?;
      let model = IssueActionModel::new(
        issue_id.clone(),
        IssueActionKind::StatusChange { from: IssueStatus::Done, to: IssueStatus::Todo },
        extension.employee.id.clone(),
        extension.run_id.clone(),
      );
      let _ = IssueRepository::add_action(model).await;

      if reopened_issue.status.active() {
        reopened_assignee_id = reopened_issue.assignee.clone();
      }

      emit_issue_event(IssueEvent { issue_id: issue_id.clone(), kind: IssueEventKind::Updated });
    }
  }

  let mut triggered_employees = HashSet::new();

  if let Some(assignee_id) = reopened_assignee_id {
    triggered_employees.insert(assignee_id.clone());
    let _ = API_EVENTS.emit(ApiEvent::StartRun {
      employee_id: assignee_id,
      run_id:      None,
      trigger:     RunTrigger::IssueAssignment { issue_id: issue_id.clone() },
      rx:          None,
    });
  }

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

  if issue.status.active() {
    API_EVENTS.emit(ApiEvent::StartRun {
      employee_id,
      run_id: None,
      trigger: RunTrigger::IssueAssignment { issue_id },
      rx: None,
    })?;
  }

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

    assert!(binding.contains("labels?: Array<IssueLabel>"), "{binding}");
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
    assert!(binding.contains("labels?: Array<IssueLabel>"), "{binding}");
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
