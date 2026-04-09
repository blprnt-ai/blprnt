use std::sync::Arc;

use anyhow::Result;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunId;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::bus::Events;

#[derive(Clone, Debug)]
pub enum MinionRunKind {
  Dreamer,
}

#[derive(Clone, Debug)]
pub enum CoordinatorEvent {
  StartRun {
    run_id:       RunId,
    cancel_token: CancellationToken,
    tx:           Arc<Mutex<Option<oneshot::Sender<Result<()>>>>>,
  },
  StartMinionRun {
    employee_id:  EmployeeId,
    kind:         MinionRunKind,
    cancel_token: CancellationToken,
    tx:           Arc<Mutex<Option<oneshot::Sender<Result<()>>>>>,
  },
}

lazy_static::lazy_static! {
  pub static ref COORDINATOR_EVENTS: Events<CoordinatorEvent> = Events::new();
}
