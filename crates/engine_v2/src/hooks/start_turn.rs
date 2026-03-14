use std::sync::Arc;

use anyhow::Result;
use common::session_dispatch::prelude::*;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;

#[derive(Clone, Debug)]
pub struct StartTurn;

#[async_trait::async_trait]
impl Hook for StartTurn {
  fn name(&self) -> String {
    "StartTurn".to_string()
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    let event = ControlEvent::TurnStart;
    tracing::info!("StartTurn: sending turn start event: {:?}", runtime_context.session_id);
    let _ = runtime_context.session_dispatch.send(event.into()).await?;

    Ok(())
  }
}
