use std::sync::Arc;

use anyhow::Result;
use persistence::prelude::EmployeeId;
use persistence::prelude::RunId;
use persistence::prelude::RunRecord;
use persistence::prelude::RunTrigger;
use tokio::sync::Mutex;
use tokio::sync::oneshot;

use crate::bus::Events;

pub type OptionalOneshotSender<T> = Option<Arc<Mutex<Option<oneshot::Sender<T>>>>>;

#[derive(Clone, Debug)]
pub enum ApiEvent {
  AddEmployee {
    employee_id: EmployeeId,
  },
  UpdateEmployee {
    employee_id: EmployeeId,
  },
  DeleteEmployee {
    employee_id: EmployeeId,
  },
  StartRun {
    employee_id: EmployeeId,
    run_id:      Option<RunId>,
    trigger:     RunTrigger,
    rx:          OptionalOneshotSender<Result<Option<RunRecord>>>,
  },
  CancelRun {
    employee_id: EmployeeId,
    run_id:      RunId,
  },
}

lazy_static::lazy_static! {
  pub static ref API_EVENTS: Events<ApiEvent> = Events::new();
}
