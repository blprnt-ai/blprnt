use persistence::prelude::EmployeeId;

use crate::bus::Events;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmployeeEventKind {
  Upsert,
  Delete,
}

#[derive(Clone, Debug)]
pub struct EmployeeEvent {
  pub employee_id: EmployeeId,
  pub kind:        EmployeeEventKind,
}

lazy_static::lazy_static! {
  pub static ref EMPLOYEE_EVENTS: Events<EmployeeEvent> = Events::new();
}
