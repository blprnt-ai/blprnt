use std::sync::Arc;

use anyhow::Result;
use common::models::ReasoningEffort;
use common::session_dispatch::prelude::ReasoningEffortChanged;
use persistence::prelude::MessageRepositoryV2;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;
use crate::runtime::reasoning_heuristics::classify_reasoning_effort;

#[derive(Clone, Debug)]
pub struct ReasoningEffortClassifier;

#[async_trait::async_trait]
impl Hook for ReasoningEffortClassifier {
  fn name(&self) -> String {
    "ReasoningEffortClassifier".to_string()
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    let Some(prompt) = runtime_context.current_prompt().await else {
      return Ok(());
    };

    let last_10_user_messages = MessageRepositoryV2::last_10_user_messages(runtime_context.session_id.clone())
      .await?
      .iter()
      .map(|h| h.reasoning_effort().unwrap_or(ReasoningEffort::Low))
      .collect::<Vec<ReasoningEffort>>();

    let reasoning_effort = classify_reasoning_effort(
      &prompt,
      &last_10_user_messages,
      runtime_context.provider_adapter.clone(),
      runtime_context.cancel_token.child_token(),
    )
    .await;

    runtime_context.set_reasoning_effort(reasoning_effort).await;
    runtime_context.session_dispatch.send(ReasoningEffortChanged::new(reasoning_effort).into()).await?;

    Ok(())
  }
}
