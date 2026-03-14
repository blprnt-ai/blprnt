use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use common::errors::ProviderError;
use common::event_source::Event;
use common::event_source::EventSourceError;
use common::event_source::RequestBuilderExt;
use common::event_source::retry::Never;
use common::provider_dispatch::ProviderDispatch;
use common::provider_dispatch::ProviderEvent;
use common::shared::prelude::*;
use futures_util::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use http::header::ACCEPT;
use http::header::AUTHORIZATION;
use http::header::CONTENT_TYPE;
use reqwest_middleware::RequestBuilder;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::responses::mapping::OpenAiResponsesMapping;
use super::tools::OpenAiTools;
use crate::providers::openai::client::OpenAiHttp;
use crate::providers::openai::error::CodexError;
use crate::providers::openai::responses::request::ResponsesChatRequestBody;
use crate::providers::openai::responses::response::MessageContentPart;
use crate::providers::openai::responses::response::OpenAiCountTokensResponse;
use crate::providers::openai::responses::response::OutputItem;
use crate::providers::openai::responses::response::ResponseCompleted;
use crate::providers::openai::responses::response::ResponseSummary;
use crate::tools::registry::ToolSchemaRegistry;
use crate::traits::ProviderAdapterTrait;
use crate::types::ChatBasic;

#[derive(Debug)]
pub struct OpenAiProvider {
  http:          Arc<OpenAiHttp>,
  base_url:      String,
  timeout_ms:    u64,
  provider_type: OpenAiProviderType,
}

#[derive(Debug)]
pub enum OpenAiProviderType {
  OpenAi(BlprntCredentials),
  Blprnt(String),
}

impl OpenAiProvider {
  pub fn new(provider_type: OpenAiProviderType, timeout_ms: Option<u64>, base_url_override: Option<String>) -> Self {
    let (http, base_url) = match &provider_type {
      OpenAiProviderType::OpenAi(BlprntCredentials::ApiKey(api_key)) => {
        (OpenAiHttp::get_client(api_key), "https://api.openai.com/v1".to_string())
      }
      OpenAiProviderType::OpenAi(BlprntCredentials::OauthToken(OauthToken::OpenAi(OpenAiOauthToken {
        access_token,
        ..
      }))) => (OpenAiHttp::get_client(access_token), "https://chatgpt.com/backend-api/codex".to_string()),
      OpenAiProviderType::Blprnt(base_url) => (OpenAiHttp::get_client(base_url), base_url.clone()),
      _ => unreachable!(),
    };

    let base_url = base_url_override.unwrap_or(base_url);

    let timeout_ms = timeout_ms.unwrap_or(900_000);

    Self { http, base_url: base_url.to_string(), timeout_ms, provider_type }
  }
}

#[async_trait::async_trait]
impl ProviderAdapterTrait for OpenAiProvider {
  fn provider(&self) -> Provider {
    Provider::OpenAi
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
    let is_openrouter = self.provider() == Provider::OpenRouter;
    let basic_model = model.unwrap_or_else(|| "gpt-5-nano".to_string());
    let headers = self.build_headers(false)?;

    let mut body = OpenAiResponsesMapping::build_body_basic(basic_model.to_string(), true, prompt, system);
    body.stream = if is_openrouter { Some(true) } else { Some(false) };
    body.store = Some(false);
    let url = format!("{}/responses", self.base_url.trim_end_matches('/'));

    if is_openrouter {
      self.openrouter_request(&url, headers, body, cancel_token).await
    } else {
      self.openai_request(&url, headers, body).await
    }
  }

  async fn count_tokens(&self, req: ChatRequest, tools: &ToolSchemaRegistry) -> Result<u32> {
    let headers = self.build_headers(false)?;

    let tools_json = OpenAiTools::to_tools_json(Some(Arc::new(tools.clone())));
    let mut body = OpenAiResponsesMapping::build_body(Provider::OpenAi, req, false, Some(tools_json)).await?;
    body.stream = None;
    body.store = None;

    let url = format!("{}/responses/input_tokens", self.base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let resp = client
      .post(url.clone())
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .json(&body)
      .send()
      .await
      .map_err(|e| ProviderError::ExternalNetwork {
        context: "openai::provider::count_tokens".into(),
        message: e.to_string(),
      })?;

    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await.unwrap_or_default();

      return Err(
        ProviderError::LlmUnknownError { context: "openai::provider::count_tokens".into(), message: text }.into(),
      );
    }

    let resp_json = resp.json::<OpenAiCountTokensResponse>().await.map_err(|e| ProviderError::DecodingFailed {
      context: "openai::provider::count_tokens".into(),
      message: e.to_string(),
    })?;

    Ok(resp_json.input_tokens)
  }
}

impl OpenAiProvider {
  async fn openrouter_request(
    &self,
    url: &str,
    headers: HeaderMap,
    body: ResponsesChatRequestBody,
    cancel_token: CancellationToken,
  ) -> Result<ChatBasic> {
    // Skip the middleware for basic chat requests
    let resp = tokio::select! {
      resp = self.http.client
      .post(url)
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .json(&body)
      .send() => resp,
      _ = cancel_token.cancelled() => return Err(ProviderError::UserCancelled.into()),
    }
    .map_err(|e| ProviderError::ExternalNetwork {
      context: "openai::provider::one_off_request".into(),
      message: e.to_string(),
    })?;

    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await.unwrap_or_default();
      let error = serde_json::from_str::<CodexError>(&text);
      tracing::error!("error: {:#?}", error);

      match error {
        Ok(error) => return Err(error.to_provider_error().into()),
        Err(e) => {
          return Err(
            ProviderError::LlmUnknownError {
              context: "openai::provider::one_off_request".into(),
              message: format!("An unknown error occurred: {}", e),
            }
            .into(),
          );
        }
      }
    }

    let response = resp.json::<ResponseSummary>().await.map_err(|e| ProviderError::DecodingFailed {
      context: "openai::provider::one_off_request".into(),
      message: e.to_string(),
    })?;

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

    Ok(ChatBasic { messages })
  }

  async fn openai_request(
    &self,
    url: &str,
    mut headers: HeaderMap,
    mut body: ResponsesChatRequestBody,
  ) -> Result<ChatBasic> {
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    body.stream = Some(true);

    let request = self
      .http
      .client
      .post(url)
      .headers(headers)
      .timeout(std::time::Duration::from_millis(self.timeout_ms))
      .json(&body);

    let response = request.send().await?;
    let response_text = response
      .text()
      .await?
      .lines()
      .filter_map(|line| if !line.is_empty() { Some(line.to_string()) } else { None })
      .collect::<Vec<_>>()
      .last()
      .cloned()
      .unwrap_or_default()
      .strip_prefix("data: ")
      .unwrap_or_default()
      .to_string();

    let response = serde_json::from_str::<Value>(&response_text)?;
    let response = response.get("response").unwrap_or_default();
    let response = serde_json::from_value::<ResponseCompleted>(response.clone())?;

    Ok(ChatBasic {
      messages: response
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
        .collect(),
    })
  }

  async fn stream_inner(
    &self,
    req: ChatRequest,
    tools: Option<Arc<ToolSchemaRegistry>>,
    provider_dispatch: Arc<ProviderDispatch>,
    cancel_token: CancellationToken,
  ) -> std::result::Result<(), ProviderError> {
    let request = self.build_request(req, tools, true).await?;

    tokio::spawn(async move {
      let result = tokio::select! {
        result = Self::run_chat_stream(request, provider_dispatch.clone()) => result,
        _ = cancel_token.cancelled() => Err(ProviderError::UserCancelled),
      };

      if let Err(e) = result {
        let _ = provider_dispatch.send(ProviderEvent::Error(e));
      }
    });

    Ok(())
  }

  pub async fn run_chat_stream(
    request: RequestBuilder,
    provider_dispatch: Arc<ProviderDispatch>,
  ) -> std::result::Result<(), ProviderError> {
    let mut content_blocks = HashMap::new();

    let mut event_source = request.eventsource().map_err(|e| ProviderError::Internal(e.to_string()))?;
    event_source.set_retry_policy(Box::new(Never));

    'stream: while let Some(event) = event_source.next().await {
      let item = match event {
        Ok(Event::Open) => continue,
        Ok(Event::Message(message)) => serde_json::from_str(&message.data).map_err(|e| {
          ProviderError::DecodingFailed { context: "openai::provider::run_chat_stream".into(), message: e.to_string() }
        })?,
        Err(e) => return Err(Self::parse_stream_error(e).await),
      };

      // tracing::info!("item: {}", serde_json::to_string_pretty(&item).unwrap_or_default());

      let event_result = OpenAiResponsesMapping::stream_event_from_sse(
        Provider::OpenAi,
        &item,
        provider_dispatch.clone(),
        &mut content_blocks,
      )
      .inspect_err(|e| {
        tracing::warn!("item: {}", serde_json::to_string_pretty(&item).unwrap_or_default());
        tracing::error!("openai::provider::run_chat_stream: error: {:#?}", e);
      });

      match event_result {
        Ok(Some(true)) => break 'stream,
        Ok(Some(false)) | Ok(None) => continue,
        Err(e) => return Err(e),
      }
    }

    Ok(())
  }

  async fn parse_stream_error(error: EventSourceError) -> ProviderError {
    match error {
      EventSourceError::InvalidContentType(_, response) => {
        let text = response.text().await.unwrap_or_default();

        match serde_json::from_str::<CodexError>(&text) {
          Ok(error)
            if error.kind.is_some() || error.code.is_some() || error.param.is_some() || error.message.is_some() =>
          {
            error.to_provider_error()
          }
          Ok(_) => ProviderError::LlmUnknownError {
            context: "openai::provider::run_chat_stream:parse_1".into(),
            message: text,
          },
          Err(e) => ProviderError::LlmError {
            context: "openai::provider::run_chat_stream:parse_2".into(),
            message: format!("An unknown error occurred: {}", e),
          },
        }
      }
      EventSourceError::InvalidStatusCode(_, _, body) => match serde_json::from_str::<CodexError>(&body) {
        Ok(error)
          if error.kind.is_some() || error.code.is_some() || error.param.is_some() || error.message.is_some() =>
        {
          error.to_provider_error()
        }
        Ok(_) => {
          ProviderError::LlmUnknownError { context: "openai::provider::run_chat_stream:parse_3".into(), message: body }
        }
        Err(e) => ProviderError::LlmError {
          context: "openai::provider::run_chat_stream:parse_4".into(),
          message: format!("An unknown error occurred: {}", e),
        },
      },
      EventSourceError::Transport(reqwest_error) => {
        if reqwest_error.is_decode() {
          let text = reqwest_error.to_string();
          ProviderError::Decode(text)
        } else {
          ProviderError::ExternalNetwork {
            context: "openai::provider::run_chat_stream:parse_5".into(),
            message: reqwest_error.to_string(),
          }
        }
      }
      _ => error.into(),
    }
  }

  fn build_headers(&self, is_streaming: bool) -> std::result::Result<HeaderMap, ProviderError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    match &self.provider_type {
      OpenAiProviderType::OpenAi(BlprntCredentials::ApiKey(api_key))
      | OpenAiProviderType::OpenAi(BlprntCredentials::OauthToken(OauthToken::OpenAi(OpenAiOauthToken {
        access_token: api_key,
        ..
      }))) => {
        let auth_header = HeaderValue::from_str(&format!("Bearer {}", api_key))
          .map_err(|e| ProviderError::AuthHeaders(e.to_string()))?;
        headers.insert(AUTHORIZATION, auth_header);
        headers.insert("OpenAi-Beta", HeaderValue::from_static("responses=experimental"));
      }
      _ => {}
    }

    if is_streaming {
      headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    } else {
      headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    }

    Ok(headers)
  }

  async fn build_request(
    &self,
    req: ChatRequest,
    tools: Option<Arc<ToolSchemaRegistry>>,
    is_streaming: bool,
  ) -> std::result::Result<reqwest_middleware::RequestBuilder, ProviderError> {
    #[cfg(all(debug_assertions, feature = "debug-tracing"))]
    let session_id = req.session_id.clone();
    let url = format!("{}/responses", self.base_url.trim_end_matches('/'));

    let headers = self.build_headers(is_streaming)?;
    let tools_json = OpenAiTools::to_tools_json(tools);
    let body = OpenAiResponsesMapping::build_body(Provider::OpenAi, req, is_streaming, Some(tools_json)).await?;

    #[cfg(all(debug_assertions, feature = "debug-tracing"))]
    {
      let body_clone = body.clone();
      tokio::task::spawn(async move {
        let cwd = std::env::var("CARGO_MANIFEST_DIR")
          .unwrap_or_else(|_| std::env::current_dir().unwrap().to_string_lossy().to_string());
        let cwd = std::path::PathBuf::from(&cwd);
        let file_name = format!("body_{}.json", session_id.key());
        let file_path = cwd.join("..").join(".debug").canonicalize().unwrap();
        std::fs::create_dir_all(&file_path).unwrap();
        let file_path = file_path.join(file_name);

        let file_contents = std::fs::read_to_string(&file_path).unwrap_or_else(|_| "[]".to_string());
        let mut file_contents_vec = serde_json::from_str::<Vec<serde_json::Value>>(&file_contents).unwrap_or_default();
        file_contents_vec.push(serde_json::to_value(&body_clone).unwrap());

        let _ = std::fs::write(&file_path, serde_json::to_string_pretty(&file_contents_vec).unwrap_or_default());
      });
    }

    Ok(
      self
        .http
        .client
        .post(url)
        .headers(headers)
        .timeout(std::time::Duration::from_millis(self.timeout_ms))
        .json(&body),
    )
  }
}
