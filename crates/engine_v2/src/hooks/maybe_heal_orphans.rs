use std::sync::Arc;

use anyhow::Result;
use session::Session;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;

#[derive(Clone, Debug)]
pub struct MaybeHealOrphans;

#[async_trait::async_trait]
impl Hook for MaybeHealOrphans {
  fn name(&self) -> String {
    "MaybeHealOrphans".to_string()
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    Session::heal_orphans(&runtime_context.session_id).await?;

    Ok(())
  }
}
