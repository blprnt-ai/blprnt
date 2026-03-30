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
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunFilter;
use persistence::prelude::RunRepository;
use persistence::prelude::RunStatus;
use persistence::prelude::RunTrigger;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

use crate::dto::RunDto;
use crate::dto::RunStreamMessageDto;
use crate::dto::RunStreamSnapshotDto;
use crate::dto::RunSummaryDto;
use crate::dto::RunSummaryPageDto;
use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/runs", get(list_runs))
    .route("/runs/{run_id}", get(get_run))
    .route("/runs", post(trigger_run))
    .route("/runs/{run_id}/cancel", delete(cancel_run))
    .route("/runs/stream", get(stream_runs))
    .layer(middleware::from_fn(owner_only))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct RunsPageQuery {
  employee: Option<EmployeeId>,
  status:   Option<RunStatus>,
  page:     Option<u32>,
  per_page: Option<u32>,
}

async fn list_runs(Query(query): Query<RunsPageQuery>) -> ApiResult<Json<RunSummaryPageDto>> {
  let page = query.page.unwrap_or(1).max(1);
  let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
  let offset = ((page - 1) * per_page) as usize;
  let filter = RunFilter { employee: query.employee, status: query.status, trigger: None };
  let items = RunRepository::list_summaries(filter.clone(), Some(per_page as usize), Some(offset))
    .await?
    .into_iter()
    .map(Into::into)
    .collect();
  let total = RunRepository::count(filter).await?;
  let total_pages = ((total as f64) / (per_page as f64)).ceil().max(1.0) as u32;

  Ok(Json(RunSummaryPageDto { items, page, per_page, total, total_pages }))
}

async fn get_run(Path(run_id): Path<Uuid>) -> ApiResult<Json<RunDto>> {
  Ok(Json(RunRepository::get(run_id.into()).await?.into()))
}

async fn cancel_run(Path(run_id): Path<Uuid>) -> ApiResult<StatusCode> {
  let run = RunRepository::get(run_id.into()).await?;

  API_EVENTS.emit(ApiEvent::CancelRun { employee_id: run.employee_id, run_id: run.id })?;

  Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct TriggerRunPayload {
  employee_id: EmployeeId,
}

async fn trigger_run(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<TriggerRunPayload>,
) -> ApiResult<Json<RunDto>> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to trigger runs")).into());
  }

  let (tx, rx) = oneshot::channel();
  API_EVENTS.emit(ApiEvent::StartRun {
    employee_id: payload.employee_id,
    trigger:     RunTrigger::Manual,
    rx:          Some(Arc::new(Mutex::new(Some(tx)))),
  })?;

  let run =
    rx.await.map_err(|_| ApiErrorKind::InternalServerError(serde_json::json!("Failed to receive run result")))??;

  match run {
    Some(run) => Ok(Json(run.into())),
    None => unreachable!("only RunTrigger::IssueAssignment can return None"),
  }
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
  let recent_runs =
    RunRepository::list_summaries(RunFilter { employee: None, status: None, trigger: None }, Some(5), Some(0))
      .await?
      .into_iter()
      .map(Into::into)
      .collect();
  let running_summary_records = RunRepository::list_summaries(
    RunFilter { employee: None, status: Some(RunStatus::Running), trigger: None },
    Some(25),
    Some(0),
  )
  .await?;
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
  let summary = RunSummaryDto {
    id:           run_record.id.uuid(),
    employee_id:  run_record.employee_id.uuid(),
    status:       run_record.status.clone(),
    trigger:      run_record.trigger.clone(),
    created_at:   run_record.created_at,
    started_at:   run_record.started_at,
    completed_at: run_record.completed_at,
  };
  let run: RunDto = run_record.into();

  let summary_message = RunStreamMessageDto::SummaryUpsert { run: summary };
  socket.send(Message::Text(serde_json::to_string(&summary_message)?.into())).await?;

  let detail_message = RunStreamMessageDto::DetailUpsert { run };
  socket.send(Message::Text(serde_json::to_string(&detail_message)?.into())).await?;

  Ok(())
}
