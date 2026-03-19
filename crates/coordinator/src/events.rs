use std::sync::Arc;

use anyhow::Result;
use persistence::prelude::RunId;
use shared::events::Events;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub enum CoordinatorEvent {
  StartRun { run_id: RunId, tx: Arc<oneshot::Sender<Result<()>>> },
  CancelRun { run_id: RunId },
}

lazy_static::lazy_static! {
  pub static ref COORDINATOR_EVENTS: Events<CoordinatorEvent> = Events::new();
}
