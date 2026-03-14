use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;

use common::shared::prelude::*;
use tokio::sync::Mutex;
use tokio::sync::Notify;

#[derive(Debug, Default)]
struct QueueState {
  items:                  VecDeque<QueueItem>,
  started_queue_item_ids: HashSet<String>,
}

#[derive(Debug)]
pub struct Queue {
  state:  Mutex<QueueState>,
  mode:   Mutex<QueueMode>,
  notify: Notify,
}

impl Queue {
  #[allow(dead_code)]
  fn index_of(items: &VecDeque<QueueItem>, queue_item_id: &str) -> Option<usize> {
    items.iter().position(|item| item.queue_item_id() == queue_item_id)
  }

  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      state:  Mutex::new(QueueState::default()),
      mode:   Mutex::new(QueueMode::Queue),
      notify: Notify::new(),
    })
  }

  pub async fn push(self: Arc<Self>, item: QueueItem) {
    {
      let mut guard = self.state.lock().await;
      guard.items.push_back(item);
    }

    self.notify.notify_one();
  }

  pub async fn recv(self: Arc<Self>) -> Option<QueueItem> {
    loop {
      let mut guard = self.state.lock().await;
      if let Some(item) = guard.items.pop_front() {
        guard.started_queue_item_ids.insert(item.queue_item_id().to_string());
        return Some(item);
      }

      if guard.items.is_empty() {
        return None;
      }

      self.notify.notified().await;
    }
  }

  pub async fn edit(self: Arc<Self>, index: usize, item: QueueItem) {
    {
      let mut guard = self.state.lock().await;
      guard.items[index] = item;
    }
  }

  pub async fn remove(self: Arc<Self>, index: usize) {
    {
      let mut guard = self.state.lock().await;
      guard.items.remove(index);
    }
  }

  #[allow(dead_code)]
  pub async fn contains_queue_item_id(self: Arc<Self>, queue_item_id: &str) -> bool {
    let guard = self.state.lock().await;

    Self::index_of(&guard.items, queue_item_id).is_some()
  }

  #[allow(dead_code)]
  pub async fn find_by_queue_item_id(self: Arc<Self>, queue_item_id: &str) -> Option<QueueItem> {
    let guard = self.state.lock().await;

    Self::index_of(&guard.items, queue_item_id).and_then(|index| guard.items.get(index).cloned())
  }

  #[allow(dead_code)]
  pub async fn remove_by_queue_item_id(self: Arc<Self>, queue_item_id: &str) -> Option<QueueItem> {
    let mut guard = self.state.lock().await;
    let index = Self::index_of(&guard.items, queue_item_id)?;

    guard.items.remove(index)
  }

  pub async fn delete_by_queue_item_id(self: Arc<Self>, queue_item_id: &str) -> DeleteQueuedPromptOutcome {
    let mut guard = self.state.lock().await;
    if let Some(index) = Self::index_of(&guard.items, queue_item_id) {
      let _ = guard.items.remove(index);
      return DeleteQueuedPromptOutcome::Deleted;
    }

    if guard.started_queue_item_ids.contains(queue_item_id) {
      return DeleteQueuedPromptOutcome::AlreadyStarted;
    }

    DeleteQueuedPromptOutcome::NotFound
  }

  pub async fn mode(&self) -> QueueMode {
    let guard = self.mode.lock().await;
    guard.clone()
  }

  pub async fn mode_changed(self: Arc<Self>, mode: QueueMode) {
    {
      let mut guard = self.mode.lock().await;
      *guard = mode;
    }
  }

  pub async fn is_inject_mode(&self) -> bool {
    let guard = self.mode.lock().await;
    guard.clone() == QueueMode::Inject
  }

  pub async fn is_empty(&self) -> bool {
    let guard = self.state.lock().await;
    guard.items.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  #[tokio::test]
  async fn remove_by_queue_item_id_removes_matching_item_and_preserves_fifo() {
    let queue = Queue::new();
    let first = QueueItem::from_str("first").unwrap();
    let second = QueueItem::from_str("second").unwrap();
    let third = QueueItem::from_str("third").unwrap();

    let second_id = second.queue_item_id().to_string();

    queue.clone().push(first.clone()).await;
    queue.clone().push(second.clone()).await;
    queue.clone().push(third.clone()).await;

    let removed = queue.clone().remove_by_queue_item_id(&second_id).await;

    assert_eq!(removed.as_ref().map(|item| item.queue_item_id()), Some(second_id.as_str()));
    assert_eq!(queue.clone().recv().await.as_ref().map(|item| item.queue_item_id()), Some(first.queue_item_id()));
    assert_eq!(queue.clone().recv().await.as_ref().map(|item| item.queue_item_id()), Some(third.queue_item_id()));
  }

  #[tokio::test]
  async fn remove_by_queue_item_id_returns_none_for_missing_item() {
    let queue = Queue::new();
    let only = QueueItem::from_str("only").unwrap();

    queue.clone().push(only.clone()).await;

    let removed = queue.clone().remove_by_queue_item_id("missing-id").await;

    assert!(removed.is_none());
    assert_eq!(queue.clone().recv().await.as_ref().map(|item| item.queue_item_id()), Some(only.queue_item_id()));
  }

  #[tokio::test]
  async fn contains_and_find_by_queue_item_id_reflect_queue_state() {
    let queue = Queue::new();
    let item = QueueItem::from_str("item").unwrap();
    let item_id = item.queue_item_id().to_string();

    assert!(!queue.clone().contains_queue_item_id(&item_id).await);
    assert!(queue.clone().find_by_queue_item_id(&item_id).await.is_none());

    queue.clone().push(item.clone()).await;

    assert!(queue.clone().contains_queue_item_id(&item_id).await);
    assert_eq!(
      queue.clone().find_by_queue_item_id(&item_id).await.as_ref().map(|queued_item| queued_item.queue_item_id()),
      Some(item_id.as_str())
    );
  }

  #[tokio::test]
  async fn delete_by_queue_item_id_returns_deleted_for_pending_item() {
    let queue = Queue::new();
    let first = QueueItem::from_str("first").unwrap();
    let second = QueueItem::from_str("second").unwrap();
    let second_id = second.queue_item_id().to_string();

    queue.clone().push(first.clone()).await;
    queue.clone().push(second).await;

    let outcome = queue.clone().delete_by_queue_item_id(&second_id).await;

    assert_eq!(outcome, DeleteQueuedPromptOutcome::Deleted);
    assert_eq!(queue.clone().recv().await.as_ref().map(|item| item.queue_item_id()), Some(first.queue_item_id()));
  }

  #[tokio::test]
  async fn delete_by_queue_item_id_returns_already_started_after_pop() {
    let queue = Queue::new();
    let item = QueueItem::from_str("item").unwrap();
    let item_id = item.queue_item_id().to_string();

    queue.clone().push(item).await;
    let _ = queue.clone().recv().await;

    let outcome = queue.clone().delete_by_queue_item_id(&item_id).await;

    assert_eq!(outcome, DeleteQueuedPromptOutcome::AlreadyStarted);
  }

  #[tokio::test]
  async fn delete_by_queue_item_id_returns_not_found_for_unknown_id() {
    let queue = Queue::new();

    let outcome = queue.clone().delete_by_queue_item_id("missing").await;

    assert_eq!(outcome, DeleteQueuedPromptOutcome::NotFound);
  }
}
