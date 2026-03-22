use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use persistence::prelude::RunId;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

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

  pub(crate) async fn active_runs(&self) -> HashMap<RunId, CancellationToken> {
    self.active_runs.lock().await.clone()
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
