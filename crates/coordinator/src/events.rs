use std::sync::Arc;

use anyhow::Result;
use persistence::prelude::RunId;
use shared::events::Events;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub enum CoordinatorEvent {
  StartRun { run_id: RunId, cancel_token: CancellationToken, tx: Arc<oneshot::Sender<Result<()>>> },
}

lazy_static::lazy_static! {
  pub static ref COORDINATOR_EVENTS: Events<CoordinatorEvent> = Events::new();
}
