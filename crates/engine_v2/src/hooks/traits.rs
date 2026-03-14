use std::sync::Arc;

use anyhow::Result;

use crate::runtime::context::RuntimeContext;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum HookKind {
  PreTurn,
  PreStep,
  PostStep,
  PostTurn,
}

#[async_trait::async_trait]
pub trait Hook: Send + Sync + std::fmt::Debug {
  #[allow(unused)]
  fn name(&self) -> String;

  fn enabled(&self) -> bool {
    true
  }

  async fn maybe_run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    if !self.enabled() {
      return Ok(());
    }

    self.run(runtime_context).await
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()>;
}
