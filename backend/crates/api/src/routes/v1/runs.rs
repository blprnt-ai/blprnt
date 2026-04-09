use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use events::ADAPTER_EVENTS;
use events::API_EVENTS;
use events::AdapterEvent;
use events::ApiEvent;
use persistence::Uuid;
use persistence::prelude::ContentsVisibility;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ReasoningEffort;
use persistence::prelude::RunEnabledMcpServerRepository;
use persistence::prelude::RunFilter;
use persistence::prelude::RunId;
use persistence::prelude::RunModel;
use persistence::prelude::RunRepository;
use persistence::prelude::RunStatus;
use persistence::prelude::RunTrigger;
use persistence::prelude::TurnModel;
use persistence::prelude::TurnRepository;
use persistence::prelude::TurnStepContent;
use persistence::prelude::TurnStepText;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

use crate::dto::RunDto;
use crate::dto::PublicRunTrigger;
use crate::dto::RunStreamMessageDto;
use crate::dto::RunStreamSnapshotDto;
use crate::dto::RunSummaryDto;
use crate::dto::RunSummaryPageDto;
use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  let protected_routes = Router::new()
    .route("/runs", post(trigger_run))
    .route("/runs/{run_id}/messages", post(append_message))
    .route("/runs/{run_id}/cancel", delete(cancel_run))
    .route("/runs/stream", get(stream_runs))
    .layer(middleware::from_fn(owner_only));

  let public_routes = Router::new().route("/runs", get(list_runs)).route("/runs/{run_id}", get(get_run));

  Router::new().merge(protected_routes).merge(public_routes)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::IntoParams)]
#[ts(export)]
pub(super) struct RunsPageQuery {
  #[param(value_type = Option<String>)]
  employee: Option<Uuid>,
  status:   Option<RunStatus>,
  page:     Option<u32>,
  per_page: Option<u32>,
}

#[utoipa::path(
  get,
  path = "/runs",
  security(("blprnt_employee_id" = [])),
  params(RunsPageQuery),
  responses(
    (status = 200, description = "List runs", body = RunSummaryPageDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "runs"
)]
pub(super) async fn list_runs(Query(query): Query<RunsPageQuery>) -> ApiResult<Json<RunSummaryPageDto>> {
  let employee: Option<EmployeeId> = query.employee.map(Into::into);
  let page = query.page.unwrap_or(1).max(1);
  let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
  let offset = ((page - 1) * per_page) as usize;
  let filter = RunFilter { employee, issue: None, status: query.status, trigger: None };
  let items = RunRepository::list_summaries(filter.clone(), Some(per_page as usize), Some(offset))
    .await?
    .into_iter()
    .filter(|run| !matches!(run.trigger, RunTrigger::Dreaming))
    .map(Into::into)
    .collect();
  let total = RunRepository::count(filter).await?;
  let total_pages = ((total as f64) / (per_page as f64)).ceil().max(1.0) as u32;

  Ok(Json(RunSummaryPageDto { items, page, per_page, total, total_pages }))
}

#[utoipa::path(
  get,
  path = "/runs/{run_id}",
  security(("blprnt_employee_id" = [])),
  params(("run_id" = Uuid, Path, description = "Run id")),
  responses(
    (status = 200, description = "Fetch a run", body = RunDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Run not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "runs"
)]
pub(super) async fn get_run(Path(run_id): Path<Uuid>) -> ApiResult<Json<RunDto>> {
  let run_id: RunId = run_id.into();
  let run = RunRepository::get(run_id.clone()).await?;
  if matches!(run.trigger, RunTrigger::Dreaming) {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Run not found")).into());
  }
  let mut dto: RunDto = run.into();
  dto.enabled_mcp_servers =
    RunEnabledMcpServerRepository::list_for_run(run_id).await?.into_iter().map(Into::into).collect();
  Ok(Json(dto))
}

pub(super) async fn cancel_run(Path(run_id): Path<Uuid>) -> ApiResult<StatusCode> {
  let run = RunRepository::get(run_id.into()).await?;

  API_EVENTS.emit(ApiEvent::CancelRun { employee_id: run.employee_id, run_id: run.id })?;

  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TriggerRunPayload {
  pub employee_id:      Uuid,
  #[serde(default)]
  pub trigger:          Option<PublicRunTrigger>,
  #[serde(default)]
  pub prompt:           Option<String>,
  #[serde(default)]
  pub reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AppendRunMessagePayload {
  pub prompt:           String,
  #[serde(default)]
  pub reasoning_effort: Option<ReasoningEffort>,
}

pub(crate) async fn trigger_run(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<TriggerRunPayload>,
) -> ApiResult<Json<RunDto>> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to trigger runs")).into());
  }

  let trigger = payload.trigger.unwrap_or(PublicRunTrigger::Manual);

  match trigger {
    PublicRunTrigger::Conversation => {
      let prompt = payload
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiErrorKind::BadRequest(serde_json::json!("Conversation runs require a prompt")))?;

      let run = RunRepository::create(RunModel::new(payload.employee_id.into(), RunTrigger::Conversation)).await?;
      seed_run_turn(&run.id, prompt, payload.reasoning_effort).await?;

      let _ = API_EVENTS.emit(ApiEvent::StartRun {
        employee_id: run.employee_id.clone(),
        run_id:      Some(run.id.clone()),
        trigger:     RunTrigger::Conversation,
        rx:          None,
      });

      Ok(Json(RunRepository::get(run.id).await?.into()))
    }
    PublicRunTrigger::Manual => {
      let (tx, rx) = oneshot::channel();
      API_EVENTS.emit(ApiEvent::StartRun {
        employee_id: payload.employee_id.into(),
        run_id:      None,
        trigger:     RunTrigger::Manual,
        rx:          Some(Arc::new(Mutex::new(Some(tx)))),
      })?;

      let run = rx
        .await
        .map_err(|_| ApiErrorKind::InternalServerError(serde_json::json!("Failed to receive run result")))??;

      match run {
        Some(run) => Ok(Json(run.into())),
        None => unreachable!("only wake-on-demand gated triggers can return None"),
      }
    }
    PublicRunTrigger::Timer | PublicRunTrigger::IssueAssignment { .. } | PublicRunTrigger::IssueMention { .. } => {
      Err(
        ApiErrorKind::BadRequest(serde_json::json!("This run trigger cannot be created from the runs endpoint")).into(),
      )
    }
  }
}

pub(crate) async fn append_message(
  Path(run_id): Path<Uuid>,
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<AppendRunMessagePayload>,
) -> ApiResult<Json<RunDto>> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to continue runs")).into());
  }

  let prompt = payload.prompt.trim();
  if prompt.is_empty() {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Prompt cannot be empty")).into());
  }

  let run = RunRepository::get(run_id.into()).await?;
  if matches!(run.status, RunStatus::Pending | RunStatus::Running) {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Only inactive runs can be continued")).into());
  }

  seed_run_turn(&run.id, prompt, payload.reasoning_effort).await?;
  let run = RunRepository::update(run.id.clone(), RunStatus::Pending).await?;

  let _ = API_EVENTS.emit(ApiEvent::StartRun {
    employee_id: run.employee_id.clone(),
    run_id:      Some(run.id.clone()),
    trigger:     run.trigger.clone(),
    rx:          None,
  });

  Ok(Json(RunRepository::get(run.id).await?.into()))
}

async fn seed_run_turn(run_id: &RunId, prompt: &str, reasoning_effort: Option<ReasoningEffort>) -> ApiResult<()> {
  let turn =
    TurnRepository::create(TurnModel { run_id: run_id.clone(), reasoning_effort, ..Default::default() }).await?;
  TurnRepository::insert_step_content(
    turn.id,
    persistence::prelude::TurnStepSide::Request,
    TurnStepContent::Text(TurnStepText {
      text:       prompt.to_string(),
      signature:  None,
      visibility: ContentsVisibility::Full,
    }),
  )
  .await?;

  Ok(())
}

async fn stream_runs(ws: WebSocketUpgrade) -> impl IntoResponse {
  ws.on_upgrade(handle_runs_socket)
}

async fn handle_runs_socket(mut socket: WebSocket) {
  if send_snapshot(&mut socket).await.is_err() {
    return;
  }

  let mut adapter_events = ADAPTER_EVENTS.subscribe();

  loop {
    tokio::select! {
      event = adapter_events.recv() => {
        let Ok(event) = event else {
          break;
        };

        if send_event_message(&mut socket, event).await.is_err() {
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

async fn send_snapshot(socket: &mut WebSocket) -> anyhow::Result<()> {
  let recent_runs = RunRepository::list_summaries(
    RunFilter { employee: None, issue: None, status: None, trigger: None },
    Some(5),
    Some(0),
  )
  .await?
  .into_iter()
  .filter(|run| !matches!(run.trigger, RunTrigger::Dreaming))
  .map(Into::into)
  .collect();
  let running_summary_records = RunRepository::list_summaries(
    RunFilter { employee: None, issue: None, status: Some(RunStatus::Running), trigger: None },
    Some(25),
    Some(0),
  )
  .await?
  .into_iter()
  .filter(|run| !matches!(run.trigger, RunTrigger::Dreaming))
  .collect::<Vec<_>>();
  let running_run_ids = running_summary_records.iter().map(|run| run.id.clone()).collect::<Vec<_>>();
  let running_runs = running_summary_records.into_iter().map(Into::into).collect();
  let mut running_run_details = Vec::new();
  for run_id in running_run_ids {
    running_run_details.push(RunRepository::get(run_id).await?.into());
  }

  let message =
    RunStreamMessageDto::Snapshot { snapshot: RunStreamSnapshotDto { recent_runs, running_runs, running_run_details } };

  socket.send(Message::Text(serde_json::to_string(&message)?.into())).await?;
  Ok(())
}

async fn send_event_message(socket: &mut WebSocket, event: AdapterEvent) -> anyhow::Result<()> {
  let run_id = match &event {
    AdapterEvent::RunStarted { run_id }
    | AdapterEvent::RunCompleted { run_id }
    | AdapterEvent::RunCancelled { run_id }
    | AdapterEvent::RunFailed { run_id, .. }
    | AdapterEvent::ResponseDelta { run_id, .. }
    | AdapterEvent::Response { run_id, .. }
    | AdapterEvent::ThinkingDelta { run_id, .. }
    | AdapterEvent::Thinking { run_id, .. }
    | AdapterEvent::ToolDone { run_id, .. } => run_id.clone(),
  };

  let run_record = RunRepository::get(run_id).await?;
  if matches!(run_record.trigger, RunTrigger::Dreaming) {
    return Ok(());
  }
  let summary = RunSummaryDto {
    id:                  run_record.id.uuid(),
    employee_id:         run_record.employee_id.uuid(),
    status:              run_record.status.clone(),
    trigger:             run_record.trigger.clone().into(),
    enabled_mcp_servers: RunEnabledMcpServerRepository::list_for_run(run_record.id.clone())
      .await?
      .into_iter()
      .map(Into::into)
      .collect(),
    usage:               run_record.usage.clone(),
    created_at:          run_record.created_at,
    started_at:          run_record.started_at,
    completed_at:        run_record.completed_at,
  };
  let run: RunDto = run_record.into();

  let summary_message = RunStreamMessageDto::SummaryUpsert { run: summary };
  socket.send(Message::Text(serde_json::to_string(&summary_message)?.into())).await?;

  let detail_message = RunStreamMessageDto::DetailUpsert { run };
  socket.send(Message::Text(serde_json::to_string(&detail_message)?.into())).await?;

  Ok(())
}
