pub mod prelude;

mod events;

use std::sync::Arc;

use anyhow::Result;
use events::SessionDispatchEvent;

use crate::blprnt_dispatch::BlprntDispatch;
use crate::blprnt_dispatch::SessionEvent;
use crate::shared::prelude::SurrealId;

#[derive(Debug)]
pub struct SessionDispatch {
  session_id: SurrealId,
  parent_id:  Option<SurrealId>,
}

impl SessionDispatch {
  pub fn new(session_id: SurrealId, parent_id: Option<SurrealId>) -> Arc<Self> {
    Arc::new(Self { session_id, parent_id })
  }

  pub async fn send(&self, event: SessionDispatchEvent) -> Result<()> {
    let session_event =
      SessionEvent { session_id: self.session_id.clone(), parent_id: self.parent_id.clone(), event_data: event };

    BlprntDispatch::send(session_event).await
  }
}
