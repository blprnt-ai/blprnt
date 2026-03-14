use std::sync::Arc;

use anyhow::Result;
use common::provider_dispatch::ProviderDispatch;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::Provider;
use tokio_util::sync::CancellationToken;

use crate::tools::registry::ToolSchemaRegistry;
use crate::types::ChatBasic;

#[async_trait::async_trait]
pub trait ProviderAdapterTrait: Send + Sync {
  fn provider(&self) -> Provider;

  async fn stream_conversation(
    &self,
    _req: ChatRequest,
    _tools: Option<Arc<ToolSchemaRegistry>>,
    _dispatch: Arc<ProviderDispatch>,
    _cancel_token: CancellationToken,
  );

  async fn one_off_request(
    &self,
    _prompt: String,
    _system: String,
    _model: Option<String>,
    _cancel_token: CancellationToken,
  ) -> Result<ChatBasic> {
    Ok(ChatBasic::default())
  }

  async fn count_tokens(&self, _req: ChatRequest, _tools: &ToolSchemaRegistry) -> Result<u32> {
    Ok(0)
  }
}
