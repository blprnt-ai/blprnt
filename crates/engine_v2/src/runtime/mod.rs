pub mod context;
pub mod provider_model_heuristics;
pub mod reasoning_heuristics;
pub mod same_provider_model_selector;
pub mod skill_matcher_heuristics;

mod ask_question_handler;
mod basic_tool_handler;
mod provider_event_handler;
mod subagent_handler;
mod terminal_handler;
mod tool_call_handler;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use common::agent::ToolId;
use common::models::ReasoningEffort;
use common::provider_dispatch::ProviderDispatch;
use common::session_dispatch::SessionDispatch;
use common::session_dispatch::prelude::*;
use common::shared::prelude::*;
use common::skills_utils::SkillsUtils;
use common::tools::TerminalSnapshot;
use common::tools::ToolResult;
use common::tools::ToolUseResponse;
use common::tools::ToolUseResponseError;
use common::tools::ToolUseResponseSuccess;
use common::tools::question::AskQuestionAnswerSource;
use common::tools::question::AskQuestionClaimResult;
use common::tools::question::AskQuestionPayload;
use persistence::prelude::MessageRepositoryV2;
use session::Session;
use surrealdb::types::Uuid;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::hooks::traits::HookKind;
use crate::prelude::ControllerConfig;
use crate::queue::Queue;
use crate::runtime::context::RuntimeContext;
use crate::runtime::provider_event_handler::ProviderEventHandler;
use crate::terminal::TerminalManager;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ControlFlow {
  Continue,
  EndStep,
  EndTurn,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum RuntimeState {
  Idle,
  Running,
  Paused(PauseReason),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum PauseReason {
  UserInterrupted,
}

#[derive(Clone, Debug)]
pub enum UserInteraction {
  QuestionAnswer(String),
}

pub type SharedTerminal = Arc<Mutex<TerminalManager>>;
pub type TerminalManagers = Arc<Mutex<HashMap<Uuid, SharedTerminal>>>;

pub struct Runtime {
  session_dispatch:          Arc<SessionDispatch>,
  queue:                     Arc<Queue>,
  config:                    ControllerConfig,
  state:                     RwLock<RuntimeState>,
  user_interaction_requests: Arc<Mutex<HashMap<String, oneshot::Sender<UserInteraction>>>>,
  terminal_managers:         TerminalManagers,
  answered_question_ids:     Arc<Mutex<HashSet<String>>>,
  slack_claim_idempotency:   Arc<Mutex<HashMap<String, SlackClaimIdempotencyEntry>>>,
}

#[derive(Clone, Debug)]
struct SlackClaimIdempotencyEntry {
  question_id: String,
  result:      AskQuestionClaimResult,
}

impl Runtime {
  pub fn new(session_dispatch: Arc<SessionDispatch>, queue: Arc<Queue>, config: ControllerConfig) -> Arc<Self> {
    let user_interaction_requests = Arc::new(Mutex::new(HashMap::new()));
    let answered_question_ids = Arc::new(Mutex::new(HashSet::new()));
    let slack_claim_idempotency = Arc::new(Mutex::new(HashMap::<String, SlackClaimIdempotencyEntry>::new()));
    let terminal_managers = Arc::new(Mutex::new(HashMap::new()));

    Arc::new(Self {
      session_dispatch: session_dispatch,
      queue: queue,
      config: config,
      state: RwLock::new(RuntimeState::Idle),
      user_interaction_requests: user_interaction_requests,
      terminal_managers,
      answered_question_ids: answered_question_ids,
      slack_claim_idempotency: slack_claim_idempotency,
    })
  }

  // State
  pub async fn state(self: Arc<Self>) -> RuntimeState {
    let guard = self.state.read().await;
    guard.clone()
  }

  async fn set_state(self: Arc<Self>, state: RuntimeState) {
    let mut guard = self.state.write().await;
    *guard = state;
  }

  fn normalize_slack_idempotency_key(
    answer_source: AskQuestionAnswerSource,
    idempotency_key: Option<&str>,
  ) -> Option<String> {
    let key = idempotency_key?.trim();
    if key.is_empty() {
      return None;
    }

    match answer_source {
      AskQuestionAnswerSource::SlackButton => Some(format!("slack_button:{key}")),
      AskQuestionAnswerSource::SlackModal => Some(format!("slack_modal:{key}")),
      AskQuestionAnswerSource::Desktop => None,
    }
  }

  fn claim_ask_question_interaction_once(
    pending: &mut HashMap<String, oneshot::Sender<UserInteraction>>,
    answered: &mut HashSet<String>,
    tool_use_id: String,
    request: UserInteraction,
    answer_source: AskQuestionAnswerSource,
  ) -> AskQuestionClaimResult {
    // No-expiration policy: claim outcomes are computed only from canonical backend state
    // (`pending` / `answered`) and never from wall-clock age or TTL.
    if let Some(tx) = pending.remove(&tool_use_id) {
      if tx.send(request).is_ok() {
        answered.insert(tool_use_id.clone());
        return AskQuestionClaimResult::accepted(tool_use_id, answer_source);
      }

      return AskQuestionClaimResult::invalid(tool_use_id, answer_source);
    }

    if answered.contains(&tool_use_id) {
      return AskQuestionClaimResult::rejected_already_answered(tool_use_id, answer_source);
    }

    AskQuestionClaimResult::invalid(tool_use_id, answer_source)
  }

  fn claim_ask_question_interaction(
    pending: &mut HashMap<String, oneshot::Sender<UserInteraction>>,
    answered: &mut HashSet<String>,
    slack_idempotency: &mut HashMap<String, SlackClaimIdempotencyEntry>,
    tool_use_id: String,
    request: UserInteraction,
    answer_source: AskQuestionAnswerSource,
    idempotency_key: Option<String>,
  ) -> AskQuestionClaimResult {
    let normalized_key = Self::normalize_slack_idempotency_key(answer_source, idempotency_key.as_deref());
    if let Some(key) = normalized_key {
      if let Some(previous) = slack_idempotency.get(&key) {
        if previous.question_id == tool_use_id {
          return previous.result.clone();
        }

        return AskQuestionClaimResult::invalid(tool_use_id, answer_source);
      }

      let question_id = tool_use_id.clone();
      let result = Self::claim_ask_question_interaction_once(pending, answered, tool_use_id, request, answer_source);
      slack_idempotency.insert(key, SlackClaimIdempotencyEntry { question_id, result: result.clone() });
      return result;
    }

    Self::claim_ask_question_interaction_once(pending, answered, tool_use_id, request, answer_source)
  }

  pub async fn close_terminal(self: Arc<Self>, terminal_id: Uuid) -> Result<()> {
    let timeout = tokio::time::timeout(Duration::from_secs(10), self.terminal_managers.lock()).await;
    match timeout {
      Ok(mut terminal_managers) => {
        if let Some(terminal) = terminal_managers.remove(&terminal_id) {
          let terminal = terminal.lock().await;
          terminal.close()?;

          let _ = Session::append_signal_to_assistant(
            &self.config.session_id,
            SignalPayload::warning(format!("<system-message>Terminal {terminal_id} closed by user</system-message>"))
              .into(),
          )
          .await?;
        }

        Ok(())
      }
      Err(_) => Ok(()),
    }
  }

  pub async fn get_terminal_snapshot(self: Arc<Self>, terminal_id: Uuid) -> Result<TerminalSnapshot> {
    let terminal_managers = self.terminal_managers.lock().await;

    if let Some(terminal) = terminal_managers.get(&terminal_id) {
      let terminal = terminal.lock().await;
      Ok(terminal.snapshot_text())
    } else {
      Err(anyhow::anyhow!("Terminal not found"))
    }
  }

  /// Ask-question claim contract (T2):
  ///
  /// Canonical state machine:
  /// - `UNANSWERED_PENDING`: `tool_use_id` exists in `user_interaction_requests` and not in `answered_question_ids`.
  /// - `ANSWER_ACCEPTED` (terminal): `tool_use_id` exists in `answered_question_ids` and not in `user_interaction_requests`.
  /// - `UNKNOWN_OR_CLOSED`: `tool_use_id` absent from both sets, or pending sender exists but receiver is closed.
  ///
  /// Transition and outcome rules:
  /// - `UNANSWERED_PENDING` + successful send => `accepted`, transition to `ANSWER_ACCEPTED`.
  /// - `ANSWER_ACCEPTED` + any later attempt (including retry/replay) => `rejected_already_answered`.
  /// - unknown `tool_use_id` => `invalid`.
  /// - pending sender with closed receiver => `invalid` and must not transition to `ANSWER_ACCEPTED`.
  pub async fn handle_user_interaction_request(
    self: Arc<Self>,
    tool_use_id: String,
    request: UserInteraction,
    answer_source: AskQuestionAnswerSource,
  ) -> Result<AskQuestionClaimResult> {
    let mut pending = self.user_interaction_requests.lock().await;
    let mut answered = self.answered_question_ids.lock().await;
    let mut slack_idempotency = self.slack_claim_idempotency.lock().await;

    Ok(Self::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut slack_idempotency,
      tool_use_id,
      request,
      answer_source,
      None,
    ))
  }

  pub async fn handle_user_interaction_request_with_idempotency(
    self: Arc<Self>,
    tool_use_id: String,
    request: UserInteraction,
    answer_source: AskQuestionAnswerSource,
    idempotency_key: Option<String>,
  ) -> Result<AskQuestionClaimResult> {
    if idempotency_key.is_none() {
      return self.handle_user_interaction_request(tool_use_id, request, answer_source).await;
    }

    let mut pending = self.user_interaction_requests.lock().await;
    let mut answered = self.answered_question_ids.lock().await;
    let mut slack_idempotency = self.slack_claim_idempotency.lock().await;

    Ok(Self::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut slack_idempotency,
      tool_use_id,
      request,
      answer_source,
      idempotency_key,
    ))
  }

  // Runtime
  #[allow(clippy::manual_async_fn)]
  pub fn run(self: Arc<Self>, cancel_token: CancellationToken) -> impl Future<Output = Result<()>> + Send + 'static {
    async move {
      let state = { self.clone().state().await };
      if matches!(state, RuntimeState::Running) {
        return Ok(());
      }

      tokio::spawn({
        let self_clone = self.clone();

        async move {
          self_clone.inner_run(cancel_token).await;
        }
      });

      Ok(())
    }
  }

  async fn inner_run(self: Arc<Self>, cancel_token: CancellationToken) {
    {
      self.clone().set_state(RuntimeState::Running).await;
    }

    let queue = self.clone().queue.clone();

    let _ = self.clone().maybe_handle_user_interaction_requests(cancel_token.child_token()).await;

    'turn: loop {
      match queue.clone().recv().await {
        Some(item) => {
          tracing::info!("Next turn: starting turn: {:?}", item.queue_item_id());

          _ = match self.clone().next_turn(item, cancel_token.child_token()).await {
            Ok(state) => match state {
              RuntimeState::Running => continue,
              RuntimeState::Idle => break 'turn,
              RuntimeState::Paused(PauseReason::UserInterrupted) => {
                let signal = SignalPayload::warning("User interrupted".into());
                let id = Session::append_signal_to_user(&self.config.session_id, signal.clone().into())
                  .await
                  .unwrap_or_default();

                let _ = self.session_dispatch.send(signal.with_id(id).into()).await;
                let _ = self.session_dispatch.send(ControlEvent::TurnStop.into()).await;
                self.clone().set_state(state).await;

                break 'turn;
              }
            },
            Err(error) => {
              let signal = SignalPayload::error_from(&error);
              let id = Session::append_signal_to_user(&self.config.session_id, signal.clone().into())
                .await
                .unwrap_or_default();

              let _ = self.session_dispatch.send(signal.with_id(id).into()).await;
              let _ = self.session_dispatch.send(ControlEvent::TurnStop.into()).await;
              tracing::error!("Runtime error: {:#}", error);
              let chain = error.chain().map(|e| e.to_string()).collect::<Vec<String>>();
              tracing::error!("Runtime error chain: {:#?}", chain);
              self.clone().set_state(RuntimeState::Idle).await;

              break 'turn;
            }
          };
        }
        None => {
          self.clone().set_state(RuntimeState::Idle).await;

          break 'turn;
        }
      }
    }
  }

  async fn next_turn(self: Arc<Self>, queue_item: QueueItem, cancel_token: CancellationToken) -> Result<RuntimeState> {
    let prompt = queue_item.display();
    let queue_item_id = queue_item.queue_item_id().to_string();

    let turn_id = Uuid::new_v7();
    let first_step_id = Uuid::new_v7();
    let runtime_context = match RuntimeContext::new(
      self.config.clone(),
      self.session_dispatch.clone(),
      cancel_token.child_token(),
      self.config.session_id.clone(),
      self.config.is_subagent,
      prompt.clone(),
    )
    .await
    {
      Ok(runtime_context) => runtime_context,
      Err(error) => {
        // Do turn cleanup
        self.session_dispatch.send(ControlEvent::TurnStop.into()).await?;

        let history_id = Session::append_user_input(
          &self.config.session_id,
          queue_item.clone().into(),
          turn_id,
          first_step_id,
          ReasoningEffort::Medium,
          Some(HistoryVisibility::User),
        )
        .await?;

        self
          .session_dispatch
          .send(
            PromptEvent::Started(PromptStarted {
              turn_id,
              id: history_id.to_string(),
              prompt: prompt.clone(),
              queue_item_id: Some(queue_item_id),
            })
            .into(),
          )
          .await?;

        return Err(error);
      }
    };

    tracing::info!("Next turn: starting turn: {:?}", runtime_context.session_id);

    runtime_context.set_current_prompt(prompt.clone()).await;
    let runtime_context = Arc::new(runtime_context);

    let history_id = Session::append_user_input(
      &runtime_context.session_id,
      queue_item.clone().into(),
      turn_id,
      first_step_id,
      ReasoningEffort::Medium,
      None,
    )
    .await?;

    self
      .session_dispatch
      .send(
        PromptEvent::Started(PromptStarted {
          turn_id,
          id: history_id.to_string(),
          prompt: prompt.clone(),
          queue_item_id: Some(queue_item_id),
        })
        .into(),
      )
      .await?;

    // Run pre-turn hooks (includes reasoning effort classification)
    runtime_context.hook_registry.run_hooks(HookKind::PreTurn, runtime_context.clone()).await?;

    let current_reasoning_effort = runtime_context.reasoning_effort().await.unwrap_or(ReasoningEffort::Medium);
    tracing::info!("Current reasoning effort: {:?}", current_reasoning_effort);
    Session::set_message_reasoning_effort(&history_id, current_reasoning_effort).await?;

    if let Some(skills) = runtime_context.current_skills().await
      && !skills.is_empty()
    {
      let pretty = skills.iter().map(|s| SkillsUtils::pretty_skill_name(s)).collect::<Vec<_>>().join(", ");
      let signal = SignalPayload::info(format!("Applied skills: {pretty}"));

      if let Ok(history_id) = Session::append_signal_to_user(&runtime_context.session_id, signal.clone().into()).await {
        if let Err(error) = runtime_context.session_dispatch.send(signal.with_id(history_id).into()).await {
          tracing::warn!("Skill matcher: failed to dispatch skill signal: {}", error);
        }
      } else {
        tracing::warn!("Skill matcher: failed to persist skill signal.");
      }
    }

    let result: Result<RuntimeState> = tokio::select! {
      state = self.clone().run_loop(turn_id, Some(first_step_id), runtime_context.clone()) => {
        state?;

        Ok(RuntimeState::Running)
      },
      _ = cancel_token.cancelled() => {
        tracing::info!("Next turn: user interrupted");
        Ok(RuntimeState::Paused(PauseReason::UserInterrupted))
      },
    };

    // Run post-turn hooks
    runtime_context.hook_registry.run_hooks(HookKind::PostTurn, runtime_context.clone()).await?;

    tracing::info!("Next turn: ending turn: {:?}", runtime_context.session_id);

    result
  }

  pub async fn next_turn_system(
    self: Arc<Self>,
    message: String,
    _reasoning_message: String,
    cancel_token: CancellationToken,
  ) -> Result<RuntimeState> {
    let turn_id = Uuid::new_v7();
    let first_step_id = Uuid::new_v7();
    self.clone().set_state(RuntimeState::Running).await;
    let runtime_context = RuntimeContext::new(
      self.config.clone(),
      self.session_dispatch.clone(),
      cancel_token.child_token(),
      self.config.session_id.clone(),
      self.config.is_subagent,
      message.clone(),
    )
    .await?;

    tracing::info!("Next turn (system): starting turn: {:?}", runtime_context.session_id);

    runtime_context.set_current_prompt(message.clone()).await;
    let runtime_context = Arc::new(runtime_context);

    // Run pre-turn hooks (includes reasoning effort classification)
    runtime_context.hook_registry.run_hooks(HookKind::PreTurn, runtime_context.clone()).await?;

    let current_reasoning_effort = runtime_context.reasoning_effort().await.unwrap_or(ReasoningEffort::Medium);
    tracing::info!("Current reasoning effort: {:?}", current_reasoning_effort);

    let signal = SignalPayload::info("Starting work on plan.".into());
    let history_id = Session::append_signal_to_user(&runtime_context.session_id, signal.clone().into()).await?;

    runtime_context.session_dispatch.send(signal.with_id(history_id.clone()).into()).await?;

    let history_id = Session::append_user_input_system(
      &runtime_context.session_id,
      message,
      turn_id,
      first_step_id,
      ReasoningEffort::High,
    )
    .await?;

    // Session::append_assistant_reasoning_from_system(
    //   &runtime_context.session_id,
    //   turn_id,
    //   first_step_id,
    //   reasoning_message,
    // )
    // .await?;

    self
      .session_dispatch
      .send(
        PromptEvent::Started(PromptStarted {
          turn_id,
          id: history_id.to_string(),
          prompt: String::new(),
          queue_item_id: None,
        })
        .into(),
      )
      .await?;

    let result: Result<RuntimeState> = tokio::select! {
      state = self.clone().run_loop(turn_id, Some(first_step_id), runtime_context.clone()) => {
        state?;

        Ok(RuntimeState::Running)
      },
      _ = cancel_token.cancelled() => {
        tracing::info!("Next turn (system): user interrupted");
        Ok(RuntimeState::Paused(PauseReason::UserInterrupted))
      },
    };

    // Run post-turn hooks
    runtime_context.hook_registry.run_hooks(HookKind::PostTurn, runtime_context.clone()).await?;

    tracing::info!("Next turn (system): ending turn: {:?}", runtime_context.session_id);

    self.clone().set_state(RuntimeState::Idle).await;

    result
  }

  async fn maybe_handle_user_interaction_requests(self: Arc<Self>, cancel_token: CancellationToken) -> Result<()> {
    let pending_user_interaction_requests =
      MessageRepositoryV2::get_pending_user_interaction_requests(self.config.session_id.clone())
        .await
        .unwrap_or_default();

    tracing::info!(
      "Pending user interaction requests: {}",
      serde_json::to_string_pretty(&pending_user_interaction_requests).unwrap_or_default()
    );

    if pending_user_interaction_requests.is_empty() {
      return Ok(());
    }

    let runtime_context = Arc::new(
      RuntimeContext::new(
        self.config.clone(),
        self.session_dispatch.clone(),
        cancel_token.child_token(),
        self.config.session_id.clone(),
        self.config.is_subagent,
        String::new(),
      )
      .await?,
    );

    runtime_context.session_dispatch.send(ControlEvent::TurnStart.into()).await?;

    #[derive(Debug, Clone)]
    struct ToolResultWithTurnAndStep {
      result:  ToolResult,
      turn_id: Uuid,
      step_id: Uuid,
    }

    let mut function_calls: JoinSet<ToolResultWithTurnAndStep> = JoinSet::new();

    let mut last_turn_id = Uuid::new_v7();

    for (history_id, question_id, turn_id, step_id) in pending_user_interaction_requests {
      let (tx, rx) = oneshot::channel();
      self.user_interaction_requests.lock().await.insert(question_id.clone(), tx);

      function_calls.spawn(async move {
        let result = rx.await;

        let result = match result {
          Ok(UserInteraction::QuestionAnswer(answer)) => ToolUseResponse::Success(ToolUseResponseSuccess {
            success: true,
            data:    AskQuestionPayload { answer }.into(),
            message: None,
          }),
          Err(e) => ToolUseResponseError::error(ToolId::AskQuestion, e),
        };

        let result =
          ToolResult { history_id: history_id.clone(), tool_use_id: question_id.clone(), result: result };
        ToolResultWithTurnAndStep { result, turn_id, step_id }
      });

      last_turn_id = turn_id;
    }

    while let Some(response) = function_calls.join_next().await {
      if let Ok(response) = response {
        let ToolResultWithTurnAndStep { result, turn_id, step_id } = response;
        let parent_id = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id);
        let completed = Session::complete_tool_request(result.tool_use_id.clone()).await;
        if !completed {
          continue;
        }

        let _ = Session::append_tool_response(&runtime_context.session_id, turn_id, step_id, result.clone(), parent_id)
          .await?;

        runtime_context
          .session_dispatch
          .send(
            ToolCallCompleted {
              id:      result.history_id.to_string(),
              item_id: result.tool_use_id.clone(),
              content: result.result,
            }
            .into(),
          )
          .await?;
      }
    }

    self.run_loop(last_turn_id, None, runtime_context).await?;

    Ok(())
  }

  async fn run_loop(
    self: Arc<Self>,
    turn_id: Uuid,
    mut first_step_id: Option<Uuid>,
    runtime_context: Arc<RuntimeContext>,
  ) -> Result<()> {
    'turn: loop {
      let mut is_turn_end: bool;

      let mut retries = 0;

      'step: loop {
        let mut is_step_done: bool;
        let step_id = match first_step_id.take() {
          Some(step_id) => step_id,
          None => Uuid::new_v7(),
        };

        // Run pre-request hooks
        runtime_context.hook_registry.run_hooks(HookKind::PreStep, runtime_context.clone()).await?;

        if self.queue.is_inject_mode().await && !self.queue.is_empty().await {
          let queue_item = self.queue.clone().recv().await;
          if let Some(queue_item) = queue_item {
            let current_reasoning_effort = runtime_context.reasoning_effort().await.unwrap_or(ReasoningEffort::Medium);
            let history_id = Session::append_user_input(
              &runtime_context.session_id,
              queue_item.clone().into(),
              turn_id,
              step_id,
              current_reasoning_effort,
              Some(HistoryVisibility::User),
            )
            .await?;

            let prompt = inject_user_prompt(queue_item.clone());

            Session::append_user_input(
              &runtime_context.session_id,
              prompt,
              turn_id,
              step_id,
              current_reasoning_effort,
              Some(HistoryVisibility::Assistant),
            )
            .await?;

            runtime_context
              .session_dispatch
              .send(
                PromptEvent::Started(PromptStarted {
                  turn_id,
                  id: history_id.to_string(),
                  prompt: queue_item.display(),
                  queue_item_id: Some(queue_item.queue_item_id().to_string()),
                })
                .into(),
              )
              .await?;
          }
        }

        let mut function_calls: JoinSet<ToolResult> = JoinSet::new();

        let (provider_tx, mut provider_rx) = broadcast::channel(10000);
        let provider_dispatch = ProviderDispatch::new(provider_tx);

        let request = runtime_context.build_chat_request().await?;

        // This will return right away, but the provider will send events to the provider_dispatch
        runtime_context
          .provider_adapter
          .stream_conversation(
            request,
            Some(runtime_context.tools_registry.clone()),
            provider_dispatch.clone(),
            runtime_context.cancel_token.child_token(),
          )
          .await;

        // Do not wait around forever for events. Some errors get swallowed by the provider, so we need to timeout and end the turn.
        let idle_timer = tokio::time::sleep(Duration::from_secs(120));
        tokio::pin!(idle_timer);
        let mut last_reasoning_id = None;
        let mut last_response_id = None;

        // Listen for provider events
        'inner: loop {
          tokio::select! {
            result = provider_rx.recv() => {
              match result {
                Ok(event) => {
                  idle_timer.as_mut().reset(tokio::time::Instant::now() + Duration::from_secs(120));

                  let provider_event_handler = ProviderEventHandler::new(
                    self.config.clone(),
                    self.user_interaction_requests.clone(),
                    self.terminal_managers.clone(),
                  );

                  let result = match provider_event_handler
                    .process_provider_events(
                      turn_id,
                      step_id,
                      runtime_context.clone(),
                      event,
                      &mut function_calls,
                      &mut last_reasoning_id,
                      &mut last_response_id,
                      runtime_context.provider_adapter.provider(),
                    )
                    .await
                  {
                    Ok(result) => result,
                    Err(e) => {
                      tracing::error!("Error processing provider events: {:?}", e);
                      retries += 1;

                      if retries > 3 {
                        is_turn_end = true;
                        is_step_done = true;
                        break 'inner;
                      }

                      let _ =
                        Session::append_assistant_system_error(&runtime_context.session_id, turn_id, step_id, e.into())
                          .await;

                      continue 'step;
                    }
                  };

                  is_turn_end = matches!(result, ControlFlow::EndTurn);
                  is_step_done = is_turn_end || matches!(result, ControlFlow::EndStep);

                  match result {
                    ControlFlow::Continue => continue,
                    _ => break 'inner,
                  }
                }
                Err(e) => {
                  tracing::error!("Error in provider events: {:?}", e);
                  break 'turn;
                }
              }
            }
            _ = &mut idle_timer => {
              tracing::warn!("Idle timer expired, ending turn");

              is_turn_end = true;
              is_step_done = true;
              break 'inner;
            }
          }
        }

        if function_calls.is_empty() && (self.queue.is_empty().await || !self.queue.is_inject_mode().await) {
          is_turn_end = true;
        }

        while let Some(response) = function_calls.join_next().await {
          if let Ok(result) = response {
            let parent_id = MessageRepositoryV2::first_by_step_id(step_id).await?.map(|h| h.id);
            let completed = Session::complete_tool_request(result.tool_use_id.clone()).await;
            if !completed {
              continue;
            }

            let _ =
              Session::append_tool_response(&runtime_context.session_id, turn_id, step_id, result.clone(), parent_id)
                .await?;

            runtime_context
              .session_dispatch
              .send(
                ToolCallCompleted {
                  id:      result.history_id.to_string(),
                  item_id: result.tool_use_id.clone(),
                  content: result.result,
                }
                .into(),
              )
              .await?;
          }
        }

        // Run post-request hooks
        runtime_context.hook_registry.run_hooks(HookKind::PostStep, runtime_context.clone()).await?;

        if is_step_done {
          break 'step;
        }
      }

      if is_turn_end {
        break 'turn;
      }
    }

    Ok(())
  }
}

pub fn inject_user_prompt(user_prompt: QueueItem) -> Vec<MessageContent> {
  let mut middle: Vec<MessageContent> = user_prompt.into();

  let first = "<system-message>The user sent the following message.</system-message>".to_string();
  let last = "<system-message>Please address this message and continue with your tasks.</system-message>".to_string();

  middle.insert(0, MessageContent::Text(MessageText { text: first, signature: None }));
  middle.push(MessageContent::Text(MessageText { text: last, signature: None }));

  middle
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::collections::HashSet;
  use std::sync::Arc;
  use std::sync::Barrier;
  use std::sync::Mutex as StdMutex;

  use common::tools::question::AskQuestionAnswerSource;
  use common::tools::question::AskQuestionClaimStatus;
  use tokio::sync::oneshot;

  use crate::runtime::Runtime;
  use crate::runtime::SlackClaimIdempotencyEntry;
  use crate::runtime::UserInteraction;

  #[test]
  fn claim_first_attempt_is_accepted_when_pending_exists() {
    let question_id = "question-1".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let result = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("yes".to_string()),
      AskQuestionAnswerSource::Desktop,
      None,
    );

    assert_eq!(result.outcome, AskQuestionClaimStatus::Accepted);
    assert!(answered.contains(&question_id));
    assert!(!pending.contains_key(&question_id));
  }

  #[test]
  fn claim_second_attempt_is_rejected_already_answered() {
    let question_id = "question-2".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let first = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("first".to_string()),
      AskQuestionAnswerSource::Desktop,
      None,
    );
    let second = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id,
      UserInteraction::QuestionAnswer("second".to_string()),
      AskQuestionAnswerSource::SlackButton,
      None,
    );

    assert_eq!(first.outcome, AskQuestionClaimStatus::Accepted);
    assert_eq!(second.outcome, AskQuestionClaimStatus::RejectedAlreadyAnswered);
  }

  #[test]
  fn claim_unknown_id_is_invalid() {
    let mut pending: HashMap<String, oneshot::Sender<UserInteraction>> = HashMap::new();
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let result = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      "missing-id".to_string(),
      UserInteraction::QuestionAnswer("n/a".to_string()),
      AskQuestionAnswerSource::SlackButton,
      None,
    );

    assert_eq!(result.outcome, AskQuestionClaimStatus::Invalid);
  }

  #[test]
  fn claim_closed_sender_is_invalid_and_not_marked_answered() {
    let question_id = "question-3".to_string();
    let (tx, rx) = oneshot::channel();
    drop(rx);

    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let first = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("late".to_string()),
      AskQuestionAnswerSource::Desktop,
      None,
    );
    let second = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("retry".to_string()),
      AskQuestionAnswerSource::Desktop,
      None,
    );

    assert_eq!(first.outcome, AskQuestionClaimStatus::Invalid);
    assert_eq!(second.outcome, AskQuestionClaimStatus::Invalid);
    assert!(!answered.contains(&question_id));
  }

  #[test]
  fn claim_result_echoes_answer_source() {
    let question_id = "question-4".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let result = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id,
      UserInteraction::QuestionAnswer("modal answer".to_string()),
      AskQuestionAnswerSource::SlackModal,
      None,
    );

    assert_eq!(result.answer_source, AskQuestionAnswerSource::SlackModal);
    assert_eq!(result.outcome, AskQuestionClaimStatus::Accepted);
  }

  #[test]
  fn claim_is_idempotent_for_duplicate_slack_delivery_key() {
    let question_id = "question-5".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let first = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("first-attempt".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("delivery-1".to_string()),
    );
    let replay = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("replayed-attempt".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("delivery-1".to_string()),
    );
    let loser = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("modal-attempt".to_string()),
      AskQuestionAnswerSource::SlackModal,
      Some("delivery-2".to_string()),
    );

    assert_eq!(first.outcome, AskQuestionClaimStatus::Accepted);
    assert_eq!(replay.outcome, AskQuestionClaimStatus::Accepted);
    assert_eq!(replay.answer_source, AskQuestionAnswerSource::SlackButton);
    assert_eq!(loser.outcome, AskQuestionClaimStatus::RejectedAlreadyAnswered);
  }

  #[test]
  fn claim_same_idempotency_key_for_different_question_is_invalid_and_not_replayed() {
    let question_a = "question-5a".to_string();
    let question_b = "question-5b".to_string();
    let (tx_a, _rx_a) = oneshot::channel();
    let (tx_b, _rx_b) = oneshot::channel();
    let mut pending = HashMap::from([(question_a.clone(), tx_a), (question_b.clone(), tx_b)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let first = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_a.clone(),
      UserInteraction::QuestionAnswer("first-attempt".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("shared-delivery-id".to_string()),
    );
    let second = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_b.clone(),
      UserInteraction::QuestionAnswer("second-attempt".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("shared-delivery-id".to_string()),
    );

    assert_eq!(first.outcome, AskQuestionClaimStatus::Accepted);
    assert_eq!(second.outcome, AskQuestionClaimStatus::Invalid);
    assert!(pending.contains_key(&question_b));

    let third = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_b.clone(),
      UserInteraction::QuestionAnswer("third-attempt".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("unique-delivery-id".to_string()),
    );

    assert_eq!(third.outcome, AskQuestionClaimStatus::Accepted);
  }

  #[test]
  fn concurrent_claims_across_sources_have_single_winner_and_deterministic_losers() {
    let question_id = "question-6".to_string();
    let (tx, _rx) = oneshot::channel();
    let state = Arc::new(StdMutex::new((
      HashMap::from([(question_id.clone(), tx)]),
      HashSet::new(),
      HashMap::<String, SlackClaimIdempotencyEntry>::new(),
    )));
    let barrier = Arc::new(Barrier::new(3));

    let handles: Vec<_> =
      [AskQuestionAnswerSource::Desktop, AskQuestionAnswerSource::SlackButton, AskQuestionAnswerSource::SlackModal]
        .into_iter()
        .enumerate()
        .map(|(idx, source)| {
          let state = state.clone();
          let barrier = barrier.clone();
          let question_id = question_id.clone();
          std::thread::spawn(move || {
            barrier.wait();
            let mut guard = state.lock().expect("mutex lock");
            let (pending, answered, idempotency) = &mut *guard;
            Runtime::claim_ask_question_interaction(
              pending,
              answered,
              idempotency,
              question_id,
              UserInteraction::QuestionAnswer(format!("answer-{idx}")),
              source,
              None,
            )
          })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|handle| handle.join().expect("thread join")).collect();
    let accepted_count = results.iter().filter(|result| result.outcome == AskQuestionClaimStatus::Accepted).count();
    let rejected_count =
      results.iter().filter(|result| result.outcome == AskQuestionClaimStatus::RejectedAlreadyAnswered).count();

    assert_eq!(accepted_count, 1);
    assert_eq!(rejected_count, 2);
    assert!(results.iter().all(|result| result.question_id == "question-6"));
  }

  #[test]
  fn old_unanswered_prompt_attempt_is_accepted_without_any_expiration_gate() {
    let question_id = "question-old-unanswered".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let result = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("late-but-still-valid".to_string()),
      AskQuestionAnswerSource::SlackButton,
      Some("delivery-old-1".to_string()),
    );

    assert_eq!(result.outcome, AskQuestionClaimStatus::Accepted);
    assert!(answered.contains(&question_id));
    assert!(!pending.contains_key(&question_id));
  }

  #[test]
  fn old_interaction_attempt_after_answer_is_rejected_already_answered() {
    let question_id = "question-old-answered".to_string();
    let (tx, _rx) = oneshot::channel();
    let mut pending = HashMap::from([(question_id.clone(), tx)]);
    let mut answered = HashSet::new();
    let mut idempotency = HashMap::<String, SlackClaimIdempotencyEntry>::new();

    let first = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("winner".to_string()),
      AskQuestionAnswerSource::Desktop,
      None,
    );
    let stale_attempt = Runtime::claim_ask_question_interaction(
      &mut pending,
      &mut answered,
      &mut idempotency,
      question_id.clone(),
      UserInteraction::QuestionAnswer("stale-loser".to_string()),
      AskQuestionAnswerSource::SlackModal,
      Some("delivery-old-2".to_string()),
    );

    assert_eq!(first.outcome, AskQuestionClaimStatus::Accepted);
    assert_eq!(stale_attempt.outcome, AskQuestionClaimStatus::RejectedAlreadyAnswered);
  }
}
