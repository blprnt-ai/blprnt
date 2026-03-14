use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use common::agent::prelude::*;
use common::errors::ProviderError;
use common::event_source::Event;
use common::event_source::EventSourceError;
use common::event_source::RequestBuilderExt;
use common::event_source::retry::Never;
use common::provider_dispatch::ProviderDispatch;
use common::provider_dispatch::ProviderEvent;
use common::shared::prelude::*;
use futures_util::StreamExt;
use futures_util::lock::Mutex;
use http::HeaderMap;
use http::HeaderValue;
use http::header::ACCEPT;
use http::header::ACCEPT_ENCODING;
use http::header::ACCEPT_LANGUAGE;
use http::header::AUTHORIZATION;
use http::header::USER_AGENT;
use reqwest_middleware::RequestBuilder;
use surrealdb::types::Uuid;
use tokio_util::sync::CancellationToken;

use crate::providers::anthropic::client::AnthropicHttp;
use crate::providers::anthropic::error::AnthropicErrorResponse;
use crate::providers::anthropic::mapping::AnthropicMapping;
use crate::providers::anthropic::sse_types::AnthropicContentBlock;
use crate::providers::anthropic::sse_types::AnthropicStreamEvent;
use crate::providers::anthropic::sse_types::Delta;
use crate::providers::anthropic::tools::AnthropicTools;
use crate::providers::anthropic::types::Beta;
use crate::providers::anthropic::types::ChatBasicResponse;
use crate::providers::anthropic::types::CountTokensResponse;
use crate::providers::anthropic::types::MessageRequestBody;
use crate::providers::anthropic::types::SystemRequestBody;
use crate::tools::registry::ToolSchemaRegistry;
use crate::traits::ProviderAdapterTrait;
use crate::types::ChatBasic;
use crate::types::ContentBlock;
use crate::types::ParsedContentBlock;
use crate::util::get_oauth_slug;
use crate::util::sse::SseItem;

#[derive(Debug)]
pub struct AnthropicProvider {
  http:        Arc<AnthropicHttp>,
  base_url:    String,
  timeout_ms:  u64,
  credentials: Arc<Mutex<BlprntCredentials>>,
}

impl AnthropicProvider {
  pub fn new(credentials: BlprntCredentials, base_url: Option<String>) -> Self {
    let http = match &credentials {
      BlprntCredentials::ApiKey(api_key) => AnthropicHttp::get_client(api_key),
      BlprntCredentials::OauthToken(OauthToken::Anthropic(AnthropicOauthToken { access_token, .. })) => {
        AnthropicHttp::get_client(access_token)
      }
      _ => unreachable!(),
    };

    Self {
      http,
      base_url: base_url.unwrap_or("https://api.anthropic.com".into()),
      timeout_ms: 900_000,
      credentials: Arc::new(Mutex::new(credentials)),
    }
  }

  pub async fn get_credentials(&self) -> Option<BlprntCredentials> {
    let guard = self.credentials.lock().await;
    Some(guard.clone())
  }

  pub async fn update_credentials(&self, credentials: BlprntCredentials) {
    let mut guard = self.credentials.lock().await;
    *guard = credentials;
  }
}

#[async_trait::async_trait]
impl ProviderAdapterTrait for AnthropicProvider {
  fn provider(&self) -> Provider {
    Provider::Anthropic
  }

  async fn stream_conversation(
    &self,
    req: ChatRequest,
    tools: Option<Arc<ToolSchemaRegistry>>,
    provider_dispatch: Arc<ProviderDispatch>,
    cancel_token: CancellationToken,
  ) {
    let result = self.stream_inner(req, tools, provider_dispatch.clone(), cancel_token).await;
    if let Err(e) = result {
      let _ = provider_dispatch.send(ProviderEvent::Error(e));
    }
  }

  async fn one_off_request(
    &self,
    prompt: String,
    system: String,
    model: Option<String>,
    cancel_token: CancellationToken,
  ) -> Result<ChatBasic> {
    let headers = self.with_headers(vec![Beta::ClaudeCode]).await?;

    let mut body = AnthropicMapping::build_body_basic(prompt, system, model);
    self.maybe_mutate_body(&mut body).await;

    let url = format!("{}/v1/messages?beta=true", self.base_url.trim_end_matches('/'));

    // Skip the middleware for basic chat requests
    let resp = tokio::select! {
      resp = self.http.client
        .post(url.clone())
        .headers(headers)
        .timeout(std::time::Duration::from_millis(self.timeout_ms))
        .json(&body)
        .send() => resp,
      _ = cancel_token.cancelled() => return Err(ProviderError::UserCancelled.into()),
    }
    .map_err(|e| ProviderError::ExternalNetwork {
      context: "anthropic::one_off_request".into(),
      message: e.to_string(),
    })?;

    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await.unwrap_or_default();
      let error = Self::parse_error_response(status, &text, None);
      return Err(error.into());
    }

    let mut chat_basic = ChatBasic::default();
    let resp_json = resp.json::<ChatBasicResponse>().await.map_err(|e| ProviderError::DecodingFailed {
      context: "anthropic::one_off_request".into(),
      message: e.to_string(),
    })?;
    chat_basic.messages = resp_json.content.iter().map(|c| c.text.clone()).collect();

    Ok(chat_basic)
  }

  async fn count_tokens(&self, req: ChatRequest, tools: &ToolSchemaRegistry) -> Result<u32> {
    let model = get_oauth_slug(&req.llm_model);
    let headers = self.create_chat_stream_headers(model.clone()).await?;

    let tools_json = AnthropicTools::to_tools_json(Some(Arc::new(tools.clone())));
    let mut body = AnthropicMapping::build_body(req, false, tools_json).await?;

    self.maybe_mutate_body(&mut body).await;

    body.max_tokens = None;
    body.stream = None;
    body.output_config = None;
    body.temperature = None;
    body.context_management = None;

    let url = format!("{}/v1/messages/count_tokens", self.base_url.trim_end_matches('/'));

    let resp = self
      .http
      .client
      .post(url.clone())
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .json(&body)
      .send()
      .await
      .map_err(|e| ProviderError::ExternalNetwork {
        context: "anthropic::count_tokens".into(),
        message: e.to_string(),
      })?;

    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await.unwrap_or_default();
      let error = Self::parse_error_response(status, &text, Some(&model));
      return Err(error.into());
    }

    let resp_json = resp.json::<CountTokensResponse>().await.map_err(|e| ProviderError::DecodingFailed {
      context: "anthropic::count_tokens".into(),
      message: e.to_string(),
    })?;

    Ok(resp_json.input_tokens)
  }
}

impl AnthropicProvider {
  /// Parse an error response from Anthropic into a ProviderError
  fn parse_error_response(status: http::StatusCode, body: &str, model: Option<&str>) -> ProviderError {
    if let Some(error_response) = AnthropicErrorResponse::parse(body) {
      return error_response.to_provider_error(status, model);
    }

    ProviderError::LlmUnknownError {
      context: format!("anthropic::{}", status.as_u16()),
      message: if body.is_empty() {
        format!("Request failed with status {}", status)
      } else if body.len() > 300 {
        format!("{}...", &body[..297])
      } else {
        body.to_string()
      },
    }
  }

  /// Parse streaming errors (EventSourceError) into ProviderError
  async fn parse_stream_error(error: EventSourceError, model: Option<&str>) -> ProviderError {
    match error {
      EventSourceError::InvalidContentType(_, response) => {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Self::parse_error_response(status, &body, model)
      }
      EventSourceError::InvalidStatusCode(status, _, body) => Self::parse_error_response(status, &body, model),
      EventSourceError::Transport(reqwest_error) => {
        if reqwest_error.is_timeout() {
          ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
        } else if reqwest_error.is_connect() {
          ProviderError::ExternalNetwork {
            context: "anthropic::stream".into(),
            message: "Failed to connect to Anthropic. Please check your internet connection.".into(),
          }
        } else {
          ProviderError::ExternalNetwork { context: "anthropic::stream".into(), message: reqwest_error.to_string() }
        }
      }
      EventSourceError::StreamEnded => ProviderError::StreamEnded,
      _ => error.into(),
    }
  }

  async fn stream_inner(
    &self,
    req: ChatRequest,
    tools: Option<Arc<ToolSchemaRegistry>>,
    provider_dispatch: Arc<ProviderDispatch>,
    cancel_token: CancellationToken,
  ) -> std::result::Result<(), ProviderError> {
    let model = get_oauth_slug(&req.llm_model);
    let headers =
      self.create_chat_stream_headers(model.clone()).await.map_err(|e| ProviderError::Internal(e.to_string()))?;

    let tools_json = AnthropicTools::to_tools_json(tools.clone());
    let mut body = AnthropicMapping::build_body(req, true, tools_json).await?;
    self.maybe_mutate_body(&mut body).await;

    let url = format!("{}/v1/messages?beta=true", self.base_url.trim_end_matches('/'));

    let request_builder = self
      .http
      .client
      .post(url.clone())
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .body(serde_json::to_string(&body).unwrap_or_default());

    tokio::spawn(async move {
      let result = tokio::select! {
        result = Self::run_chat_stream(request_builder, provider_dispatch.clone(), &model) => result,
        _ = cancel_token.cancelled() => Err(ProviderError::UserCancelled),
      };

      if let Err(e) = result {
        let _ = provider_dispatch.send(ProviderEvent::Error(e));
      }
    });

    Ok(())
  }

  async fn create_chat_stream_headers(&self, model: String) -> Result<HeaderMap> {
    let mut beta_headers =
      vec![Beta::ClaudeCode, Beta::InterleavedThinking, Beta::FineGrainedToolStreaming, Beta::ContextManagement];
    if model.contains("opus") && model.contains("4.5") {
      beta_headers.push(Beta::Effort);
    }

    let mut headers = self.with_headers(beta_headers).await?;
    headers.insert(http::header::ACCEPT, HeaderValue::from_static("text/event-stream"));

    Ok(headers)
  }

  async fn run_chat_stream(
    request_builder: RequestBuilder,
    provider_dispatch: Arc<ProviderDispatch>,
    model: &str,
  ) -> std::result::Result<(), ProviderError> {
    let mut content_blocks: HashMap<u32, ParsedContentBlock> = HashMap::new();

    let mut event_source = request_builder.eventsource().map_err(|e| ProviderError::Internal(e.to_string()))?;
    event_source.set_retry_policy(Box::new(Never));

    'stream: while let Some(event) = event_source.next().await {
      let item = match event {
        Ok(Event::Open) => continue,
        Ok(Event::Message(message)) => serde_json::from_str(&message.data).map_err(|e| {
          ProviderError::DecodingFailed { context: "anthropic::run_chat_stream".into(), message: e.to_string() }
        })?,
        Err(e) => return Err(Self::parse_stream_error(e, Some(model)).await),
      };

      let _ = provider_dispatch.send(ProviderEvent::Ping);

      let should_stop = AnthropicSseParser::parse(item, provider_dispatch.clone(), &mut content_blocks).await?;
      if should_stop.unwrap_or(false) {
        break 'stream;
      }
    }

    Ok(())
  }

  async fn maybe_mutate_body(&self, body: &mut MessageRequestBody) {
    let credentials = {
      let guard = self.credentials.lock().await.clone();
      guard.clone()
    };

    if let BlprntCredentials::OauthToken(OauthToken::Anthropic(_)) = &credentials {
      body.system.insert(
        0,
        SystemRequestBody {
          kind: "text".into(),
          text: "You are Claude Code, Anthropic's official CLI for Claude.".to_string(),
        },
      );
    }
  }

  async fn with_headers(&self, betas: Vec<Beta>) -> std::result::Result<HeaderMap, ProviderError> {
    let mut headers = HeaderMap::new();
    let mut betas = betas;

    let credentials = {
      let guard = self.credentials.lock().await;
      guard.clone()
    };

    match &credentials {
      BlprntCredentials::ApiKey(api_key) => {
        let api_key = HeaderValue::from_str(api_key).map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
        headers.insert("x-api-key", api_key);
      }
      BlprntCredentials::OauthToken(OauthToken::Anthropic(AnthropicOauthToken { access_token, .. })) => {
        let auth_header = HeaderValue::from_str(&format!("Bearer {access_token}"))
          .map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert(USER_AGENT, HeaderValue::from_static("claude-cli/2.0.8 (external, cli)"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("*"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("br, gzip, deflate"));
        headers.insert("x-app", HeaderValue::from_static("cli"));
        headers.insert("X-Stainless-Helper-Method", HeaderValue::from_static("stream"));
        headers.insert("X-Stainless-Lang", HeaderValue::from_static("js"));
        headers.insert("X-Stainless-Version", HeaderValue::from_static("0.69.0"));
        headers.insert("X-Stainless-Runtime-Version", HeaderValue::from_static("v18.19.1"));
        headers.insert("X-Stainless-Timeout", HeaderValue::from_static("600"));
        headers.insert("X-Stainless-Retry-Count", HeaderValue::from_static("0"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));

        betas.push(Beta::Oauth);
      }
      _ => unreachable!(),
    }

    let betas = betas.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(",");
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let betas_value = HeaderValue::from_str(&betas).map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
    headers.insert("anthropic-beta", betas_value);
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert("anthropic-dangerous-direct-browser-access", HeaderValue::from_static("true"));

    Ok(headers)
  }
}

pub struct AnthropicSseParser;

impl AnthropicSseParser {
  pub async fn parse(
    value: SseItem,
    provider_dispatch: Arc<ProviderDispatch>,
    content_blocks: &mut HashMap<u32, ParsedContentBlock>,
  ) -> std::result::Result<Option<bool>, ProviderError> {
    // #[cfg(all(debug_assertions, not(feature = "testing")))]
    // {
    //   let event_type =
    //     value.as_object().and_then(|obj| obj.get("type")).and_then(|value| value.as_str()).unwrap_or("unknown");
    //   tracing::debug!("Anthropic event type: {}", event_type);

    //   let str_value = serde_json::to_string_pretty(&value).unwrap_or_default();
    //   tracing::debug!("Anthropic event: {}", str_value);
    // }

    let event = serde_json::from_value::<AnthropicStreamEvent>(value.clone())
      .map_err(|e| ProviderError::DecodingFailed { context: "anthropic::parse".into(), message: e.to_string() });

    if let Err(e) = event {
      let event_type =
        value.as_object().and_then(|obj| obj.get("type")).and_then(|value| value.as_str()).unwrap_or("unknown");
      tracing::error!("Anthropic event decoding error: {:?}", e);
      tracing::error!("Event Type: {}", event_type);
      tracing::debug!("Event: {}", serde_json::to_string_pretty(&value).unwrap_or_default());
      return Ok(None);
    }

    match event {
      // Control
      Ok(AnthropicStreamEvent::MessageStart { .. }) => {
        provider_dispatch
          .send(ProviderEvent::Start("null".to_string()))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      Ok(AnthropicStreamEvent::MessageStop) => {
        provider_dispatch
          .send(ProviderEvent::Stop("null".to_string()))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(Some(true))
      }

      // Response
      Ok(AnthropicStreamEvent::ContentBlockStart {
        index,
        content_block: AnthropicContentBlock::Text { text, .. },
        ..
      }) => {
        let id = Uuid::new_v7().to_string();
        let mut content_block = ParsedContentBlock::new_text(index, id.clone(), None);
        content_block.append_text(text.clone());
        content_blocks.insert(index, content_block);

        provider_dispatch
          .send(ProviderEvent::ResponseStarted { rel_id: id })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      Ok(AnthropicStreamEvent::ContentBlockDelta { index, delta: Delta::TextDelta { text }, .. })
        if content_blocks.get(&index).is_some() =>
      {
        let Some(content_block) = content_blocks.get_mut(&index) else {
          return Err(ProviderError::Internal(format!("missing content block for index={index}")));
        };
        let id = content_block.get_id();
        content_block.append_text(text.clone());

        provider_dispatch
          .send(ProviderEvent::ResponseDelta { rel_id: id, delta: text })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      // Reasoning
      Ok(AnthropicStreamEvent::ContentBlockStart {
        index,
        content_block: AnthropicContentBlock::Thinking { thinking, signature, .. },
        ..
      }) => {
        let id = Uuid::new_v7().to_string();
        let mut content_block = ParsedContentBlock::new_thinking(index, id.clone(), signature.clone());
        content_block.append_thinking(thinking.clone());
        content_blocks.insert(index, content_block);

        provider_dispatch
          .send(ProviderEvent::ReasoningStarted { rel_id: id })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      Ok(AnthropicStreamEvent::ContentBlockDelta { index, delta: Delta::ThinkingDelta { thinking }, .. })
        if content_blocks.get(&index).is_some() =>
      {
        let Some(content_block) = content_blocks.get_mut(&index) else {
          return Err(ProviderError::Internal(format!("missing content block for index={index}")));
        };
        let id = content_block.get_id();
        content_block.append_thinking(thinking.clone());

        provider_dispatch
          .send(ProviderEvent::ReasoningDelta { rel_id: id, delta: thinking })
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }
      Ok(AnthropicStreamEvent::ContentBlockDelta { index, delta: Delta::SignatureDelta { signature }, .. })
        if content_blocks.get(&index).is_some() =>
      {
        let Some(content_block) = content_blocks.get_mut(&index) else {
          return Err(ProviderError::Internal(format!("missing content block for index={index}")));
        };
        content_block.append_signature(signature.clone());

        Ok(None)
      }

      // Tool use
      Ok(AnthropicStreamEvent::ContentBlockStart {
        index,
        content_block: AnthropicContentBlock::ToolUse { id, name, .. },
        ..
      }) => {
        let content_block = ParsedContentBlock::new_tool_use(index, id.clone(), name.clone(), None);
        content_blocks.insert(index, content_block);

        Ok(None)
      }
      Ok(AnthropicStreamEvent::ContentBlockDelta { index, delta: Delta::InputJsonDelta { partial_json }, .. })
        if content_blocks.get(&index).is_some() =>
      {
        let Some(content_block) = content_blocks.get_mut(&index) else {
          return Err(ProviderError::Internal(format!("missing content block for index={index}")));
        };
        content_block.append_input(partial_json.clone());

        Ok(None)
      }

      // Agnostic Content Stop
      Ok(AnthropicStreamEvent::ContentBlockStop { index }) if content_blocks.get(&index).is_some() => {
        let Some(content_block) = content_blocks.remove(&index) else {
          return Err(ProviderError::Internal(format!("missing content block for index={index}")));
        };

        match content_block.content_block {
          ContentBlock::Text { id, text, .. } => {
            provider_dispatch
              .send(ProviderEvent::Response { rel_id: id.clone(), content: text, signature: None })
              .map_err(|e| ProviderError::Internal(e.to_string()))?;
            provider_dispatch
              .send(ProviderEvent::ResponseDone { rel_id: id })
              .map_err(|e| ProviderError::Internal(e.to_string()))?;
          }
          ContentBlock::ToolUse { id, name, input, .. } => {
            provider_dispatch
              .send(ProviderEvent::ToolCall {
                tool_id:     ToolId::try_from(name).map_err(|e| ProviderError::LlmMistake {
                  context: "anthropic::parse".into(),
                  message: e.to_string(),
                })?,
                tool_use_id: id,
                args:        input,
                signature:   None,
              })
              .map_err(|e| ProviderError::Internal(e.to_string()))?;
          }
          ContentBlock::Thinking { id, thinking, signature } => {
            provider_dispatch
              .send(ProviderEvent::Reasoning { rel_id: id.clone(), reasoning: thinking, signature })
              .map_err(|e| ProviderError::Internal(e.to_string()))?;
            provider_dispatch
              .send(ProviderEvent::ReasoningDone { rel_id: id })
              .map_err(|e| ProviderError::Internal(e.to_string()))?;
          }
          ContentBlock::Status { .. } => {}
        }

        Ok(None)
      }

      Ok(AnthropicStreamEvent::Error { error }) => {
        Err(ProviderError::LlmError { context: "anthropic::parse".into(), message: error.message })
      }

      Ok(AnthropicStreamEvent::MessageDelta { usage }) => {
        tracing::debug!("Anthropic usage: {:?}", usage);
        let total_input_tokens = usage.input_tokens + usage.cache_creation_input_tokens.unwrap_or(0);

        provider_dispatch
          .send(ProviderEvent::TokenUsage(total_input_tokens))
          .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(None)
      }

      _ => Ok(None),
    }
  }
}
