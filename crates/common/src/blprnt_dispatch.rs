use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;
use serde::ser::SerializeStruct;
use surrealdb::types::ToSql;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

use crate::session_dispatch::prelude::SessionDispatchEvent;
use crate::shared::prelude::SurrealId;

lazy_static::lazy_static! {
  static ref BLPRNT_DISPATCH: RwLock<Option<Arc<BlprntDispatch>>> = RwLock::new(None);
}

#[derive(Clone, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
  #[specta(type = String)]
  pub session_id: SurrealId,
  #[specta(type = String)]
  pub parent_id:  Option<SurrealId>,
  pub event_data: SessionDispatchEvent,
}

impl serde::Serialize for SessionEvent {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("SessionEvent", 3)?;

    state.serialize_field("sessionId", &self.session_id.0.to_sql())?;
    state.serialize_field("parentId", &self.parent_id.as_ref().map(|id| id.0.to_sql()))?;
    state.serialize_field("eventData", &self.event_data)?;

    state.end()
  }
}

pub struct BlprntDispatch {
  pub tx: Sender<SessionEvent>,
}

impl BlprntDispatch {
  pub fn get_or_init() -> Arc<Self> {
    if let Some(dispatch) = BLPRNT_DISPATCH.read().expect("blprnt dispatch lock poisoned").as_ref() {
      return dispatch.clone();
    }

    let mut dispatch_guard = BLPRNT_DISPATCH.write().expect("blprnt dispatch lock poisoned");

    if let Some(dispatch) = dispatch_guard.as_ref() {
      return dispatch.clone();
    }

    let (tx, _rx) = broadcast::channel(10000);
    let dispatch = Arc::new(BlprntDispatch { tx });
    *dispatch_guard = Some(dispatch.clone());
    dispatch
  }

  pub async fn send(event: SessionEvent) -> Result<()> {
    let dispatch = Self::get_or_init();
    let _ = dispatch.tx.send(event);

    Ok(())
  }

  pub async fn recv(session_id: SurrealId) -> SessionEvent {
    loop {
      let dispatch = Self::get_or_init();
      let mut rx = dispatch.tx.subscribe();

      'recv: loop {
        match rx.recv().await {
          Ok(event) if event.session_id == session_id.clone() => return event,
          Err(broadcast::error::RecvError::Closed) => break 'recv,
          _ => continue 'recv,
        }
      }

      tracing::warn!("blprnt dispatch channel closed, reinitializing");
      *BLPRNT_DISPATCH.write().expect("blprnt dispatch lock poisoned") = None;

      tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
  }

  #[cfg(not(feature = "testing"))]
  pub fn run() {
    let dispatch = Self::get_or_init();
    let mut rx = dispatch.tx.subscribe();

    tokio::task::spawn(async move {
      use crate::blprnt::Blprnt;
      use crate::blprnt::BlprntEventKind;

      while let Ok(event) = rx.recv().await {
        Blprnt::emit(BlprntEventKind::SessionEvent, event.clone().into());
      }
    });
  }

  #[cfg(feature = "testing")]
  pub fn run() {}
}
