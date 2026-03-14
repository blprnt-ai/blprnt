use std::str::FromStr;
use std::sync::Arc;

use common::agent::AgentKind;
use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use common::models::ReasoningEffort;
use common::shared::prelude::DeleteQueuedPromptOutcome;
use common::shared::prelude::DeleteQueuedPromptRequest;
use common::shared::prelude::QueueMode;
use common::skills_utils::SkillsUtils;
use common::tools::SkillItem;
use common::tools::TerminalSnapshot;
use common::tools::question::AskQuestionAnswerSource;
use common::tools::question::AskQuestionClaimResult;
use persistence::prelude::MessageRecord;
use persistence::prelude::SessionModelV2;
use persistence::prelude::SessionPatchV2;
use persistence::prelude::SessionRecord;
use persistence::prelude::SurrealId;
use surrealdb::types::Uuid;
use tauri::State;

use crate::engine_manager::EngineManager;
use crate::engine_manager::SessionRecordDto;

#[tauri::command]
#[specta::specta]
pub async fn session_list(
  manager: State<'_, Arc<EngineManager>>,
  project_id: String,
) -> TauriResult<Vec<SessionRecord>> {
  tracing::debug!("List Sessions");
  let project_id = project_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_list(project_id).await.map(|sessions| sessions.into_iter().collect()).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_get(manager: State<'_, Arc<EngineManager>>, session_id: String) -> TauriResult<SessionRecord> {
  tracing::debug!("Get Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_get(session_id).await.into_tauri()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SessionCreateParams {
  pub project_id:         String,
  #[serde(default, alias = "personality_id")]
  pub personality_key:    Option<String>,
  pub name:               String,
  pub description:        String,
  pub agent_kind:         AgentKind,
  pub yolo:               bool,
  pub read_only:          bool,
  pub network_access:     bool,
  pub model_override:     String,
  pub web_search_enabled: Option<bool>,
  pub reasoning_effort:   ReasoningEffort,
  pub queue_mode:         QueueMode,
}

impl From<SessionCreateParams> for SessionModelV2 {
  fn from(params: SessionCreateParams) -> Self {
    Self {
      personality_key: params.personality_key,
      name: params.name,
      description: if params.description.is_empty() { None } else { Some(params.description) },
      agent_kind: params.agent_kind,
      yolo: params.yolo,
      read_only: params.read_only,
      network_access: params.network_access,
      model_override: params.model_override,
      web_search_enabled: params.web_search_enabled,
      reasoning_effort: params.reasoning_effort,
      queue_mode: Some(params.queue_mode),
      ..Default::default()
    }
  }
}

#[tauri::command]
#[specta::specta]
pub async fn session_create(
  manager: State<'_, Arc<EngineManager>>,
  params: SessionCreateParams,
) -> TauriResult<SessionRecord> {
  tracing::debug!("New Session");
  manager.session_create(params).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_update(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  mut session_patch: SessionPatchV2,
) -> TauriResult<SessionRecord> {
  tracing::debug!("Edit Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;

  if session_patch.model_override.is_none() {
    session_patch.model_override = Some("REMOVE".to_string());
  }

  manager.session_update(session_id, session_patch).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_start(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
) -> TauriResult<SessionRecordDto> {
  tracing::debug!("Start Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_start(session_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_stop(manager: State<'_, Arc<EngineManager>>, session_id: String) -> TauriResult<()> {
  tracing::debug!("Close Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_stop(session_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_delete(manager: State<'_, Arc<EngineManager>>, session_id: String) -> TauriResult<()> {
  tracing::debug!("Delete Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.delete_session(session_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn send_interrupt(manager: State<'_, Arc<EngineManager>>, session_id: String) -> TauriResult<()> {
  tracing::debug!("Send Interrupt: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.send_interrupt(session_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn send_prompt(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  prompt: String,
  image_urls: Option<Vec<String>>,
) -> TauriResult<()> {
  tracing::debug!("Send Prompt: {:?}", prompt);
  tracing::debug!("Image URLs: {:?}", image_urls);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.send_prompt(session_id, prompt, image_urls).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_queued_prompt(
  manager: State<'_, Arc<EngineManager>>,
  request: DeleteQueuedPromptRequest,
) -> TauriResult<DeleteQueuedPromptOutcome> {
  tracing::debug!("Delete Queued Prompt: {:?}", request.queue_item_id);
  let session_id = request.session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.delete_queued_prompt(session_id, request.queue_item_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn start_plan(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Start plan build: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.start_plan(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn continue_plan(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Continue plan build: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.continue_plan(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn complete_plan(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Complete plan: {:?}", plan_id);
  let session_id: SurrealId = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.complete_plan(session_id.clone(), plan_id.clone()).await.into_tauri()?;
  manager.unassign_plan_from_session(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_plan(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Cancel plan: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.cancel_plan(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_plan(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Delete plan: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.delete_plan(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn assign_plan_to_session(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Assign plan to session: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.assign_plan_to_session(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn unassign_plan_from_session(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  plan_id: String,
) -> TauriResult<()> {
  tracing::debug!("Unassign plan from session: {:?}", plan_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.unassign_plan_from_session(session_id, plan_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn rewind_to(manager: State<'_, Arc<EngineManager>>, session_id: String, id: SurrealId) -> TauriResult<()> {
  tracing::debug!("Rewind To: {:?}", id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.rewind_to(session_id, id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn session_history(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
) -> TauriResult<Vec<MessageRecord>> {
  tracing::debug!("Session History: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_history(session_id).await.map(|history| history.into_iter().collect()).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn delete_message(manager: State<'_, Arc<EngineManager>>, message_id: String) -> TauriResult<()> {
  tracing::debug!("Delete Message: {:?}", message_id);

  if SurrealId::looks_like(message_id.clone()) {
    let message_id = message_id.clone().try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
    manager.delete_message(message_id).await.into_tauri()
  } else {
    manager.delete_message_by_tool_call_id(message_id).await.into_tauri()
  }
}

#[tauri::command]
#[specta::specta]
pub async fn session_rename(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  new_name: String,
) -> TauriResult<()> {
  tracing::debug!("Rename Session: {:?}", session_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.session_rename(session_id, new_name).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn answer_question(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  question_id: String,
  answer: String,
) -> TauriResult<AskQuestionClaimResult> {
  tracing::debug!("Answer Question: {:?}", question_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.answer_question(session_id, question_id, answer, AskQuestionAnswerSource::Desktop).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn list_skills() -> TauriResult<Vec<SkillItem>> {
  tracing::debug!("List Skills");

  SkillsUtils::list_skills().into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn set_queue_mode(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  mode: QueueMode,
) -> TauriResult<()> {
  tracing::debug!("Set Queue Mode: {:?}", mode);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  manager.set_queue_mode(session_id, mode).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn close_terminal(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  terminal_id: String,
) -> TauriResult<()> {
  tracing::debug!("Close Terminal: {:?}", terminal_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  let terminal_id =
    Uuid::from_str(&terminal_id).map_err(|e| anyhow::anyhow!("Invalid terminal ID: {}", e)).into_tauri()?;
  manager.close_terminal(session_id, terminal_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn get_terminal_snapshot(
  manager: State<'_, Arc<EngineManager>>,
  session_id: String,
  terminal_id: String,
) -> TauriResult<TerminalSnapshot> {
  tracing::debug!("Get Terminal Snapshot: {:?}", terminal_id);
  let session_id = session_id.try_into().map_err(|e: anyhow::Error| e).into_tauri()?;
  let terminal_id =
    Uuid::from_str(&terminal_id).map_err(|e| anyhow::anyhow!("Invalid terminal ID: {}", e)).into_tauri()?;
  manager.get_terminal_snapshot(session_id, terminal_id).await.into_tauri()
}
