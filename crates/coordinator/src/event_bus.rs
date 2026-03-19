use persistence::prelude::EmployeeId;
use persistence::prelude::RunId;
use persistence::prelude::RunTrigger;

// Events that are emitted by the application
#[derive(Clone)]
pub enum AppEvent {
  RunStarted(EmployeeId),
  RunCompleted(EmployeeId, RunId),
}

// Triggers are consumed by the application
#[derive(Clone)]
pub enum AppTrigger {
  StartRun { trigger: RunTrigger, employee_id: EmployeeId },
  CancelRun { run_id: RunId },
}

pub struct EventBus<TEvent> {
  bus: tokio::sync::broadcast::Sender<TEvent>,
}

impl<TEvent: Send + Sync + Clone> EventBus<TEvent> {
  pub fn new() -> Self {
    let (bus, _) = tokio::sync::broadcast::channel::<TEvent>(100);

    Self { bus }
  }

  pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<TEvent> {
    self.bus.subscribe()
  }
}

lazy_static::lazy_static! {
  static ref EVENT_BUS: EventBus<AppEvent> = EventBus::new();
  static ref TRIGGER_BUS: EventBus<AppTrigger> = EventBus::new();
}
