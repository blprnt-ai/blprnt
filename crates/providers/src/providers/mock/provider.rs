use std::sync::Arc;

use anyhow::Result;
use common::errors::ProviderError;
use common::provider_dispatch::ProviderDispatch;
use common::provider_dispatch::ProviderEvent;
use common::shared::prelude::ChatRequest;
use common::shared::prelude::MessageContent;
use common::shared::prelude::MessageText;
use common::shared::prelude::Provider;
use persistence::prelude::MessageRepositoryV2;
use surrealdb::types::Uuid;
use tokio_util::sync::CancellationToken;

use crate::tools::registry::ToolSchemaRegistry;
use crate::traits::ProviderAdapterTrait;
use crate::types::ChatBasic;

#[derive(Debug)]
pub struct MockProvider;

impl Default for MockProvider {
  fn default() -> Self {
    Self::new()
  }
}

impl MockProvider {
  pub fn new() -> Self {
    Self
  }
}

/// Maps trigger keywords to ProviderError variants for testing.
fn get_mock_error(prompt: &str) -> Option<ProviderError> {
  match prompt {
    "user_cancelled" => Some(ProviderError::UserCancelled),
    "external_network" => {
      Some(ProviderError::ExternalNetwork { context: "mock".into(), message: "mock external network error".into() })
    }
    "decoding_failed" => {
      Some(ProviderError::DecodingFailed { context: "mock".into(), message: "mock decoding failed".into() })
    }
    "llm_mistake" => Some(ProviderError::LlmMistake { context: "mock".into(), message: "mock LLM mistake".into() }),
    "llm_error" => Some(ProviderError::LlmError { context: "mock".into(), message: "mock LLM error".into() }),
    "llm_unknown_error" => {
      Some(ProviderError::LlmUnknownError { context: "mock".into(), message: "mock LLM unknown error".into() })
    }
    "rate_limit" => {
      Some(ProviderError::RateLimit { context: "mock".into(), message: "mock rate limit exceeded".into() })
    }
    "bad_request" => Some(ProviderError::BadRequest { context: "mock".into(), message: "mock bad request".into() }),
    "unauthorized" => Some(ProviderError::Unauthorized {
      url:     "https://mock.api/v1".into(),
      context: "mock".into(),
      message: "mock unauthorized".into(),
    }),
    "cannot_clone_request" => Some(ProviderError::CannotCloneRequest),
    "utf8" => Some(ProviderError::Utf8("mock UTF-8 error".into())),
    "parser" => Some(ProviderError::Parser("mock parser error".into())),
    "transport" => Some(ProviderError::Transport("mock transport error".into())),
    "middleware_transport" => Some(ProviderError::MiddlewareTransport("mock middleware transport error".into())),
    "invalid_content_type" => Some(ProviderError::InvalidContentType("mock invalid content type".into())),
    "invalid_status_code" => Some(ProviderError::InvalidStatusCode {
      code:        "500".into(),
      status_text: "Internal Server Error".into(),
      body:        "mock error body".into(),
    }),
    "stream_ended" => Some(ProviderError::StreamEnded),
    "decode" => Some(ProviderError::Decode("mock decode error".into())),
    "auth_headers" => Some(ProviderError::AuthHeaders("mock auth headers error".into())),
    "timeout" => Some(ProviderError::Timeout),
    "canceled" => Some(ProviderError::Canceled),
    "not_supported" => Some(ProviderError::NotSupported("mock not supported".into())),
    "not_supported_streaming" => Some(ProviderError::NotSupportedStreaming("mock not supported streaming".into())),
    "upstream" => Some(ProviderError::Upstream("mock upstream error".into())),
    "internal" => Some(ProviderError::Internal("mock internal error".into())),
    "invalid_provider" => Some(ProviderError::InvalidProvider("mock invalid provider".into())),
    "encoding" => Some(ProviderError::Encoding("mock encoding error".into())),
    "invalid_schema" => Some(ProviderError::InvalidSchema("mock invalid schema".into())),
    "content_moderation" => Some(ProviderError::ContentModeration {
      code:    "content_filter".into(),
      message: "mock content moderation triggered".into(),
    }),
    "insufficient_credits" => Some(ProviderError::InsufficientCredits {
      code:    "insufficient_funds".into(),
      message: "mock insufficient credits".into(),
    }),
    "model_not_found" => {
      Some(ProviderError::ModelNotFound { model: "mock-model".into(), message: "mock model not found".into() })
    }
    "model_unavailable" => {
      Some(ProviderError::ModelUnavailable { model: "mock-model".into(), message: "mock model unavailable".into() })
    }
    "provider_unavailable" => Some(ProviderError::ProviderUnavailable {
      provider: "mock-provider".into(),
      message:  "mock provider unavailable".into(),
    }),
    "gateway_timeout" => Some(ProviderError::GatewayTimeout { message: "mock gateway timeout".into() }),
    "server_error" => Some(ProviderError::ServerError { code: "500".into(), message: "mock server error".into() }),
    "invalid_api_key" => Some(ProviderError::InvalidApiKey { message: "mock invalid API key".into() }),
    "context_length_exceeded" => Some(ProviderError::ContextLengthExceeded {
      model:   "mock-model".into(),
      message: "mock context length exceeded".into(),
    }),
    _ => None,
  }
}

/// Extracts the prompt text from the last user message in the ChatRequest.
async fn extract_prompt_text(req: &ChatRequest) -> Option<String> {
  let history = match MessageRepositoryV2::list(req.session_id.clone()).await {
    Ok(history) => history,
    Err(e) => {
      tracing::error!("Failed to load history for mock provider: {}", e);
      return None;
    }
  };

  let history = crate::util::history_pruning::apply_pruning(
    history,
    req.llm_model.clone(),
    crate::util::history_pruning::prune_history,
  )
  .await;

  history.iter().rev().find(|m| m.is_user()).and_then(|m| match &m.content() {
    MessageContent::Text(MessageText { text, .. }) => Some(text.clone()),
    _ => None,
  })
}

#[async_trait::async_trait]
impl ProviderAdapterTrait for MockProvider {
  fn provider(&self) -> Provider {
    Provider::Mock
  }

  async fn stream_conversation(
    &self,
    req: ChatRequest,
    _tools: Option<Arc<ToolSchemaRegistry>>,
    provider_dispatch: Arc<ProviderDispatch>,
    _cancel_token: CancellationToken,
  ) {
    let prompt = extract_prompt_text(&req).await.unwrap_or_default();

    // Check if the prompt triggers an error
    if let Some(error) = get_mock_error(&prompt) {
      let _ = provider_dispatch.send(ProviderEvent::Error(error));
      return;
    }

    // Success: dispatch full event sequence
    let rel_id = Uuid::new_v7().to_string();
    let _ = provider_dispatch.send(ProviderEvent::Start(rel_id.clone()));
    let _ = provider_dispatch.send(ProviderEvent::ResponseStarted { rel_id: rel_id.clone() });
    let _ = provider_dispatch.send(ProviderEvent::Response {
      rel_id:    rel_id.clone(),
      content:   format!("Mock: {}", prompt),
      signature: None,
    });
    let _ = provider_dispatch.send(ProviderEvent::ResponseDone { rel_id: rel_id.clone() });
    let _ = provider_dispatch.send(ProviderEvent::Stop(rel_id));
  }

  async fn one_off_request(
    &self,
    prompt: String,
    _system: String,
    _model: Option<String>,
    _cancel_token: CancellationToken,
  ) -> Result<ChatBasic> {
    // Check if the prompt triggers an error
    if let Some(error) = get_mock_error(&prompt) {
      return Err(error.into());
    }

    // Success: return mock response
    Ok(ChatBasic { messages: vec![format!("Mock: {}", prompt)] })
  }
}
