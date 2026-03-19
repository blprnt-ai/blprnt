use persistence::prelude::EmployeeId;
use persistence::prelude::RunId;
use shared::events::Events;

#[derive(Clone, Debug)]
pub enum ApiEvent {
  CancelRun { employee_id: EmployeeId, run_id: RunId },
}

lazy_static::lazy_static! {
  pub static ref API_EVENTS: Events<ApiEvent> = Events::new();
}
