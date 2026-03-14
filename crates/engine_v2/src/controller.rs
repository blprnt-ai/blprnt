use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use common::session_dispatch::SessionDispatch;
use common::session_dispatch::prelude::PromptDeleted;
use common::session_dispatch::prelude::PromptEvent;
use common::session_dispatch::prelude::PromptQueued;
use common::shared::prelude::DeleteQueuedPromptOutcome;
use common::shared::prelude::McpRuntimeBridgeRef;
use common::shared::prelude::QueueItem;
use common::shared::prelude::QueueMode;
use common::shared::prelude::SurrealId;
use common::tools::TerminalSnapshot;
use common::tools::question::AskQuestionAnswerSource;
use common::tools::question::AskQuestionClaimResult;
use surrealdb::types::Uuid;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::queue::Queue;
use crate::runtime::Runtime;
use crate::runtime::RuntimeState;
use crate::runtime::UserInteraction;

#[derive(Clone, Debug)]
pub struct ControllerConfig {
  pub sandbox_key:          String,
  pub is_subagent:          bool,
  pub session_id:           SurrealId,
  pub parent_id:            Option<SurrealId>,
  pub mcp_runtime:          Option<McpRuntimeBridgeRef>,
  pub memory_tools_enabled: bool,
}

pub struct Controller {
  config:           ControllerConfig,
  cancel_token:     CancellationToken,
  session_dispatch: Arc<SessionDispatch>,
  queue:            Arc<Queue>,
  runtime:          Arc<Runtime>,
}

impl Controller {
  pub async fn new(config: ControllerConfig) -> Arc<RwLock<Self>> {
    let cancel_token = CancellationToken::new();

    Self::new_with_cancel_token(config, cancel_token).await
  }

  pub async fn new_with_cancel_token(config: ControllerConfig, cancel_token: CancellationToken) -> Arc<RwLock<Self>> {
    let dispatch = SessionDispatch::new(config.session_id.clone(), config.parent_id.clone());
    let queue = Queue::new();
    let runtime = Runtime::new(dispatch.clone(), queue.clone(), config.clone());

    Arc::new(RwLock::new(Self { config, cancel_token, session_dispatch: dispatch, queue, runtime }))
  }

  pub async fn state(&self) -> RuntimeState {
    self.runtime.clone().state().await
  }

  pub async fn run(&self) -> Result<()> {
    let runtime = self.runtime.clone();

    runtime.run(self.cancel_token.child_token()).await
  }

  pub async fn stop(&mut self) {
    self.cancel_token.cancel();
    self.cancel_token.cancelled().await;
    self.cancel_token = CancellationToken::new();
  }

  pub async fn push_prompt(&self, prompt: String, image_urls: Option<Vec<String>>) -> Result<()> {
    // Safe unwrap, from_str will never fail here
    let mut queue_item = QueueItem::from_str(&prompt)?;
    let queue_item_id = queue_item.queue_item_id().to_string();
    if let Some(image_urls) = image_urls {
      for image_url in image_urls {
        queue_item.push_image_url(image_url)?;
      }
    }

    let queued_prompt = queue_item.display();
    self.queue.clone().push(queue_item).await;
    self
      .session_dispatch
      .send(
        PromptEvent::Queued(PromptQueued {
          id: queue_item_id.clone(),
          queue_item_id,
          prompt: if queued_prompt.trim().is_empty() { None } else { Some(queued_prompt) },
        })
        .into(),
      )
      .await?;

    if self.runtime.clone().state().await != RuntimeState::Running {
      self.run().await?;
    }

    Ok(())
  }

  pub fn start_system_turn(&self, message: String, reasoning_message: String) {
    let runtime = self.runtime.clone();
    let cancel_token = self.cancel_token.child_token();

    tokio::spawn({
      let runtime = runtime.clone();

      async move {
        let _ = runtime.clone().next_turn_system(message, reasoning_message, cancel_token.child_token()).await;

        if runtime.clone().state().await != RuntimeState::Running {
          let _ = runtime.run(cancel_token.child_token()).await;
        }
      }
    });
  }

  pub async fn answer_question(
    &self,
    question_id: String,
    answer: String,
    answer_source: AskQuestionAnswerSource,
  ) -> Result<AskQuestionClaimResult> {
    self.answer_question_with_idempotency(question_id, answer, answer_source, None).await
  }

  pub async fn answer_question_with_idempotency(
    &self,
    question_id: String,
    answer: String,
    answer_source: AskQuestionAnswerSource,
    idempotency_key: Option<String>,
  ) -> Result<AskQuestionClaimResult> {
    let runtime = self.runtime.clone();

    runtime
      .handle_user_interaction_request_with_idempotency(
        question_id,
        UserInteraction::QuestionAnswer(answer),
        answer_source,
        idempotency_key,
      )
      .await
  }

  pub async fn edit_prompt(&self, index: usize, prompt: String) -> Result<()> {
    let queue_item = QueueItem::from_str(&prompt)?;
    self.queue.clone().edit(index, queue_item).await;
    Ok(())
  }

  pub async fn remove_prompt(&self, index: usize) {
    self.queue.clone().remove(index).await;
  }

  pub async fn delete_queued_prompt(&self, queue_item_id: &str) -> DeleteQueuedPromptOutcome {
    let outcome = self.queue.clone().delete_by_queue_item_id(queue_item_id).await;

    if outcome == DeleteQueuedPromptOutcome::Deleted {
      let _ = self
        .session_dispatch
        .send(PromptEvent::Deleted(PromptDeleted { queue_item_id: queue_item_id.to_string() }).into())
        .await;
    }

    outcome
  }

  pub async fn close_terminal(&self, terminal_id: Uuid) -> Result<()> {
    self.runtime.clone().close_terminal(terminal_id).await
  }

  pub async fn get_terminal_snapshot(&self, terminal_id: Uuid) -> Result<TerminalSnapshot> {
    self.runtime.clone().get_terminal_snapshot(terminal_id).await
  }

  #[cfg(feature = "testing")]
  pub async fn testing_push_queue_item(&self, queue_item: QueueItem) {
    self.queue.clone().push(queue_item).await;
  }

  #[cfg(feature = "testing")]
  pub async fn testing_pop_queue_item(&self) -> Option<QueueItem> {
    self.queue.clone().recv().await
  }

  pub async fn queue_mode(&self) -> QueueMode {
    self.queue.mode().await
  }

  pub async fn set_queue_mode(&self, mode: QueueMode) {
    self.queue.clone().mode_changed(mode).await;
  }

  pub fn is_subagent(&self) -> bool {
    self.config.is_subagent
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;
  use std::sync::Arc;

  use common::blprnt_dispatch::BlprntDispatch;
  use common::session_dispatch::prelude::PromptEvent;
  use common::shared::prelude::SurrealId;

  use super::*;

  async fn next_prompt_event_for_session(
    receiver: &mut tokio::sync::broadcast::Receiver<common::blprnt_dispatch::SessionEvent>,
    session_id: &SurrealId,
  ) -> PromptEvent {
    loop {
      let event = tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv())
        .await
        .expect("timed out waiting for session event")
        .expect("session channel closed");

      if event.session_id != session_id.clone() {
        continue;
      }

      if let common::session_dispatch::prelude::SessionDispatchEvent::Prompt(prompt_event) = event.event_data {
        return prompt_event;
      }
    }
  }

  async fn build_controller(session_id: SurrealId) -> Arc<RwLock<Controller>> {
    Controller::new(ControllerConfig {
      sandbox_key: "test".to_string(),
      is_subagent: false,
      session_id,
      parent_id: None,
      mcp_runtime: None,
      memory_tools_enabled: true,
    })
    .await
  }

  #[tokio::test]
  async fn push_prompt_emits_queued_event_with_queue_item_id() {
    let session_id = SurrealId::new("sessions".to_string());
    let controller = build_controller(session_id.clone()).await;
    let dispatch = BlprntDispatch::get_or_init();
    let mut receiver = dispatch.tx.subscribe();

    let result = controller.read().await.push_prompt("queued prompt".to_string(), None).await;
    assert!(result.is_ok());

    loop {
      let prompt_event = next_prompt_event_for_session(&mut receiver, &session_id).await;
      match prompt_event {
        PromptEvent::Queued(payload) => {
          assert!(!payload.queue_item_id.is_empty());
          assert_eq!(payload.id, payload.queue_item_id);
          assert_eq!(payload.prompt.as_deref(), Some("queued prompt"));
          break;
        }
        _ => continue,
      }
    }
  }

  #[tokio::test]
  async fn delete_pending_item_returns_deleted_and_emits_deleted_event() {
    let session_id = SurrealId::new("sessions".to_string());
    let controller = build_controller(session_id.clone()).await;
    let dispatch = BlprntDispatch::get_or_init();
    let mut receiver = dispatch.tx.subscribe();

    let item = QueueItem::from_str("pending").unwrap();
    let queue_item_id = item.queue_item_id().to_string();
    let queue = controller.read().await.queue.clone();
    queue.push(item).await;

    let outcome = controller.read().await.delete_queued_prompt(&queue_item_id).await;
    assert_eq!(outcome, DeleteQueuedPromptOutcome::Deleted);

    loop {
      let prompt_event = next_prompt_event_for_session(&mut receiver, &session_id).await;
      match prompt_event {
        PromptEvent::Deleted(payload) => {
          assert_eq!(payload.queue_item_id, queue_item_id);
          break;
        }
        _ => continue,
      }
    }
  }

  #[tokio::test]
  async fn delete_after_pop_returns_already_started_without_deleted_event() {
    let session_id = SurrealId::new("sessions".to_string());
    let controller = build_controller(session_id.clone()).await;
    let dispatch = BlprntDispatch::get_or_init();
    let mut receiver = dispatch.tx.subscribe();

    let item = QueueItem::from_str("popped").unwrap();
    let queue_item_id = item.queue_item_id().to_string();
    let queue = controller.read().await.queue.clone();
    queue.clone().push(item).await;
    let _ = queue.recv().await;

    let outcome = controller.read().await.delete_queued_prompt(&queue_item_id).await;
    assert_eq!(outcome, DeleteQueuedPromptOutcome::AlreadyStarted);

    let no_deleted_event = tokio::time::timeout(std::time::Duration::from_millis(200), async {
      loop {
        let event = receiver.recv().await.expect("session channel closed");
        if event.session_id != session_id.clone() {
          continue;
        }

        if let common::session_dispatch::prelude::SessionDispatchEvent::Prompt(PromptEvent::Deleted(_)) =
          event.event_data
        {
          panic!("deleted event must not be emitted for AlreadyStarted outcome");
        }
      }
    })
    .await;

    assert!(no_deleted_event.is_err());
  }

  #[tokio::test]
  async fn delete_unknown_id_returns_not_found_without_deleted_event() {
    let session_id = SurrealId::new("sessions".to_string());
    let controller = build_controller(session_id.clone()).await;
    let dispatch = BlprntDispatch::get_or_init();
    let mut receiver = dispatch.tx.subscribe();

    let outcome = controller.read().await.delete_queued_prompt("missing-queue-item").await;
    assert_eq!(outcome, DeleteQueuedPromptOutcome::NotFound);

    let no_deleted_event = tokio::time::timeout(std::time::Duration::from_millis(200), async {
      loop {
        let event = receiver.recv().await.expect("session channel closed");
        if event.session_id != session_id.clone() {
          continue;
        }

        if let common::session_dispatch::prelude::SessionDispatchEvent::Prompt(PromptEvent::Deleted(_)) =
          event.event_data
        {
          panic!("deleted event must not be emitted for NotFound outcome");
        }
      }
    })
    .await;

    assert!(no_deleted_event.is_err());
  }
}
