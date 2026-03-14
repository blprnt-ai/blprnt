use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use crate::hooks::traits::Hook;
use crate::hooks::traits::HookKind;
use crate::runtime::context::RuntimeContext;

#[derive(Debug)]
pub struct HookRegistry {
  hooks: HashMap<HookKind, Vec<Box<dyn Hook>>>,
}

impl HookRegistry {
  pub fn new() -> Self {
    Self { hooks: HashMap::new() }
  }

  pub fn register_hook(&mut self, kind: HookKind, hook: Box<dyn Hook>) {
    self.hooks.entry(kind).or_default().push(hook);
  }

  pub async fn run_hooks(&self, kind: HookKind, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    let Some(hooks) = self.hooks.get(&kind) else { return Ok(()) };

    for hook in hooks {
      hook.maybe_run(runtime_context.clone()).await?;
    }

    Ok(())
  }
}
