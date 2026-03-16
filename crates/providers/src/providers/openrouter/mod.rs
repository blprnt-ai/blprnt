mod error;
mod mapping;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use common::errors::ProviderError;
use common::event_source::Event;
use common::event_source::EventSourceError;
use common::event_source::RequestBuilderExt;
use common::event_source::retry::Never;
use common::models::ReasoningEffort;
use common::provider_dispatch::ProviderDispatch;
use common::provider_dispatch::ProviderEvent;
use common::shared::prelude::*;
use futures_util::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use http::header::ACCEPT;
use http::header::AUTHORIZATION;
use http::header::CONTENT_TYPE;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::RequestBuilder;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::providers::openai::responses::mapping::OpenAiResponsesMapping;
use crate::providers::openai::responses::request::ChatRequestBodyReasoning;
use crate::providers::openai::responses::request::ContentItem;
use crate::providers::openai::responses::request::InputItem;
use crate::providers::openai::responses::response::MessageContentPart;
use crate::providers::openai::responses::response::OutputItem;
use crate::providers::openai::responses::response::ResponseSummary;
use crate::providers::openai::tools::OpenAiTools;
use crate::providers::openrouter::error::OpenRouterErrorResponse;
use crate::providers::openrouter::mapping::OpenRouterMapping;
use crate::tools::registry::ToolSchemaRegistry;
use crate::traits::ProviderAdapterTrait;
use crate::types::ChatBasic;

#[derive(Debug)]
pub struct OpenRouterProvider {
  base_url:   String,
  timeout_ms: u64,
  api_key:    String,
}

impl OpenRouterProvider {
  pub fn new(api_key: String) -> Self {
    Self { base_url: "https://openrouter.ai/api/v1".into(), timeout_ms: 900_000, api_key }
  }
}

#[async_trait::async_trait]
impl ProviderAdapterTrait for OpenRouterProvider {
  fn provider(&self) -> Provider {
    Provider::OpenRouter
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
    let basic_model =
      LlmModel { slug: model.unwrap_or_else(|| "openai/gpt-oss-120b:free".to_string()), ..Default::default() };
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // TODO: Convert this to non-streaming for basic chat requests
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    let auth_header = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
      .map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
    headers.insert(AUTHORIZATION, auth_header);

    let mut body = OpenRouterMapping::build_body_basic(basic_model, prompt, system).with_debug(true);
    body.inner.stream = Some(false);
    body.inner.reasoning = Some(ChatRequestBodyReasoning::from(ReasoningEffort::Minimal));
    let instructions = body.inner.instructions.clone();
    body.inner.instructions = None;

    if let Some(instructions) = instructions {
      body.inner.input.insert(
        0,
        InputItem::Message {
          role:    "developer".to_string(),
          content: vec![ContentItem::InputText { text: instructions }],
          id:      None,
          status:  None,
        },
      );
    }

    let url = format!("{}/responses", self.base_url.trim_end_matches('/'));

    // Skip the middleware for basic chat requests
    let resp = tokio::select! {
      resp = reqwest::Client::new()
      .post(url)
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .json(&body)
      .send() => resp,
      _ = cancel_token.cancelled() => return Err(ProviderError::UserCancelled.into()),
    }
    .map_err(|e| ProviderError::ExternalNetwork {
      context: "openrouter::one_off_request".into(),
      message: e.to_string(),
    })?;

    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await.unwrap_or_default();
      let error =
        Self::parse_error_response(status, &text, None, Some(&serde_json::to_string(&body).unwrap_or_default()));
      return Err(error.into());
    }

    let response = resp.text().await.unwrap_or_default();

    match serde_json::from_str::<ResponseSummary>(&response) {
      Ok(response) => {
        tracing::debug!("ONE OFF REQUEST response: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
        let messages = response
          .output
          .iter()
          .filter_map(|o| {
            if let OutputItem::Message { content, .. } = o {
              if let Some(MessageContentPart::OutputText { text, .. }) = content.first() {
                Some(text.clone())
              } else {
                None
              }
            } else {
              None
            }
          })
          .collect();

        // tracing::debug!("ONE OFF REQUEST messages: {}", serde_json::to_string_pretty(&messages).unwrap_or_default());
        Ok(ChatBasic { messages })
      }
      Err(e) => {
        tracing::error!("ONE OFF REQUEST response: {}", e.to_string());
        let chat_basic = ChatBasic::default();

        Ok(chat_basic)
      }
    }
  }
}

impl OpenRouterProvider {
  /// Parse an error response from OpenRouter into a ProviderError
  fn parse_error_response(
    status: StatusCode,
    body: &str,
    model: Option<&str>,
    _request_body: Option<&str>,
  ) -> ProviderError {
    #[cfg(debug_assertions)]
    tracing::error!(
      "OpenRouter request body: {}",
      serde_json::to_string_pretty(&_request_body.as_ref()).unwrap_or_default()
    );
    tracing::error!("OpenRouter error response: {:?}, {:?}, {:?}", status, model, body);

    // Try to parse as OpenRouter error response
    if let Some(error_response) = OpenRouterErrorResponse::parse(body) {
      return error_response.to_provider_error(status, model);
    }

    // Fallback for unparseable responses
    ProviderError::LlmUnknownError {
      context: format!("openrouter::{}", status.as_u16()),
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
  async fn parse_stream_error(
    error: EventSourceError,
    model: Option<&str>,
    request_body: Option<&str>,
  ) -> ProviderError {
    match error {
      EventSourceError::InvalidContentType(_, response) => {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Self::parse_error_response(status, &body, model, request_body)
      }
      EventSourceError::InvalidStatusCode(status, _, body) => {
        Self::parse_error_response(status, &body, model, request_body)
      }
      EventSourceError::Transport(reqwest_error) => {
        if reqwest_error.is_timeout() {
          ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
        } else if reqwest_error.is_connect() {
          ProviderError::ExternalNetwork {
            context: "openrouter::stream".into(),
            message: "Failed to connect to OpenRouter. Please check your internet connection.".into(),
          }
        } else {
          ProviderError::ExternalNetwork { context: "openrouter::stream".into(), message: reqwest_error.to_string() }
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
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    let auth_header = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
      .map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
    headers.insert(AUTHORIZATION, auth_header);

    let mut headers_clone = headers.clone();
    headers_clone.remove(AUTHORIZATION);

    let tools_json = OpenAiTools::to_tools_json(tools.clone());
    let model = req.llm_model.slug.clone();

    let mut body = OpenRouterMapping::build_body(req, true, Some(tools_json)).await?;

    let instructions = body.inner.instructions.clone();
    body.inner.instructions = None;
    body.inner.model = model.clone();

    if let Some(instructions) = instructions {
      body.inner.input.insert(
        0,
        InputItem::Message {
          role:    "developer".to_string(),
          content: vec![ContentItem::InputText { text: instructions }],
          id:      None,
          status:  None,
        },
      );
    }

    let url = format!("{}/responses", self.base_url.trim_end_matches('/'));

    let client = ClientBuilder::new(reqwest::Client::new()).build();

    let request_builder =
      client.post(url.clone()).headers(headers).timeout(std::time::Duration::from_millis(self.timeout_ms)).json(&body);

    let request_body = serde_json::to_string(&body).unwrap_or_default();

    tokio::select! {
      result = Self::run_chat_stream(request_builder, provider_dispatch.clone(), &model, &request_body) => result,
      _ = cancel_token.cancelled() => Err(ProviderError::UserCancelled),
    }
  }

  async fn run_chat_stream(
    request_builder: RequestBuilder,
    provider_dispatch: Arc<ProviderDispatch>,
    model: &str,
    request_body: &str,
  ) -> std::result::Result<(), ProviderError> {
    let mut response_items = HashMap::new();

    let mut event_source = request_builder.eventsource().map_err(|e| ProviderError::Internal(e.to_string()))?;
    event_source.set_retry_policy(Box::new(Never));

    'stream: while let Some(event) = event_source.next().await {
      let item: Value = match event {
        Ok(Event::Message(message))
          if looks_like_json(&message.data) && serde_json::from_str::<serde_json::Value>(&message.data).is_ok() =>
        {
          serde_json::from_str(&message.data).expect("THIS SHOULD NEVER FAIL")
        }
        Err(e) => {
          if matches!(e, EventSourceError::StreamEnded) {
            break 'stream;
          }

          return Err(Self::parse_stream_error(e, Some(model), Some(request_body)).await);
        }
        _result => {
          // tracing::warn!("unhandled result: {:?}", result);

          continue;
        }
      };

      // if let Some(kind) = item.get("type") {
      //   tracing::info!("EVENT TYPE: {:?}", kind);
      // }

      // let _ = provider_dispatch.send(ProviderEvent::Ping);

      // tracing::info!("item: {}", serde_json::to_string_pretty(&item).unwrap_or_default());

      let should_stop = OpenAiResponsesMapping::stream_event_from_sse(
        Provider::OpenRouter,
        &item,
        provider_dispatch.clone(),
        &mut response_items,
      )
      .map_err(|e| {
        tracing::error!("openrouter::run_chat_stream: error: {:#?}", e);
        tracing::warn!("item: {}", serde_json::to_string_pretty(&item).unwrap_or_default());
        ProviderError::Internal(e.to_string())
      })?;

      if should_stop.unwrap_or(false) {
        break 'stream;
      }
    }

    Ok(())
  }
}

fn looks_like_json(s: &str) -> bool {
  let trimmed = s.trim();
  (trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}
