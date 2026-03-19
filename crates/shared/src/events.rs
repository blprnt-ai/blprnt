use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

pub struct Events<TEvent> {
  bus: Sender<TEvent>,
}

impl<TEvent: Send + Sync + Clone> Events<TEvent> {
  pub fn new() -> Self {
    let (bus, _) = broadcast::channel::<TEvent>(100);

    Self { bus }
  }

  pub fn subscribe(&self) -> Receiver<TEvent> {
    self.bus.subscribe()
  }
}
