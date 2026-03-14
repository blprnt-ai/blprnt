use std::collections::HashMap;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use anyhow::Result;
use common::blprnt::Blprnt;
use common::blprnt_dispatch::BlprntDispatch;
use common::blprnt_dispatch::SessionEvent;
use common::errors::AppCoreError;
use common::session_dispatch::prelude::SessionDispatchEvent;
use common::session_dispatch::prelude::SlackInput;
use common::shared::prelude::SurrealId;
use common::slack::SendMessage;
use common::slack::SlackChannel;
use common::tools::question::AskQuestionAnswerSource;
use common::tools::question::AskQuestionClaimStatus;
use persistence::prelude::MessageRepositoryV2;
use persistence::prelude::SessionRepositoryV2;
use serde_json::Value;
use surrealdb::types::Uuid;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use vault::Vault;

use crate::cmd::slack_access_token_key;
use crate::cmd::slack_authed_user_id_key;
use crate::engine_manager::BLPRNT_STORE;
use crate::engine_manager::TUNNEL_STORE;

#[derive(Debug)]
pub struct SlackMessage {
  pub title:      String,
  pub body:       String,
  pub cta_url:    Option<String>,
  pub session_id: Option<String>,
}

#[derive(Debug)]
pub struct SlackInteractiveMessage;

#[derive(Debug)]
pub struct SlackNotifyResponse {
  pub ok:    bool,
  pub error: Option<String>,
}

fn format_session_root_post(session_name: &str) -> String {
  session_name.to_string()
}

fn format_final_assistant_reply(content: &str) -> String {
  content.to_string()
}

fn format_subagent_completed_line(subagent_name: &str) -> String {
  format!("Subagent completed: {subagent_name}")
}

fn format_subagent_failed_line(subagent_name: &str) -> String {
  format!("Subagent failed: {subagent_name}")
}

fn format_ask_question_prompt(details: &str, choices: &[String]) -> String {
  let choice_lines = choices.iter().map(|choice| format!("- {choice}")).collect::<Vec<_>>().join("\n");
  format!("**Question**\n\n{details}\n\n{choice_lines}")
}

async fn fallback_session_root_post(session_id: &str) -> String {
  let Ok(surreal_session_id) = SurrealId::try_from(session_id) else {
    return format_session_root_post(session_id);
  };

  let Ok(session_record) = SessionRepositoryV2::get(surreal_session_id).await else {
    return format_session_root_post(session_id);
  };

  let session_name = session_record.name.trim();
  if session_name.is_empty() { format_session_root_post(session_id) } else { format_session_root_post(session_name) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SlackAskQuestionState {
  QueuedUnsent,
  AwaitingAnswer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SlackAskQuestion {
  details: String,
  choices: Vec<String>,
  state:   SlackAskQuestionState,
}

impl SlackAskQuestion {
  fn new(details: String, choices: Vec<String>) -> Self {
    Self { details, choices, state: SlackAskQuestionState::QueuedUnsent }
  }
}

type SlackAskQuestionQueue = VecDeque<SlackAskQuestion>;

pub struct SlackManager {
  adapter:                RwLock<Option<Arc<SlackChannel>>>,
  app_focused:            AtomicBool,
  ask_question_send_lock: Mutex<()>,
  session_init_locks:     Mutex<HashMap<String, Arc<Mutex<()>>>>,
  subagent_name_cache:    RwLock<HashMap<String, String>>,
  ask_question_queues:    RwLock<HashMap<String, SlackAskQuestionQueue>>,
  session_thread_ts_map:  RwLock<HashMap<String, String>>,
  thread_ts_session_map:  RwLock<HashMap<String, String>>,
}

impl SlackManager {
  pub fn new() -> Self {
    Self {
      adapter:                RwLock::new(None),
      app_focused:            AtomicBool::new(true),
      ask_question_send_lock: Mutex::new(()),
      session_init_locks:     Mutex::new(HashMap::new()),
      subagent_name_cache:    RwLock::new(HashMap::new()),
      ask_question_queues:    RwLock::new(HashMap::new()),
      session_thread_ts_map:  RwLock::new(HashMap::new()),
      thread_ts_session_map:  RwLock::new(HashMap::new()),
    }
  }

  pub fn set_app_focused(&self, focused: bool) -> bool {
    let was_focused = self.app_focused.swap(focused, Ordering::Relaxed);
    was_focused && !focused
  }

  pub fn is_app_focused(&self) -> bool {
    self.app_focused.load(Ordering::Relaxed)
  }

  pub async fn send_final_assistant_response(
    &self,
    session_id: String,
    session_name: String,
    content: String,
  ) -> Result<()> {
    if self.is_app_focused() {
      return Ok(());
    }

    let adapter = self.adapter.read().await.clone();
    let Some(adapter) = adapter else {
      return Ok(());
    };

    let thread_ts = self.ensure_session_thread_ts(&adapter, &session_id, &session_name).await?;
    self.send_plain_text(&adapter, Self::format_final_assistant_reply(&content), Some(thread_ts)).await?;

    Ok(())
  }

  pub async fn cache_subagent_name(&self, session_id: String, subagent_name: String) {
    self.subagent_name_cache.write().await.insert(session_id, subagent_name);
  }

  pub async fn send_subagent_completed(&self, session_id: String, subagent_name: String) -> Result<()> {
    self.send_subagent_status(session_id, Self::format_subagent_completed_reply(&subagent_name)).await
  }

  pub async fn send_subagent_failed(&self, session_id: String, subagent_name: String) -> Result<()> {
    self.send_subagent_status(session_id, Self::format_subagent_failed_reply(&subagent_name)).await
  }

  pub fn format_session_root_post(session_name: &str) -> String {
    format_session_root_post(session_name)
  }

  pub fn format_final_assistant_reply(content: &str) -> String {
    format_final_assistant_reply(content)
  }

  pub fn format_subagent_completed_reply(subagent_name: &str) -> String {
    format_subagent_completed_line(subagent_name)
  }

  pub fn format_subagent_failed_reply(subagent_name: &str) -> String {
    format_subagent_failed_line(subagent_name)
  }

  pub fn format_ask_question_prompt(details: &str, choices: &[String]) -> String {
    format_ask_question_prompt(details, choices)
  }

  async fn take_subagent_name(&self, session_id: &str) -> Option<String> {
    self.subagent_name_cache.write().await.remove(session_id)
  }

  async fn send_subagent_status(&self, session_id: String, content: String) -> Result<()> {
    if self.is_app_focused() {
      return Ok(());
    }

    let adapter = self.adapter.read().await.clone();
    let Some(adapter) = adapter else {
      return Ok(());
    };

    let thread_ts =
      self.ensure_session_thread_ts(&adapter, &session_id, &fallback_session_root_post(&session_id).await).await?;

    self.send_plain_text(&adapter, content, Some(thread_ts)).await?;

    Ok(())
  }

  pub async fn send_cached_subagent_completion(&self, session_id: String) -> Result<()> {
    let Some(subagent_name) = self.take_subagent_name(&session_id).await else {
      return Ok(());
    };

    self.send_subagent_completed(session_id, subagent_name).await
  }

  pub async fn send_cached_subagent_failure(&self, session_id: String) -> Result<()> {
    let Some(subagent_name) = self.take_subagent_name(&session_id).await else {
      return Ok(());
    };

    self.send_subagent_failed(session_id, subagent_name).await
  }

  pub async fn init(self: &Arc<Self>) {
    if !self.enabled() || !self.connected() {
      return;
    }

    if let Err(error) = self.restore_runtime().await {
      tracing::warn!("Failed to restore Slack runtime: {error}");
      let _ = self.set_status(false, Some(error.to_string()));
    }
  }

  pub fn oauth_state(&self) -> Option<String> {
    let store = Blprnt::handle().clone().store(BLPRNT_STORE).ok()?;
    store.get("slack_oauth_state").and_then(|value| value.as_str().map(|s| s.to_string()))
  }

  pub fn connected(&self) -> bool {
    let store = Blprnt::handle().clone().store(BLPRNT_STORE).ok();
    store.and_then(|store| store.get("slack_connected")).and_then(|value| value.as_bool()).unwrap_or(false)
  }

  pub fn last_error(&self) -> Option<String> {
    let store = Blprnt::handle().clone().store(BLPRNT_STORE).ok()?;
    store.get("slack_last_error").and_then(|value| value.as_str().map(|s| s.to_string()))
  }

  pub fn enabled(&self) -> bool {
    let store = Blprnt::handle().clone().store(BLPRNT_STORE).ok();
    store.and_then(|store| store.get("slack_enabled")).and_then(|value| value.as_bool()).unwrap_or(false)
  }

  pub fn set_oauth_state(&self, state: Option<String>) -> Result<()> {
    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;
    match state {
      Some(state) => store.set("slack_oauth_state", state),
      None => store.set("slack_oauth_state", Value::Null),
    }
    Ok(())
  }

  pub fn set_status(&self, connected: bool, last_error: Option<String>) -> Result<()> {
    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;
    store.set("slack_connected", connected);
    match last_error {
      Some(error) => store.set("slack_last_error", error),
      None => store.set("slack_last_error", Value::Null),
    }
    Ok(())
  }

  pub fn set_enabled(&self, enabled: bool) -> Result<()> {
    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;
    store.set("slack_enabled", enabled);
    Ok(())
  }

  pub fn persist_oauth_success(&self, payload: &Value) -> Result<()> {
    let access_token = payload.get("access_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let authed_user_id =
      payload.get("authed_user").and_then(|v| v.get("id")).and_then(|v| v.as_str()).map(|s| s.to_string());

    let team_id = payload
      .get("team")
      .and_then(|v| v.get("id"))
      .and_then(|v| v.as_str())
      .or_else(|| payload.get("team_id").and_then(|v| v.as_str()))
      .map(|s| s.to_string());

    let bot_user_id = payload.get("bot_user_id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let scope = payload.get("scope").and_then(|v| v.as_str()).map(|s| s.to_string());

    if let Some(token) = access_token {
      let tunnel_id = Self::get_tunnel_uuid();
      let key = slack_access_token_key(tunnel_id);
      tokio::spawn(async move {
        if let Err(err) = vault::set_stronghold_secret(Vault::Key, key, &token).await {
          tracing::warn!("Failed to store Slack access token: {err}");
        }
      });
    }

    if let Some(authed_user_id) = authed_user_id {
      let tunnel_id = Self::get_tunnel_uuid();
      let key = slack_authed_user_id_key(tunnel_id);
      tokio::spawn(async move {
        if let Err(err) = vault::set_stronghold_secret(Vault::Key, key, &authed_user_id).await {
          tracing::warn!("Failed to store Slack authed user id: {err}");
        }
      });
    }

    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;
    store.set("slack_connected", true);
    store.set("slack_last_error", Value::Null);
    store.set("slack_team_id", team_id.unwrap_or_default());
    store.set("slack_bot_user_id", bot_user_id.unwrap_or_default());
    store.set("slack_scope", scope.unwrap_or_default());
    store.set("slack_enabled", true);
    store.set("slack_oauth_state", Value::Null);
    Ok(())
  }

  async fn restore_runtime(self: &Arc<Self>) -> Result<()> {
    let tunnel_id = Self::get_tunnel_uuid();
    let bot_token =
      vault::get_stronghold_secret(Vault::Key, slack_access_token_key(tunnel_id)).await.unwrap_or_default();
    let authed_user_id =
      vault::get_stronghold_secret(Vault::Key, slack_authed_user_id_key(tunnel_id)).await.unwrap_or_default();

    anyhow::ensure!(!bot_token.trim().is_empty(), "missing Slack access token");
    anyhow::ensure!(!authed_user_id.trim().is_empty(), "missing Slack authed user id");

    let dm_channel_id = SlackChannel::open_dm_channel_id(&bot_token, &authed_user_id).await?;
    let adapter = Arc::new(SlackChannel::new(bot_token, dm_channel_id, authed_user_id));

    anyhow::ensure!(adapter.health_check().await, "Slack health check failed");

    self.restore_thread_maps().await?;

    {
      let mut guard = self.adapter.write().await;
      *guard = Some(adapter.clone());
    }

    self.set_status(true, None)?;

    Ok(())
  }

  async fn restore_thread_maps(&self) -> Result<()> {
    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;

    let session_thread_ts_map = store
      .get("slack_session_thread_ts_map")
      .and_then(|value| serde_json::from_value::<HashMap<String, String>>(value).ok())
      .unwrap_or_default();
    let thread_ts_session_map = store
      .get("slack_thread_ts_session_map")
      .and_then(|value| serde_json::from_value::<HashMap<String, String>>(value).ok())
      .unwrap_or_default();
    *self.session_thread_ts_map.write().await = session_thread_ts_map;
    *self.thread_ts_session_map.write().await = thread_ts_session_map;

    Ok(())
  }

  async fn persist_thread_maps(&self) -> Result<()> {
    let store =
      Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|e| AppCoreError::FailedToOpenStore(e.to_string()))?;

    let session_thread_ts_map = self.session_thread_ts_map.read().await.clone();
    let thread_ts_session_map = self.thread_ts_session_map.read().await.clone();
    store.set("slack_session_thread_ts_map", serde_json::to_value(session_thread_ts_map)?);
    store.set("slack_thread_ts_session_map", serde_json::to_value(thread_ts_session_map)?);

    Ok(())
  }

  async fn thread_ts_for_session(&self, session_id: &str) -> Option<String> {
    self.session_thread_ts_map.read().await.get(session_id).cloned()
  }

  pub async fn enqueue_ask_question_for_session(
    &self,
    session_id: &str,
    details: String,
    choices: Vec<String>,
  ) -> Option<usize> {
    let thread_ts = self.thread_ts_for_session(session_id).await?;
    let queue_len = self.enqueue_ask_question(thread_ts.clone(), details, choices).await;
    self.try_deliver_ask_question_head(&thread_ts).await;
    Some(queue_len)
  }

  async fn session_id_for_thread_ts(&self, thread_ts: &str) -> Option<String> {
    self.thread_ts_session_map.read().await.get(thread_ts).cloned()
  }

  async fn enqueue_ask_question(&self, thread_ts: String, details: String, choices: Vec<String>) -> usize {
    let mut queues = self.ask_question_queues.write().await;
    let queue = queues.entry(thread_ts).or_default();
    queue.push_back(SlackAskQuestion::new(details, choices));
    queue.len()
  }

  async fn ask_question_queue_head(&self, thread_ts: &str) -> Option<SlackAskQuestion> {
    self.ask_question_queues.read().await.get(thread_ts).and_then(|queue| queue.front().cloned())
  }

  async fn mark_ask_question_head_awaiting_answer(&self, thread_ts: &str) -> bool {
    let mut queues = self.ask_question_queues.write().await;
    let Some(question) = queues.get_mut(thread_ts).and_then(VecDeque::front_mut) else {
      return false;
    };
    question.state = SlackAskQuestionState::AwaitingAnswer;
    true
  }

  async fn pop_completed_ask_question_head(&self, thread_ts: &str) -> Option<SlackAskQuestion> {
    let mut queues = self.ask_question_queues.write().await;
    let queue = queues.get_mut(thread_ts)?;
    let question = queue.pop_front();
    if queue.is_empty() {
      queues.remove(thread_ts);
    }
    question
  }

  async fn has_awaiting_ask_question(&self, thread_ts: &str) -> bool {
    matches!(
      self.ask_question_queue_head(thread_ts).await.as_ref().map(|question| &question.state),
      Some(SlackAskQuestionState::AwaitingAnswer)
    )
  }

  pub async fn try_deliver_all_queued_ask_questions(&self) {
    let thread_tss = self.ask_question_queues.read().await.keys().cloned().collect::<Vec<_>>();
    for thread_ts in thread_tss {
      self.try_deliver_ask_question_head(&thread_ts).await;
    }
  }

  async fn try_deliver_ask_question_head(&self, thread_ts: &str) {
    if self.is_app_focused() {
      return;
    }

    let adapter = self.adapter.read().await.clone();
    let Some(adapter) = adapter else {
      return;
    };

    let _send_guard = self.ask_question_send_lock.lock().await;

    let Some(question) = self.ask_question_queue_head(thread_ts).await else {
      return;
    };

    if question.state != SlackAskQuestionState::QueuedUnsent {
      return;
    }

    let content = Self::format_ask_question_prompt(&question.details, &question.choices);

    if self.send_plain_text(&adapter, content, Some(thread_ts.to_string())).await.is_err() {
      return;
    }

    let _ = self.mark_ask_question_head_awaiting_answer(thread_ts).await;
  }

  fn message_ts_from_id(message_id: &str) -> Option<String> {
    message_id.rsplit('_').next().map(str::to_string)
  }

  fn is_main_dm_message(message: &common::slack::ChannelMessage) -> bool {
    message.thread_ts.is_none()
  }

  async fn answer_awaiting_ask_question(
    &self,
    session_id: SurrealId,
    answer: String,
  ) -> Result<common::tools::question::AskQuestionClaimResult> {
    let pending_requests = MessageRepositoryV2::get_pending_user_interaction_requests(session_id.clone()).await?;
    let Some((_, question_id, ..)) = pending_requests.into_iter().next() else {
      anyhow::bail!("missing pending ask_question request")
    };

    let app = Blprnt::handle();
    let manager = app.state::<Arc<super::EngineManager>>();
    manager.answer_question(session_id, question_id, answer, AskQuestionAnswerSource::Desktop).await
  }

  async fn route_inbound_message(&self, message: common::slack::ChannelMessage) {
    let Some(thread_ts) = message.thread_ts.clone() else {
      tracing::debug!(sender = %message.sender, channel = %message.reply_target, "Ignoring Slack inbound message without thread_ts");
      return;
    };

    if Self::is_main_dm_message(&message) {
      tracing::debug!(sender = %message.sender, channel = %message.reply_target, "Ignoring Slack inbound main DM message");
      return;
    }

    let Some(session_id) = self.session_id_for_thread_ts(&thread_ts).await else {
      tracing::debug!(thread_ts = %thread_ts, sender = %message.sender, channel = %message.reply_target, "Ignoring Slack inbound message without persisted session mapping");
      return;
    };

    let surreal_session_id = match SurrealId::try_from(session_id.as_str()) {
      Ok(session_id) => session_id,
      Err(error) => {
        tracing::warn!(session_id = %session_id, thread_ts = %thread_ts, "Ignoring Slack inbound message with invalid session id: {error}");
        return;
      }
    };

    if self.has_awaiting_ask_question(&thread_ts).await {
      match self.answer_awaiting_ask_question(surreal_session_id.clone(), message.content).await {
        Ok(claim_result) => {
          if matches!(
            claim_result.outcome,
            AskQuestionClaimStatus::Accepted | AskQuestionClaimStatus::RejectedAlreadyAnswered
          ) {
            let _ = self.pop_completed_ask_question_head(&thread_ts).await;
            self.try_deliver_ask_question_head(&thread_ts).await;
          }
        }
        Err(error) => {
          tracing::warn!(session_id = %session_id, thread_ts = %thread_ts, "Failed to capture Slack ask_question answer: {error}");
          if let Some(adapter) = self.adapter.read().await.clone() {
            let _ = self
              .send_plain_text(
                &adapter,
                "I couldn’t process that answer, so your question is still waiting. Please reply again.".to_string(),
                Some(thread_ts.clone()),
              )
              .await;
          }
        }
      }
      return;
    }

    let event = SessionEvent {
      session_id: surreal_session_id,
      parent_id:  None,
      event_data: SessionDispatchEvent::from(SlackInput {
        session_id,
        text: message.content,
        slack_user_id: message.sender,
        slack_channel_id: message.reply_target,
        thread_ts: Some(thread_ts),
        message_ts: Self::message_ts_from_id(&message.id),
      }),
    };

    if let Err(error) = BlprntDispatch::send(event).await {
      tracing::warn!("Failed to dispatch routed Slack inbound message: {error}");
    }
  }

  pub async fn handle_inbound_message(&self, message: common::slack::ChannelMessage) {
    self.route_inbound_message(message).await;
  }

  async fn persist_session_thread_mapping(&self, session_id: String, thread_ts: String) -> Result<()> {
    {
      let mut session_map = self.session_thread_ts_map.write().await;
      let mut thread_map = self.thread_ts_session_map.write().await;
      session_map.insert(session_id.clone(), thread_ts.clone());
      thread_map.insert(thread_ts, session_id);
    }
    self.persist_thread_maps().await
  }

  async fn ensure_session_thread_ts(
    &self,
    adapter: &SlackChannel,
    session_id: &str,
    session_name: &str,
  ) -> Result<String> {
    if let Some(thread_ts) = self.thread_ts_for_session(session_id).await {
      return Ok(thread_ts);
    }

    let init_lock = {
      let mut locks = self.session_init_locks.lock().await;
      locks.entry(session_id.to_string()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
    };
    let _guard = init_lock.lock().await;

    if let Some(thread_ts) = self.thread_ts_for_session(session_id).await {
      return Ok(thread_ts);
    }

    let thread_ts = self.send_plain_text(adapter, Self::format_session_root_post(session_name), None).await?;
    self.persist_session_thread_mapping(session_id.to_string(), thread_ts.clone()).await?;

    Ok(thread_ts)
  }

  async fn send_plain_text(
    &self,
    adapter: &SlackChannel,
    content: String,
    thread_ts: Option<String>,
  ) -> Result<String> {
    let send_message = SendMessage::new(content, adapter.dm_channel_id().to_string()).in_thread(thread_ts);
    adapter.send(&send_message).await
  }

  pub async fn send_dm(&self, message: SlackMessage) -> Result<SlackNotifyResponse> {
    let adapter = self.adapter.read().await.clone();
    let Some(adapter) = adapter else {
      return Ok(SlackNotifyResponse { ok: false, error: Some("slack_messaging_disabled".into()) });
    };

    let session_id =
      message.session_id.as_deref().map(str::trim).filter(|value: &&str| !value.is_empty()).map(str::to_string);
    let thread_ts = match session_id.as_deref() {
      Some(session_id) => {
        let session_name = message.title.trim();
        let session_root = if session_name.is_empty() {
          fallback_session_root_post(session_id).await
        } else {
          format_session_root_post(session_name)
        };
        match self.ensure_session_thread_ts(&adapter, session_id, &session_root).await {
          Ok(thread_ts) => Some(thread_ts),
          Err(error) => return Ok(SlackNotifyResponse { ok: false, error: Some(error.to_string()) }),
        }
      }
      None => None,
    };

    let content = Self::format_final_assistant_reply(&message.body);

    match self.send_plain_text(&adapter, content, thread_ts).await {
      Ok(_) => (),
      Err(error) => return Ok(SlackNotifyResponse { ok: false, error: Some(error.to_string()) }),
    }

    Ok(SlackNotifyResponse { ok: true, error: None })
  }

  pub async fn send_dm_interactive(&self, _message: SlackInteractiveMessage) -> Result<SlackNotifyResponse> {
    Ok(SlackNotifyResponse { ok: false, error: Some("slack_messaging_disabled".into()) })
  }

  pub async fn try_bind_session_registration_on_tunnel_connect(&self) {}

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
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use common::session_dispatch::prelude::SessionDispatchEvent;
  use common::session_dispatch::prelude::SlackEvent;

  use super::*;

  fn test_channel_message(thread_ts: &str, message_ts: &str, content: &str) -> common::slack::ChannelMessage {
    common::slack::ChannelMessage {
      id:           format!("slack_D123_{message_ts}"),
      sender:       "U123".to_string(),
      reply_target: "D123".to_string(),
      content:      content.to_string(),
      channel:      "slack".to_string(),
      timestamp:    1712345678,
      thread_ts:    Some(thread_ts.to_string()),
    }
  }

  #[tokio::test]
  async fn routes_normalized_inbound_thread_reply_to_session_dispatch() {
    let manager = SlackManager::new();
    let session_id = SurrealId::new("sessions".to_string()).to_string();
    let thread_ts = "1712000.000".to_string();
    let message_ts = "1712345.678";

    manager.session_thread_ts_map.write().await.insert(session_id.clone(), thread_ts.clone());
    manager.thread_ts_session_map.write().await.insert(thread_ts.clone(), session_id.clone());

    let dispatch = BlprntDispatch::get_or_init();
    let mut rx = dispatch.tx.subscribe();

    manager.handle_inbound_message(test_channel_message(&thread_ts, message_ts, "thread reply")).await;

    let event = tokio::time::timeout(Duration::from_secs(1), rx.recv())
      .await
      .expect("expected dispatch event")
      .expect("dispatch recv should succeed");

    assert_eq!(event.session_id.to_string(), session_id);
    match event.event_data {
      SessionDispatchEvent::Slack(SlackEvent::Input(input)) => {
        assert_eq!(input.session_id, session_id);
        assert_eq!(input.text, "thread reply");
        assert_eq!(input.slack_user_id, "U123");
        assert_eq!(input.slack_channel_id, "D123");
        assert_eq!(input.thread_ts.as_deref(), Some(thread_ts.as_str()));
        assert_eq!(input.message_ts.as_deref(), Some(message_ts));
      }
      other => panic!("unexpected event: {other:?}"),
    }
  }

  #[tokio::test]
  async fn intercepts_normalized_inbound_reply_for_awaiting_ask_question() {
    let manager = Arc::new(SlackManager::new());
    let session_id = SurrealId::new("sessions".to_string()).to_string();
    let thread_ts = "1712000.000".to_string();

    manager.session_thread_ts_map.write().await.insert(session_id.clone(), thread_ts.clone());
    manager.thread_ts_session_map.write().await.insert(thread_ts.clone(), session_id.clone());
    manager.enqueue_ask_question(thread_ts.clone(), "question".to_string(), vec!["one".to_string()]).await;
    assert!(manager.mark_ask_question_head_awaiting_answer(&thread_ts).await);

    let dispatch = BlprntDispatch::get_or_init();
    let mut rx = dispatch.tx.subscribe();

    let manager_for_task = manager.clone();
    let thread_ts_for_task = thread_ts.clone();
    let join = tokio::spawn(async move {
      manager_for_task
        .handle_inbound_message(test_channel_message(&thread_ts_for_task, "1712345.679", "chosen answer"))
        .await;
    })
    .await;

    assert!(join.is_ok());
    loop {
      match rx.try_recv() {
        Ok(event) => assert_ne!(event.session_id.to_string(), session_id),
        Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
        Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
        Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
      }
    }
    assert!(manager.has_awaiting_ask_question(&thread_ts).await);
  }
}
