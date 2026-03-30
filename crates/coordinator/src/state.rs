use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use persistence::prelude::RunId;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct EmployeeRuntimeState {
  running_count:     AtomicUsize,
  completion_notify: Notify,
  active_runs:       Mutex<HashMap<RunId, CancellationToken>>,
}

impl EmployeeRuntimeState {
  pub(crate) fn new() -> Self {
    Self {
      running_count:     AtomicUsize::new(0),
      completion_notify: Notify::new(),
      active_runs:       Mutex::new(HashMap::new()),
    }
  }

  pub(crate) async fn register_run(&self, run_id: RunId, cancel_token: CancellationToken) {
    self.active_runs.lock().await.insert(run_id, cancel_token);
  }

  pub(crate) async fn finish_run(&self, run_id: &RunId) -> Option<usize> {
    let removed = self.active_runs.lock().await.remove(run_id).is_some();
    removed.then(|| self.release_slot())
  }

  pub(crate) async fn notified(&self) {
    self.completion_notify.notified().await
  }

  pub(crate) fn try_reserve_slot(&self, max_concurrent_runs: usize) -> bool {
    loop {
      let running_count = self.running_count.load(Ordering::Acquire);

      if running_count >= max_concurrent_runs {
        return false;
      }

      if self
        .running_count
        .compare_exchange(running_count, running_count + 1, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
      {
        return true;
      }
    }
  }

  pub(crate) fn release_slot(&self) -> usize {
    let previous_count = self.running_count.fetch_sub(1, Ordering::AcqRel);
    let remaining_count = previous_count.saturating_sub(1);

    self.completion_notify.notify_one();

    remaining_count
  }

  pub(crate) async fn cancel_run(&self, run_id: &RunId) {
    let mut active_runs = self.active_runs.lock().await;
    if let Some(cancel_token) = active_runs.remove(run_id) {
      cancel_token.cancel();
      self.release_slot();
    }
  }
}

#[cfg(test)]
mod tests {
  use persistence::Uuid;

  use super::*;

  #[tokio::test]
  async fn cancelling_a_registered_run_releases_its_slot_only_once() {
    let state = EmployeeRuntimeState::new();
    let run_id = RunId::from(Uuid::new_v4());
    let cancel_token = CancellationToken::new();

    assert!(state.try_reserve_slot(1), "initial slot reservation should succeed");

    state.register_run(run_id.clone(), cancel_token.clone()).await;
    state.cancel_run(&run_id).await;

    assert!(cancel_token.is_cancelled(), "cancellation should reach the registered run");
    assert!(state.try_reserve_slot(1), "cancelled run should free its slot");

    state.finish_run(&run_id).await;

    assert!(!state.try_reserve_slot(1), "finishing an already-cancelled run must not release the same slot twice");
  }
}
