use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use common::errors::ProviderError;
use common::provider_dispatch::ProviderDispatch;
use common::shared::prelude::BlprntCredentials;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::Provider;
use tokio_util::sync::CancellationToken;

use crate::providers::anthropic::provider::AnthropicProvider;
use crate::providers::mock::provider::MockProvider;
use crate::providers::openai::provider::OpenAiProvider;
use crate::providers::openai::provider::OpenAiProviderType;
use crate::providers::openrouter::OpenRouterProvider;
use crate::tools::registry::ToolSchemaRegistry;
use crate::traits::ProviderAdapterTrait;
use crate::types::ChatBasic;

#[derive(Debug)]
pub enum ProviderAdapter {
  Anthropic(Arc<AnthropicProvider>),
  OpenAi(Arc<OpenAiProvider>),
  OpenRouter(Arc<OpenRouterProvider>),
  Mock(Arc<MockProvider>),
}

impl ProviderAdapter {
  pub fn provider(&self) -> Provider {
    match self {
      Self::Anthropic(p) => p.provider(),
      Self::OpenAi(p) => p.provider(),
      Self::OpenRouter(p) => p.provider(),
      Self::Mock(p) => p.provider(),
    }
  }

  pub async fn credentials(&self) -> Option<BlprntCredentials> {
    match self {
      Self::Anthropic(p) => p.get_credentials().await,
      _ => None,
    }
  }

  pub async fn update_credentials(&self, credentials: BlprntCredentials) {
    if let Self::Anthropic(p) = self {
      p.update_credentials(credentials).await
    }
  }

  pub async fn stream_conversation(
    &self,
    req: ChatRequest,
    tools: Option<Arc<ToolSchemaRegistry>>,
    provider_dispatch: Arc<ProviderDispatch>,
    cancel_token: CancellationToken,
  ) {
    match self {
      Self::Anthropic(provider) => {
        tokio::spawn({
          let provider = provider.clone();

          // Give some time for the caller to subscribe to the provider dispatch
          tokio::time::sleep(Duration::from_millis(400)).await;
          async move { provider.stream_conversation(req, tools, provider_dispatch, cancel_token).await }
        });
      }
      Self::OpenAi(provider) => {
        tokio::spawn({
          let provider = provider.clone();

          // Give some time for the caller to subscribe to the provider dispatch
          tokio::time::sleep(Duration::from_millis(400)).await;
          async move { provider.stream_conversation(req, tools, provider_dispatch, cancel_token).await }
        });
      }
      Self::OpenRouter(provider) => {
        tokio::spawn({
          let provider = provider.clone();

          // Give some time for the caller to subscribe to the provider dispatch
          tokio::time::sleep(Duration::from_millis(400)).await;
          async move { provider.stream_conversation(req, tools, provider_dispatch, cancel_token).await }
        });
      }
      Self::Mock(provider) => {
        tokio::spawn({
          let provider = provider.clone();

          // Give some time for the caller to subscribe to the provider dispatch
          tokio::time::sleep(Duration::from_millis(400)).await;
          async move { provider.stream_conversation(req, tools, provider_dispatch, cancel_token).await }
        });
      }
    };
  }

  pub async fn one_off_request(
    &self,
    prompt: String,
    system: String,
    model: Option<String>,
    cancel_token: CancellationToken,
  ) -> Result<ChatBasic> {
    match self {
      Self::Anthropic(p) => p.one_off_request(prompt, system, model, cancel_token).await,
      Self::OpenAi(p) => p.one_off_request(prompt, system, model, cancel_token).await,
      Self::OpenRouter(p) => p.one_off_request(prompt, system, model, cancel_token).await,
      Self::Mock(p) => p.one_off_request(prompt, system, model, cancel_token).await,
    }
  }

  pub async fn count_tokens(&self, req: ChatRequest, tools: &ToolSchemaRegistry) -> Result<u32> {
    match self {
      Self::Anthropic(p) => p.count_tokens(req, tools).await,
      Self::OpenAi(p) => p.count_tokens(req, tools).await,
      Self::OpenRouter(p) => p.count_tokens(req, tools).await,
      Self::Mock(p) => p.count_tokens(req, tools).await,
    }
  }
}

pub fn build_adapter(
  provider: &Provider,
  credentials: Option<BlprntCredentials>,
  base_url: Option<String>,
) -> Result<ProviderAdapter> {
  let engine_provider_adapter = match provider {
    Provider::Blprnt => {
      let provider =
        OpenAiProvider::new(OpenAiProviderType::Blprnt("https://chat.blprnt.ai/v1".to_string()), Some(10_000), None);

      ProviderAdapter::OpenAi(Arc::new(provider))
    }

    Provider::OpenAi | Provider::OpenAiFnf => {
      let provider = OpenAiProvider::new(OpenAiProviderType::OpenAi(credentials.unwrap()), None, base_url);

      ProviderAdapter::OpenAi(Arc::new(provider))
    }

    Provider::Anthropic | Provider::AnthropicFnf => {
      let provider = AnthropicProvider::new(credentials.unwrap(), base_url);

      ProviderAdapter::Anthropic(Arc::new(provider))
    }

    Provider::OpenRouter => {
      let BlprntCredentials::ApiKey(api_key) = credentials.unwrap() else {
        return Err(ProviderError::auth("Invalid credentials").into());
      };

      let provider = OpenRouterProvider::new(api_key);

      ProviderAdapter::OpenRouter(Arc::new(provider))
    }

    Provider::Mock => {
      let provider = MockProvider::new();

      ProviderAdapter::Mock(Arc::new(provider))
    }
  };

  Ok(engine_provider_adapter)
}
