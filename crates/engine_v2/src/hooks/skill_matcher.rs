use std::sync::Arc;

use anyhow::Result;
use persistence::prelude::MessageRepositoryV2;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;
use crate::runtime::skill_matcher_heuristics::match_skills_for_turn;

const MATCHER_MESSAGE_FETCH_LIMIT: usize = 25;

#[derive(Clone, Debug)]
pub struct SkillMatcherHook;

#[async_trait::async_trait]
impl Hook for SkillMatcherHook {
  fn name(&self) -> String {
    "SkillMatcherHook".to_string()
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    let Some(prompt) = runtime_context.current_prompt().await else {
      return Ok(());
    };

    let recent_user_messages =
      MessageRepositoryV2::last_user_messages(runtime_context.session_id.clone(), MATCHER_MESSAGE_FETCH_LIMIT)
        .await
        .unwrap_or_else(|error| {
          tracing::warn!("Skill matcher: failed to read recent user messages: {}", error);
          vec![]
        });

    let matched_skills = match_skills_for_turn(
      prompt,
      recent_user_messages,
      runtime_context.provider_adapter.clone(),
      runtime_context.cancel_token.child_token(),
    )
    .await;

    tracing::info!("Skill matcher: matched skills: {:?}", matched_skills);

    runtime_context.set_current_skills(matched_skills.clone()).await;

    Ok(())
  }
}
