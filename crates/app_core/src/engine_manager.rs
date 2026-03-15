mod memory_sweep;
pub(crate) mod provider_handler;
pub(crate) mod slack;

use std::collections::HashMap;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::agent::ToolId;
use common::blprnt::Blprnt;
use common::blprnt_dispatch::BlprntDispatch;
use common::blprnt_dispatch::SessionEvent;
use common::bun_runtime::BunRuntimeCommandState;
use common::bun_runtime::bun_runtime_install;
use common::bun_runtime::load_bun_runtime_status;
use common::errors::AppCoreError;
use common::errors::ErrorEvent;
use common::errors::ToolError;
use common::memory::QmdMemorySearchService;
use common::memory::detect_command;
use common::paths::BlprntPath;
use common::personality_service::PersonalityRecord;
use common::personality_service::PersonalityService;
use common::plan_utils::get_plan_content;
use common::plan_utils::get_plan_content_by_parent_session_id;
use common::plan_utils::parse_frontmatter;
use common::plan_utils::render_plan_content;
use common::plan_utils::resolve_plan_directory;
use common::session_dispatch::prelude::LlmEvent;
use common::session_dispatch::prelude::ResponseDone;
use common::session_dispatch::prelude::SessionDispatchEvent;
use common::session_dispatch::prelude::SlackEvent;
use common::session_dispatch::prelude::SlackInput;
use common::session_dispatch::prelude::ToolCallCompleted;
use common::session_dispatch::prelude::ToolCallStarted;
use common::shared::prelude::*;
use common::tools::PlanDocumentStatus;
use common::tools::PlanGetPayload;
use common::tools::TerminalSnapshot;
use common::tools::question::AskQuestionAnswerSource;
use common::tools::question::AskQuestionArgs;
use common::tools::question::AskQuestionClaimResult;
use engine_v2::prelude::*;
use persistence::prelude::*;
use sandbox::get_sandbox;
use surrealdb::types::Uuid;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tunnel_client::TunnelClient;

use crate::cmd::SessionCreateParams;
use crate::engine_manager::memory_sweep::MemorySweepCoordinator;
use crate::engine_manager::slack::SlackManager;
use crate::mcp_runtime::McpRuntimeManager;
use crate::preview::PreviewManager;
use crate::preview::PreviewSession;
use crate::preview::PreviewStartParams;
use crate::preview::PreviewStatusResponse;
use crate::tunnel_handler::handle_tunnel_request;

pub const AUTH_STORE: &str = "auth.json";
pub const BLPRNT_STORE: &str = "blprnt.json";
pub const TUNNEL_STORE: &str = "tunnel.json";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(serde::Serialize, specta::Type)]
pub struct SessionRecordDto {
  #[serde(flatten)]
  pub session: SessionRecord,
  pub status:  RuntimeState,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PersonalityModelDto {
  pub id:              String,
  pub name:            String,
  pub description:     String,
  pub system_prompt:   String,
  pub is_default:      bool,
  pub is_user_defined: bool,
  #[specta(type = String)]
  pub created_at:      DateTime<Utc>,
  #[specta(type = String)]
  pub updated_at:      DateTime<Utc>,
}

pub struct EngineManager {
  pub slack:       Arc<SlackManager>,
  controllers:     Arc<Mutex<HashMap<SurrealId, Arc<RwLock<Controller>>>>>,
  mcp_runtime:     Arc<McpRuntimeManager>,
  preview_manager: Arc<PreviewManager>,
  memory_sweep:    Mutex<HashMap<String, MemorySweepRuntime>>,
}

impl Default for EngineManager {
  fn default() -> Self {
    Self::new()
  }
}

struct MemorySweepRuntime {
  cancel_token: CancellationToken,
  join_handle:  JoinHandle<()>,
}

impl Drop for MemorySweepRuntime {
  fn drop(&mut self) {
    self.cancel_token.cancel();
    self.join_handle.abort();
  }
}

impl EngineManager {
  pub fn new() -> Self {
    BlprntDispatch::run();

    Self {
      slack:           Arc::new(SlackManager::new()),
      controllers:     Arc::new(Mutex::new(HashMap::new())),
      mcp_runtime:     Arc::new(McpRuntimeManager::new()),
      preview_manager: Arc::new(PreviewManager::new()),
      memory_sweep:    Mutex::new(HashMap::new()),
    }
  }

  pub fn get_tunnel_uuid() -> Uuid {
    let store = Blprnt::handle().store(TUNNEL_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string())).ok();

    if let Some(store) = store {
      if let Some(tunnel_uuid) = store.get("tunnel_uuid") {
        Uuid::from_str(tunnel_uuid.as_str().unwrap()).unwrap_or_default()
      } else {
        let tunnel_uuid = Uuid::new_v4();
        store.set("tunnel_uuid", tunnel_uuid.to_string());
        tunnel_uuid
      }
    } else {
      Uuid::new_v4()
    }
  }

  pub async fn init(self: &Arc<Self>) -> Result<()> {
    let manager = self.clone();

    self.spawn_slack_hook();

    tokio::spawn({
      let manager = manager.clone();
      async move {
        manager.slack.init().await;
      }
    });

    tokio::spawn({
      let manager = manager.clone();
      async move {
        manager.sync_mcp_runtime_from_store().await;
      }
    });

    tokio::spawn({
      let manager = manager.clone();
      async move {
        let Ok(bun_install) = bun_runtime_install(false).await else {
          return;
        };

        let bun_bin = if bun_install.status.bun.state == BunRuntimeCommandState::Available {
          bun_install.status.bun.command
        } else {
          bun_install.status.user_local_bun.command
        };

        let qmd = detect_command(BlprntPath::home().join(".bun").join("bin").join("qmd").to_string_lossy().as_ref());

        if !qmd {
          tracing::warn!("QMD runtime not found; installing...");
          let mut cmd = Command::new(bun_bin);
          cmd.arg("install").arg("-g").arg("@tobilu/qmd");

          #[cfg(windows)]
          cmd.creation_flags(CREATE_NO_WINDOW);

          let output = cmd.output();
          tracing::warn!("QMD installation output: {:?}", output);
        }

        let qmd = detect_command(BlprntPath::home().join(".bun").join("bin").join("qmd").to_string_lossy().as_ref());

        if !qmd {
          tracing::warn!("QMD runtime not found after installation; skipping memory sweep");
          return;
        }

        let _ = manager.init_memory_sweep().await;
      }
    });

    tokio::spawn({
      let manager = manager.clone();
      async move {
        let tunnel_uuid = Self::get_tunnel_uuid();
        let tunnel_client = TunnelClient::new("wss://relay.blprnt.ai", tunnel_uuid.to_string());
        let manager_for_requests = manager.clone();
        let result = tunnel_client
          .connect_with_lifecycle(
            move |request| {
              let manager = manager_for_requests.clone();
              async move { handle_tunnel_request(manager, request) }
            },
            || async {},
          )
          .await;
        tracing::warn!("Tunnel connection task ended: {:?}", result);
      }
    });

    Ok(())
  }

  fn spawn_slack_hook(self: &Arc<Self>) {
    let manager = self.clone();
    let mut rx = BlprntDispatch::get_or_init().tx.subscribe();

    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(event) => manager.handle_slack_event(event).await,
          Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
            tracing::warn!("Slack final-response hook lagged and skipped {skipped} events");
          }
          Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
      }
    });
  }

  async fn handle_slack_event(&self, event: SessionEvent) {
    if self.slack.is_app_focused() {
      return;
    }

    if let SessionDispatchEvent::Llm(LlmEvent::ToolCallStarted(tool_call_started)) = &event.event_data
      && tool_call_started.tool_id == ToolId::AskQuestion
    {
      self.handle_slack_ask_question_start(event.session_id.clone(), tool_call_started).await;
    }

    self.slack.try_deliver_all_queued_ask_questions().await;

    match &event.event_data {
      SessionDispatchEvent::Llm(LlmEvent::ResponseDone(response_done)) if event.parent_id.is_none() => {
        self.handle_slack_final_response_event(event.session_id, response_done).await;
      }
      SessionDispatchEvent::Llm(LlmEvent::ToolCallStarted(tool_call_started))
        if tool_call_started.tool_id == ToolId::SubAgent =>
      {
        tracing::info!(session_id = %event.session_id, "Handling Slack subagent start event");
        self.handle_slack_subagent_start(event.session_id, tool_call_started).await;
      }
      SessionDispatchEvent::Llm(LlmEvent::ToolCallCompleted(tool_call_completed))
        if tool_call_completed.content.get_tool_id() == ToolId::SubAgent =>
      {
        tracing::info!(session_id = %event.session_id, "Handling Slack subagent completed event");
        self.handle_slack_subagent_completed(event.session_id, tool_call_completed).await
      }
      SessionDispatchEvent::Slack(SlackEvent::Input(input)) => {
        tracing::info!(session_id = %event.session_id, "Handling Slack input event");
        let result = self.handle_slack_input(event.session_id.clone(), input).await;
        if let Err(error) = result {
          tracing::warn!(session_id = %event.session_id, "Failed to handle Slack input event: {error}");
        }
      }
      _ => {}
    }
  }

  async fn handle_slack_final_response_event(&self, session_id: SurrealId, response_done: &ResponseDone) {
    let Ok(message) = MessageRepositoryV2::get(response_done.id.clone()).await else {
      return;
    };

    if !Self::is_slack_eligible_final_assistant_message(&message) {
      return;
    }

    let Some(content) =
      message.content().as_text().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
    else {
      return;
    };

    let Ok(session) = SessionRepositoryV2::get(session_id.clone()).await else {
      return;
    };

    if let Err(error) =
      self.slack.send_final_assistant_response(session_id.to_string(), session.name().clone(), content).await
    {
      tracing::warn!(session_id = %session_id, "Failed to send final assistant response to Slack: {error}");
    }
  }

  async fn handle_slack_subagent_start(&self, session_id: SurrealId, tool_call_started: &ToolCallStarted) {
    let Some(subagent_name) = tool_call_started
      .args
      .get("name")
      .and_then(|value| value.as_str())
      .map(str::trim)
      .filter(|value| !value.is_empty())
      .map(str::to_string)
    else {
      return;
    };

    self.slack.cache_subagent_name(session_id.to_string(), subagent_name).await;
  }

  async fn handle_slack_ask_question_start(&self, session_id: SurrealId, tool_call_started: &ToolCallStarted) {
    tracing::info!(session_id = %session_id, "Handling Slack ask question start event");
    let Ok(args) = serde_json::from_value::<AskQuestionArgs>(tool_call_started.args.clone()) else {
      tracing::warn!(session_id = %session_id, tool_call_id = %tool_call_started.id, "Failed to parse ask_question args for Slack queue enqueue");
      return;
    };

    let session_id_value = session_id.to_string();
    let _ = self.slack.enqueue_ask_question_for_session(&session_id_value, args.details, args.options).await;
  }

  async fn handle_slack_subagent_completed(&self, session_id: SurrealId, tool_call_completed: &ToolCallCompleted) {
    let send_result = if tool_call_completed.content.is_ok() {
      self.slack.send_cached_subagent_completion(session_id.to_string()).await
    } else {
      self.slack.send_cached_subagent_failure(session_id.to_string()).await
    };

    if let Err(error) = send_result {
      tracing::warn!(session_id = %session_id, tool_call_id = %tool_call_completed.id, "Failed to send subagent Slack status: {error}");
    }
  }

  async fn handle_slack_input(&self, session_id: SurrealId, input: &SlackInput) -> Result<()> {
    if let Some(controller) = self.controllers.lock().await.get(&session_id) {
      let controller = controller.read().await;
      let _ = controller.push_prompt(input.text.clone(), None).await;
    }

    Ok(())
  }

  fn is_slack_eligible_final_assistant_message(message: &MessageRecord) -> bool {
    message.is_assistant() && !message.visibility().is_tool_request() && !message.visibility().is_tool_result()
  }

  async fn init_memory_sweep(self: &Arc<Self>) -> Result<()> {
    if !self.memory_features_enabled_on_app_start() {
      tracing::warn!("Memory features not enabled on app start; skipping memory sweep");
      return Ok(());
    }

    tracing::info!("Initializing memory sweep");

    let projects = ProjectRepositoryV2::list().await?;
    for project in projects {
      let project_id = project.id.key().to_string();
      let qmd = QmdMemorySearchService::new(project_id.clone());

      if let Err(e) = qmd.bootstrap_collections() {
        tracing::warn!(project_id = %project_id, "Failed to bootstrap collections: {e}");
        continue;
      }

      let coordinator = MemorySweepCoordinator::new(project_id.clone());
      if let Err(e) = coordinator.ensure_today_dir().await {
        tracing::warn!(project_id = %project_id, "Failed to ensure today dir: {e}");
        continue;
      }

      coordinator.run_boot_catch_up().await;

      let cancel_token = CancellationToken::new();
      let join_handle = tokio::spawn({
        let coordinator = coordinator.clone();
        let cancel_token = cancel_token.clone();
        async move {
          coordinator.run_periodic(cancel_token).await;
        }
      });

      let mut runtime = self.memory_sweep.lock().await;

      runtime.insert(project.id.key().to_string(), MemorySweepRuntime { cancel_token, join_handle });
    }

    Ok(())
  }

  fn memory_features_enabled_on_app_start(&self) -> bool {
    let status = load_bun_runtime_status();

    match status.bun.state {
      BunRuntimeCommandState::Available => true,
      _ => matches!(status.user_local_bun.state, BunRuntimeCommandState::Available),
    }
  }

  pub async fn mcp_statuses(&self) -> Vec<McpServerStatus> {
    self.mcp_runtime.statuses().await
  }

  pub fn mcp_runtime_bridge(&self) -> common::shared::prelude::McpRuntimeBridgeRef {
    self.mcp_runtime.clone()
  }

  pub async fn sync_mcp_runtime_from_store(self: &Arc<Self>) {
    let store = match Blprnt::handle().store(BLPRNT_STORE) {
      Ok(store) => store,
      Err(error) => {
        tracing::warn!("Failed to open config store for MCP runtime sync: {error}");
        Blprnt::emit_error(ErrorEvent::internal(format!("MCP runtime sync failed to open config store: {error}")));
        return;
      }
    };

    let servers = match store.get("mcp_servers") {
      Some(value) => match serde_json::from_value::<Vec<McpServerConfig>>(value) {
        Ok(servers) => servers,
        Err(error) => {
          tracing::warn!("Failed to parse persisted MCP server config; preserving current runtime state: {error}");
          Blprnt::emit_error(ErrorEvent::internal(format!(
            "MCP runtime sync parse failure; preserving current runtime state: {error}"
          )));
          return;
        }
      },
      None => Vec::new(),
    };

    self.mcp_runtime.sync_servers(servers).await;
  }

  // Project

  pub async fn create_project(&self, project: ProjectModelV2) -> Result<ProjectRecord> {
    ProjectRepositoryV2::create(project).await
  }

  pub async fn edit_project(&self, id: SurrealId, project: ProjectPatchV2) -> Result<ProjectRecord> {
    let project_model = ProjectRepositoryV2::update(id, project).await?;

    {
      let sandbox_key = format!("project_{}", project_model.id.clone().key());

      let sandbox = get_sandbox();
      let mut sandbox = sandbox.write().await;

      sandbox.remove_root(&sandbox_key);

      for root in project_model.working_directories().0.clone() {
        sandbox.add_dir(sandbox_key.clone(), &PathBuf::from(root)).await?;
      }
    }

    Ok(project_model)
  }

  pub async fn get_project(&self, project_id: SurrealId) -> Result<ProjectRecord> {
    ProjectRepositoryV2::get(project_id).await
  }

  pub async fn list_projects(&self) -> Result<Vec<ProjectRecord>> {
    ProjectRepositoryV2::list().await
  }

  pub async fn delete_project(&self, project_id: SurrealId) -> Result<()> {
    ProjectRepositoryV2::delete(project_id).await
  }

  pub async fn get_full_project(&self, project_id: SurrealId) -> Result<ProjectRecord> {
    ProjectRepositoryV2::get(project_id).await
  }

  // Preview
  pub async fn preview_start(&self, params: PreviewStartParams) -> Result<PreviewSession> {
    self.preview_manager.start_preview(params).await
  }

  pub async fn preview_stop(&self, project_id: String) -> Result<()> {
    self.preview_manager.stop_preview(project_id).await
  }

  pub async fn preview_reload(&self, project_id: String) -> Result<PreviewSession> {
    self.preview_manager.reload_preview(project_id).await
  }

  pub async fn preview_status(&self, project_id: String) -> Result<PreviewStatusResponse> {
    self.preview_manager.status(project_id).await
  }

  // Session
  pub async fn session_list(&self, project_id: SurrealId) -> Result<Vec<SessionRecord>> {
    SessionRepositoryV2::list(project_id).await
  }

  pub async fn session_get(&self, session_id: SurrealId) -> Result<SessionRecord> {
    SessionRepositoryV2::get(session_id).await
  }

  pub async fn session_create(&self, params: SessionCreateParams) -> Result<SessionRecord> {
    let project_id = params.project_id.clone();
    let session_model = SessionRepositoryV2::create(params.into(), project_id.try_into()?).await?;

    Ok(session_model)
  }

  pub async fn session_rename(&self, session_id: SurrealId, new_name: String) -> Result<()> {
    let session_patch = SessionPatchV2 { name: Some(new_name.clone()), ..Default::default() };
    let _ = SessionRepositoryV2::update(session_id, session_patch).await?;

    Ok(())
  }

  pub async fn session_update(&self, session_id: SurrealId, session_patch: SessionPatchV2) -> Result<SessionRecord> {
    tracing::info!("Session update: {:?}", session_patch);
    let session_model = SessionRepositoryV2::update(session_id.clone(), session_patch).await?;

    let controllers = self.controllers.lock().await;
    let controller = controllers.get(&session_id).unwrap();
    let _ = controller.read().await.set_queue_mode(session_model.queue_mode.clone().unwrap_or(QueueMode::Queue)).await;

    Ok(session_model)
  }

  pub async fn session_start(&self, session_id: SurrealId) -> Result<SessionRecordDto> {
    {
      let controllers = self.controllers.lock().await;
      if controllers.contains_key(&session_id) {
        let controller = controllers.get(&session_id).unwrap();

        let session_model = SessionRepositoryV2::get(session_id).await?;
        let session_patch = SessionPatchV2 { ..Default::default() };
        let session_model = SessionRepositoryV2::update(session_model.id, session_patch).await?;

        let status = controller.read().await.state().await;

        return Ok(SessionRecordDto { session: session_model, status });
      }
    }

    let session_model = self.init_engine(session_id).await?;

    Ok(SessionRecordDto { session: session_model, status: RuntimeState::Idle })
  }

  pub async fn delete_session(&self, session_id: SurrealId) -> Result<()> {
    let session_model = SessionRepositoryV2::get(session_id.clone()).await?;

    if let Some(attached_plan) =
      get_plan_content_by_parent_session_id(session_model.project.clone(), &session_id.to_string())?
    {
      let plan_directory = resolve_plan_directory(session_model.project.clone())?;
      let plan_path = PathBuf::from(&plan_directory.path).join(&attached_plan.id);
      let content = std::fs::read_to_string(&plan_path)
        .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
      let (mut frontmatter, body) = parse_frontmatter(&content)?;
      frontmatter.status = PlanDocumentStatus::Archived;
      frontmatter.parent_session_id = None;
      let updated_content = render_plan_content(&frontmatter, &body)?;
      std::fs::write(&plan_path, updated_content)
        .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    }

    let mut controllers = self.controllers.lock().await;
    let _ = controllers.remove(&session_id);

    SessionRepositoryV2::delete(session_id).await
  }

  pub async fn active_sessions(&self) -> Vec<SurrealId> {
    self.controllers.lock().await.keys().cloned().collect()
  }

  async fn init_engine(&self, session_id: SurrealId) -> Result<SessionRecord> {
    let session_model = SessionRepositoryV2::get(session_id.clone()).await?;
    tracing::info!("Init Engine, getting session model: {:?}", session_model);

    let sandbox_key = format!("project_{}", session_model.project.key());
    let project = ProjectRepositoryV2::get(session_model.project.clone()).await?;

    tracing::info!("Init Engine, getting sandbox key: {:?}", sandbox_key);

    {
      let sandbox = get_sandbox();
      let mut sandbox = sandbox.write().await;

      sandbox.remove_root(&sandbox_key);
      for root in project.working_directories().0.clone() {
        sandbox.add_dir(sandbox_key.clone(), &PathBuf::from(root)).await?;
      }
    }

    tracing::info!("Init Engine");

    let config = ControllerConfig {
      sandbox_key:          sandbox_key.clone(),
      is_subagent:          false,
      session_id:           session_id.clone(),
      parent_id:            None,
      mcp_runtime:          Some(self.mcp_runtime_bridge()),
      memory_tools_enabled: self.memory_features_enabled_on_app_start(),
    };
    let controller = Controller::new(config).await;
    let _ = controller.read().await.run().await;
    let _ = controller.read().await.set_queue_mode(session_model.queue_mode.clone().unwrap_or(QueueMode::Queue)).await;
    let mut engines = self.controllers.lock().await;
    engines.insert(session_id, controller.clone());

    Ok(session_model)
  }

  pub async fn session_history(&self, session_id: SurrealId) -> Result<Vec<MessageRecord>> {
    MessageRepositoryV2::list(session_id).await
  }

  pub async fn delete_message(&self, message_id: SurrealId) -> Result<()> {
    MessageRepositoryV2::delete(message_id).await
  }

  pub async fn delete_message_by_tool_call_id(&self, tool_call_id: String) -> Result<()> {
    let message = MessageRepositoryV2::get_by_rel_id(tool_call_id).await?;

    MessageRepositoryV2::delete(message.id).await
  }

  pub async fn session_stop(&self, session_id: SurrealId) -> Result<()> {
    let mut controllers = self.controllers.lock().await;

    if let Some(controller) = controllers.remove(&session_id) {
      let mut guard = controller.write().await;
      let is_subagent = guard.is_subagent();
      if !is_subagent {
        guard.stop().await;
      }
    }

    Ok(())
  }

  pub async fn plan_get(&self, project_id: SurrealId, plan_id: String) -> Result<PlanGetPayload> {
    get_plan_content(project_id, plan_id)
  }

  pub async fn close_terminal(&self, session_id: SurrealId, terminal_id: Uuid) -> Result<()> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    controller.close_terminal(terminal_id).await
  }

  pub async fn get_terminal_snapshot(&self, session_id: SurrealId, terminal_id: Uuid) -> Result<TerminalSnapshot> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;

    let controller = controller.read().await;

    match controller.get_terminal_snapshot(terminal_id).await {
      Ok(snapshot) => Ok(snapshot),
      Err(e) => {
        tracing::error!("Failed to get terminal snapshot: {e}");
        Ok(TerminalSnapshot { rows: 0, cols: 0, lines: vec![] })
      }
    }
  }

  // Command
  pub async fn send_prompt(
    &self,
    session_id: SurrealId,
    prompt: String,
    image_urls: Option<Vec<String>>,
  ) -> Result<()> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    controller.push_prompt(prompt, image_urls).await?;

    Ok(())
  }

  pub async fn delete_queued_prompt(
    &self,
    session_id: SurrealId,
    queue_item_id: String,
  ) -> Result<DeleteQueuedPromptOutcome> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    Ok(controller.delete_queued_prompt(&queue_item_id).await)
  }

  pub async fn start_plan(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();
    let session_id_value = session_id.to_string();
    let attached_plan =
      get_plan_content_by_parent_session_id(project_id.clone(), &session_id_value)?.ok_or_else(|| {
        AppCoreError::PlanNotAttachedToSession { plan_id: plan_id.clone(), session_id: session_id_value.clone() }
      })?;

    if attached_plan.id != plan_id {
      return Err(
        AppCoreError::SessionAlreadyHasDifferentPlan {
          session_id:        session_id_value,
          existing_plan_id:  attached_plan.id,
          requested_plan_id: plan_id,
        }
        .into(),
      );
    }

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&attached_plan.id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;
    frontmatter.status = PlanDocumentStatus::InProgress;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    let message = format!(
      "Execute the plan ({plan_id}) now. Do not summarize the plan. Start with task 1 immediately, make the required changes, and proceed task-by-task until complete, running verification after each task. No questions unless blocked. If the plan is already partially completed, continue from where it was left off based on the completed todos."
    );
    let reasoning_message = format!(
      r#"**Executing the plan**
I'm preparing to execute the plan ({plan_id}) now. I should not summarize the plan to the user. But start with the todos immediately, make the required changes, and proceed task-by-task until complete, running verification after each task. Let me double check on the plan before I continue. I shouldn't ask any questions unless I am blocked. If the plan is already partially completed, I should continue from where it was left off based on the completed todos."#
    );
    controller.start_system_turn(message, reasoning_message);

    Ok(())
  }

  pub async fn continue_plan(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();
    let session_id_value = session_id.to_string();
    let attached_plan =
      get_plan_content_by_parent_session_id(project_id.clone(), &session_id_value)?.ok_or_else(|| {
        AppCoreError::PlanNotAttachedToSession { plan_id: plan_id.clone(), session_id: session_id_value.clone() }
      })?;

    if attached_plan.id != plan_id {
      return Err(
        AppCoreError::SessionAlreadyHasDifferentPlan {
          session_id:        session_id_value,
          existing_plan_id:  attached_plan.id,
          requested_plan_id: plan_id,
        }
        .into(),
      );
    }

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&attached_plan.id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;
    frontmatter.status = PlanDocumentStatus::InProgress;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    let message =
      "The plan stopped being executed for some reason. Please continue from where it was left off.".to_string();
    let reasoning_message = format!(
      r#"**Continue executing plan**
The user has indicated that the plan execution was interrupted. I should to continue the plan ({plan_id}) now."#
    );
    controller.start_system_turn(message, reasoning_message);

    Ok(())
  }

  pub async fn complete_plan(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();
    let session_id_value = session_id.to_string();
    let attached_plan =
      get_plan_content_by_parent_session_id(project_id.clone(), &session_id_value)?.ok_or_else(|| {
        AppCoreError::PlanNotAttachedToSession { plan_id: plan_id.clone(), session_id: session_id_value.clone() }
      })?;

    if attached_plan.id != plan_id {
      return Err(
        AppCoreError::SessionAlreadyHasDifferentPlan {
          session_id:        session_id_value,
          existing_plan_id:  attached_plan.id,
          requested_plan_id: plan_id,
        }
        .into(),
      );
    }

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&attached_plan.id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;
    frontmatter.status = PlanDocumentStatus::Completed;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    Ok(())
  }

  pub async fn cancel_plan(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;

    let session_id_value = session_id.to_string();
    match frontmatter.parent_session_id.as_deref() {
      Some(parent_session_id) if parent_session_id == session_id_value => {}
      Some(parent_session_id) => {
        return Err(
          AppCoreError::PlanAttachedToDifferentSession {
            plan_id,
            parent_session_id: parent_session_id.to_string(),
            requested_session_id: session_id_value,
          }
          .into(),
        );
      }
      None => {
        return Err(AppCoreError::PlanNotAttachedToSession { plan_id, session_id: session_id_value }.into());
      }
    }

    frontmatter.status = PlanDocumentStatus::Archived;
    frontmatter.parent_session_id = None;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn cancel_plan_for_project(&self, project_id: SurrealId, plan_id: String) -> Result<()> {
    let _ = get_plan_content(project_id.clone(), plan_id.clone())?;

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;
    frontmatter.status = PlanDocumentStatus::Archived;
    frontmatter.parent_session_id = None;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn delete_plan(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (frontmatter, _) = parse_frontmatter(&content)?;

    let session_id_value = session_id.to_string();
    match frontmatter.parent_session_id.as_deref() {
      Some(parent_session_id) if parent_session_id == session_id_value => {}
      Some(parent_session_id) => {
        return Err(
          AppCoreError::PlanAttachedToDifferentSession {
            plan_id,
            parent_session_id: parent_session_id.to_string(),
            requested_session_id: session_id_value,
          }
          .into(),
        );
      }
      None => {
        return Err(AppCoreError::PlanNotAttachedToSession { plan_id, session_id: session_id_value }.into());
      }
    }

    std::fs::remove_file(&plan_path)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn delete_plan_for_project(&self, project_id: SurrealId, plan_id: String) -> Result<()> {
    let _ = get_plan_content(project_id.clone(), plan_id.clone())?;

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);

    std::fs::remove_file(&plan_path)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn assign_plan_to_session(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;
    let session_id_value = session_id.to_string();

    if let Some(existing_plan) = get_plan_content_by_parent_session_id(project_id.clone(), &session_id_value)?
      && existing_plan.id != plan_id
    {
      return Err(
        AppCoreError::SessionAlreadyHasDifferentPlan {
          session_id:        session_id_value,
          existing_plan_id:  existing_plan.id,
          requested_plan_id: plan_id,
        }
        .into(),
      );
    }

    match frontmatter.parent_session_id.as_deref() {
      Some(parent_session_id) if parent_session_id == session_id_value => {}
      Some(parent_session_id) => {
        return Err(
          AppCoreError::PlanAlreadyAttachedToDifferentSession {
            plan_id,
            parent_session_id: parent_session_id.to_string(),
            requested_session_id: session_id_value,
          }
          .into(),
        );
      }
      None => {
        frontmatter.parent_session_id = Some(session_id_value.clone());
      }
    }

    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn unassign_plan_from_session(&self, session_id: SurrealId, plan_id: String) -> Result<()> {
    let session = SessionRepositoryV2::get(session_id.clone()).await?;
    let project_id = session.project.clone();

    let plan_directory = resolve_plan_directory(project_id.clone())?;
    let plan_path = PathBuf::from(&plan_directory.path).join(&plan_id);
    let content = std::fs::read_to_string(&plan_path)
      .map_err(|e| ToolError::FileReadFailed { path: plan_path.display().to_string(), error: e.to_string() })?;
    let (mut frontmatter, body) = parse_frontmatter(&content)?;

    let session_id_value = session_id.to_string();
    match frontmatter.parent_session_id.as_deref() {
      Some(parent_session_id) if parent_session_id == session_id_value => {}
      Some(parent_session_id) => {
        return Err(
          AppCoreError::PlanAttachedToDifferentSession {
            plan_id,
            parent_session_id: parent_session_id.to_string(),
            requested_session_id: session_id_value,
          }
          .into(),
        );
      }
      None => {
        return Err(AppCoreError::PlanNotAttachedToSession { plan_id, session_id: session_id_value }.into());
      }
    }

    frontmatter.parent_session_id = None;
    let updated_content = render_plan_content(&frontmatter, &body)?;
    std::fs::write(&plan_path, updated_content)
      .map_err(|e| ToolError::FileWriteFailed { path: plan_path.display().to_string(), error: e.to_string() })?;

    Ok(())
  }

  pub async fn send_interrupt(&self, session_id: SurrealId) -> Result<()> {
    let controllers = self.controllers.lock().await;
    let instance = controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let mut controller = instance.write().await;

    controller.stop().await;

    Ok(())
  }

  pub async fn rewind_to(&self, session_id: SurrealId, history_id: SurrealId) -> Result<()> {
    tracing::info!("Rewind to, truncate to fast");
    MessageRepositoryV2::truncate_to(session_id.clone(), history_id).await?;
    Ok(())
  }

  pub async fn answer_question(
    &self,
    session_id: SurrealId,
    question_id: String,
    answer: String,
    answer_source: AskQuestionAnswerSource,
  ) -> Result<AskQuestionClaimResult> {
    self.answer_question_with_idempotency(session_id, question_id, answer, answer_source, None).await
  }

  pub async fn set_queue_mode(&self, session_id: SurrealId, mode: QueueMode) -> Result<()> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    controller.set_queue_mode(mode).await;
    Ok(())
  }

  pub async fn answer_question_with_idempotency(
    &self,
    session_id: SurrealId,
    question_id: String,
    answer: String,
    answer_source: AskQuestionAnswerSource,
    idempotency_key: Option<String>,
  ) -> Result<AskQuestionClaimResult> {
    let controllers = self.controllers.lock().await;
    let controller =
      controllers.get(&session_id).ok_or_else(|| AppCoreError::SessionNotFound(session_id.to_string()))?;
    let controller = controller.read().await;

    let claim_result = controller
      .answer_question_with_idempotency(question_id.clone(), answer.clone(), answer_source, idempotency_key)
      .await?;

    Ok(claim_result)
  }

  // Personality CRUD
  pub async fn personality_create(
    &self,
    name: String,
    description: String,
    system_prompt: String,
  ) -> Result<PersonalityModelDto> {
    let service = PersonalityService::new();
    let record = service.create(common::personality_service::PersonalityCreateInput {
      id: None,
      name,
      description,
      body: system_prompt,
      is_default: false,
    })?;

    Ok(Self::personality_record_to_dto(record))
  }

  pub async fn personality_update(
    &self,
    id: String,
    name: String,
    description: String,
    system_prompt: String,
  ) -> Result<PersonalityModelDto> {
    let service = PersonalityService::new();
    let record = service.update(
      &id,
      common::personality_service::PersonalityUpdateInput {
        id:          None,
        name:        Some(name),
        description: Some(description),
        body:        Some(system_prompt),
        is_default:  None,
      },
    )?;

    Ok(Self::personality_record_to_dto(record))
  }

  pub async fn personality_delete(&self, id: String) -> Result<()> {
    let service = PersonalityService::new();
    service.delete(&id)?;
    Ok(())
  }

  pub async fn personality_list(&self) -> Result<Vec<PersonalityModelDto>> {
    let service = PersonalityService::new();
    let records = service.list()?;
    Ok(records.into_iter().map(Self::personality_record_to_dto).collect())
  }

  fn personality_record_to_dto(record: PersonalityRecord) -> PersonalityModelDto {
    let now = Utc::now();
    PersonalityModelDto {
      id:              record.frontmatter.id,
      name:            record.frontmatter.name,
      description:     record.frontmatter.description,
      system_prompt:   record.body,
      is_default:      record.frontmatter.is_default,
      is_user_defined: record.source == common::personality_files::PersonalitySource::User,
      created_at:      now,
      updated_at:      now,
    }
  }
}
