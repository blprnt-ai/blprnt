use std::sync::Arc;

use anyhow::Result;
use common::shared::prelude::*;
use session::Session;

use crate::hooks::traits::Hook;
use crate::runtime::context::RuntimeContext;

#[derive(Clone, Debug)]
pub struct SessionTokenUsage;

#[async_trait::async_trait]
impl Hook for SessionTokenUsage {
  fn name(&self) -> String {
    "SessionTokenUsage".to_string()
  }

  fn enabled(&self) -> bool {
    false
  }

  async fn run(&self, runtime_context: Arc<RuntimeContext>) -> Result<()> {
    let request = runtime_context.build_chat_request().await?;
    let response =
      runtime_context.provider_adapter.count_tokens(request, &runtime_context.tools_registry).await.unwrap_or(0);

    let _ = Session::update_token_usage(&runtime_context.session_id, response).await;

    let event = TokenUsage { input_tokens: response, output_tokens: 0 };
    let _ = runtime_context.session_dispatch.send(event.into()).await;

    Ok(())
  }
}
