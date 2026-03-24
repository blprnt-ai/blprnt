use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use events::ADAPTER_EVENTS;
use events::AdapterEvent;
use events::COORDINATOR_EVENTS;
use events::CoordinatorEvent;
use persistence::prelude::ContentsVisibility;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::IssueRecord;
use persistence::prelude::IssueRepository;
use persistence::prelude::ProjectRecord;
use persistence::prelude::ProjectRepository;
use persistence::prelude::ProviderRecord;
use persistence::prelude::ProviderRepository;
use persistence::prelude::RunId;
use persistence::prelude::RunRepository;
use persistence::prelude::RunStatus;
use persistence::prelude::RunTrigger;
use persistence::prelude::TurnId;
use persistence::prelude::TurnModel;
use persistence::prelude::TurnRepository;
use persistence::prelude::TurnStepContent;
use persistence::prelude::TurnStepRole;
use persistence::prelude::TurnStepStatus;
use persistence::prelude::TurnStepText;
use persistence::prelude::TurnStepThinking;
use persistence::prelude::TurnStepToolResult;
use persistence::prelude::TurnStepToolUse;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use shared::agent::AgentKind;
use shared::agent::BlprntCredentials;
use shared::agent::OauthToken;
use shared::agent::Provider;
use shared::agent::ToolId;
use shared::errors::ProviderError;
use shared::sandbox_flags::SandboxFlags;
use shared::tools::ToolUseResponse;
use shared::tools::config::ToolRuntimeConfig;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tools::Tool;
use tools::Tools;
use tools::tool_use::ToolUseContext;

use crate::prompt::PromptAssemblyInput;

#[derive(Clone, Debug)]
pub struct ProviderSelection {
  pub provider:    Provider,
  pub model_slug:  String,
  pub base_url:    Option<String>,
  pub credentials: Option<BlprntCredentials>,
}

#[derive(Clone, Debug, Default)]
pub struct ProviderRequest {
  pub system_prompt: String,
  pub messages:      Vec<ProviderMessage>,
}

impl ProviderRequest {
  fn tool_result_count(&self) -> usize {
    self
      .messages
      .iter()
      .flat_map(|message| message.contents.iter())
      .filter(|content| matches!(content, ProviderMessageContent::ToolResult(_)))
      .count()
  }
}

#[derive(Clone, Debug)]
pub struct ProviderMessage {
  pub role:     TurnStepRole,
  pub contents: Vec<ProviderMessageContent>,
}

#[derive(Clone, Debug)]
pub enum ProviderMessageContent {
  Text(String),
  ToolUse(ToolCallSpec),
  ToolResult(ToolCallResult),
}

#[derive(Clone, Debug)]
pub struct ToolCallSpec {
  pub tool_use_id: String,
  pub tool_id:     ToolId,
  pub input:       serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ToolCallResult {
  pub tool_use_id: String,
  pub tool_id:     ToolId,
  pub result:      ToolUseResponse,
}

#[derive(Clone, Debug, Default)]
pub struct ProviderReply {
  pub thinking:   Option<String>,
  pub text:       Option<String>,
  pub tool_calls: Vec<ToolCallSpec>,
}

#[async_trait::async_trait]
pub trait ProviderClient: Send + Sync {
  async fn next_reply(&self, request: ProviderRequest, cancel_token: CancellationToken) -> Result<ProviderReply>;
}

pub trait ProviderFactory: Send + Sync {
  fn client(&self, selection: &ProviderSelection) -> Arc<dyn ProviderClient>;
}

#[derive(Clone, Default)]
pub struct DefaultProviderFactory;

impl ProviderFactory for DefaultProviderFactory {
  fn client(&self, selection: &ProviderSelection) -> Arc<dyn ProviderClient> {
    match selection.provider {
      Provider::Mock => Arc::new(MockProviderClient),
      Provider::OpenAi => Arc::new(OpenAiCompatibleProviderClient::new(
        Provider::OpenAi,
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
      Provider::OpenRouter => Arc::new(OpenAiCompatibleProviderClient::new(
        Provider::OpenRouter,
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
      Provider::Anthropic => Arc::new(AnthropicProviderClient::new(
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
      unsupported => Arc::new(UnsupportedProviderClient { provider: unsupported }),
    }
  }
}

#[derive(Clone)]
pub struct ScriptedProviderFactory {
  replies: Arc<Mutex<VecDeque<ScriptedProviderReply>>>,
}

impl Default for ScriptedProviderFactory {
  fn default() -> Self {
    Self::new(Vec::new())
  }
}

impl ScriptedProviderFactory {
  pub fn new(replies: Vec<ScriptedProviderReply>) -> Self {
    Self { replies: Arc::new(Mutex::new(replies.into())) }
  }
}

impl ProviderFactory for ScriptedProviderFactory {
  fn client(&self, _selection: &ProviderSelection) -> Arc<dyn ProviderClient> {
    Arc::new(ScriptedProviderClient { replies: self.replies.clone() })
  }
}

#[derive(Clone, Debug)]
pub struct ScriptedProviderReply(ProviderReply);

impl ScriptedProviderReply {
  pub fn tool_call(thinking: String, tool_call: ToolCallSpec) -> Self {
    Self(ProviderReply { thinking: Some(thinking), text: None, tool_calls: vec![tool_call] })
  }

  pub fn final_text(text: String) -> Self {
    Self(ProviderReply { thinking: None, text: Some(text), tool_calls: Vec::new() })
  }
}

struct ScriptedProviderClient {
  replies: Arc<Mutex<VecDeque<ScriptedProviderReply>>>,
}

#[async_trait::async_trait]
impl ProviderClient for ScriptedProviderClient {
  async fn next_reply(&self, _request: ProviderRequest, _cancel_token: CancellationToken) -> Result<ProviderReply> {
    let mut replies = self.replies.lock().await;
    let reply = replies.pop_front().context("scripted provider exhausted before the run completed")?;
    Ok(reply.0)
  }
}

struct MockProviderClient;

#[async_trait::async_trait]
impl ProviderClient for MockProviderClient {
  async fn next_reply(&self, request: ProviderRequest, _cancel_token: CancellationToken) -> Result<ProviderReply> {
    let text = if request.tool_result_count() == 0 {
      "Mock provider completed the run.".to_string()
    } else {
      format!("Mock provider completed the run after {} tool result(s).", request.tool_result_count())
    };

    Ok(ProviderReply {
      thinking:   Some("Mock provider executing the active runtime path.".to_string()),
      text:       Some(text),
      tool_calls: Vec::new(),
    })
  }
}

struct UnsupportedProviderClient {
  provider: Provider,
}

#[async_trait::async_trait]
impl ProviderClient for UnsupportedProviderClient {
  async fn next_reply(&self, _request: ProviderRequest, _cancel_token: CancellationToken) -> Result<ProviderReply> {
    Err(anyhow::anyhow!("provider {} is not supported by the active runtime yet", self.provider))
  }
}

struct OpenAiCompatibleProviderClient {
  provider:    Provider,
  model_slug:  String,
  base_url:    String,
  credentials: Option<BlprntCredentials>,
  http:        reqwest::Client,
}

impl OpenAiCompatibleProviderClient {
  fn new(
    provider: Provider,
    model_slug: String,
    base_url: Option<String>,
    credentials: Option<BlprntCredentials>,
  ) -> Self {
    let default_base_url = match provider {
      Provider::OpenRouter => "https://openrouter.ai/api/v1",
      _ => "https://api.openai.com/v1",
    };

    Self {
      provider,
      model_slug,
      base_url: base_url.unwrap_or_else(|| default_base_url.to_string()),
      credentials,
      http: reqwest::Client::new(),
    }
  }
}

#[async_trait::async_trait]
impl ProviderClient for OpenAiCompatibleProviderClient {
  async fn next_reply(&self, request: ProviderRequest, cancel_token: CancellationToken) -> Result<ProviderReply> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, bearer_auth_header(self.credentials.as_ref(), self.provider)?);

    let body = build_openai_request_body(&request, &self.model_slug);
    let url = format!("{}/responses", self.base_url.trim_end_matches('/'));

    let response = self
      .http
      .post(url)
      .headers(headers)
      .json(&body)
      .send()
      .await
      .map_err(|error| normalize_provider_request_error(error, &cancel_token))?;

    parse_openai_response(response, &cancel_token).await
  }
}

struct AnthropicProviderClient {
  model_slug:  String,
  base_url:    String,
  credentials: Option<BlprntCredentials>,
  http:        reqwest::Client,
}

impl AnthropicProviderClient {
  fn new(model_slug: String, base_url: Option<String>, credentials: Option<BlprntCredentials>) -> Self {
    Self {
      model_slug,
      base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
      credentials,
      http: reqwest::Client::new(),
    }
  }
}

#[async_trait::async_trait]
impl ProviderClient for AnthropicProviderClient {
  async fn next_reply(&self, request: ProviderRequest, cancel_token: CancellationToken) -> Result<ProviderReply> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    apply_anthropic_auth_headers(&mut headers, self.credentials.as_ref())?;

    let body = build_anthropic_request_body(&request, &self.model_slug);
    let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));

    let response = self
      .http
      .post(url)
      .headers(headers)
      .json(&body)
      .send()
      .await
      .map_err(|error| normalize_provider_request_error(error, &cancel_token))?;

    parse_anthropic_response(response, &cancel_token).await
  }
}

pub struct AdapterRuntime {
  provider_factory: Arc<dyn ProviderFactory>,
  api_url:          String,
}

#[derive(Debug)]
struct RunCancelled;

impl std::fmt::Display for RunCancelled {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "run cancelled")
  }
}

impl std::error::Error for RunCancelled {}

impl AdapterRuntime {
  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      provider_factory: Arc::new(DefaultProviderFactory),
      api_url:          env::var("BLPRNT_API_URL").unwrap_or_else(|_| "http://127.0.0.1:9171".to_string()),
    })
  }

  pub fn new_for_tests(provider_factory: impl ProviderFactory + 'static, api_url: String) -> Self {
    Self { provider_factory: Arc::new(provider_factory), api_url }
  }

  pub async fn listen(self: Arc<Self>) {
    let mut rx = COORDINATOR_EVENTS.subscribe();

    loop {
      match rx.recv().await {
        Ok(CoordinatorEvent::StartRun { run_id, cancel_token, tx }) => {
          let runtime = self.clone();
          tokio::spawn(async move {
            let result = runtime.execute_run(run_id, cancel_token).await;
            if let Some(sender) = tx.lock().await.take() {
              let _ = sender.send(result);
            }
          });
        }
        Err(error) => {
          tracing::error!(?error, "failed to receive coordinator event");
        }
      }
    }
  }

  pub async fn execute_run(&self, run_id: RunId, cancel_token: CancellationToken) -> Result<()> {
    let run = RunRepository::get(run_id.clone()).await.context("failed to load run")?;
    let employee = persistence::prelude::EmployeeRepository::get(run.employee_id.clone())
      .await
      .context("failed to load employee")?;
    let mut turn_id = None;

    let result = async {
      let issue = load_issue_context(&run.trigger).await?;
      let project = load_project_context(issue.as_ref()).await?;
      let provider = load_provider_selection(&employee).await?;

      ensure_supported_provider(&provider)?;
      emit_adapter_event(AdapterEvent::RunStarted { run_id: run.id.clone() });

      let agent_home = agent_home_for_employee(&employee.id)?;
      let project_home = project_home_for_run(project.as_ref());
      let prompt = PromptAssemblyInput {
        agent_home:       agent_home.clone(),
        project_home:     project_home.clone(),
        employee_id:      employee.id.uuid().to_string(),
        api_url:          self.api_url.clone(),
        operating_system: env::consts::OS.to_string(),
        heartbeat_prompt: employee
          .runtime_config
          .as_ref()
          .map(|config| config.heartbeat_prompt.clone())
          .unwrap_or_default(),
        trigger:          run.trigger.clone(),
        issue_id:         issue.as_ref().map(|record| record.id.uuid()),
      }
      .build();

      let turn = TurnRepository::create(TurnModel { run_id: run.id.clone(), ..Default::default() })
        .await
        .context("failed to create turn")?;
      turn_id = Some(turn.id.clone());

      self
        .execute_turn(
          &run.id,
          &turn.id,
          &provider,
          prompt,
          ToolRuntimeConfig {
            agent_home:   Some(agent_home.clone()),
            project_home: project_home.clone(),
            employee_id:  Some(employee.id.uuid().to_string()),
            api_url:      Some(self.api_url.clone()),
          },
          project,
          cancel_token,
        )
        .await
    }
    .await;

    match result {
      Ok(()) => {
        RunRepository::update(run.id.clone(), RunStatus::Completed).await.context("failed to complete run")?;
        emit_adapter_event(AdapterEvent::RunCompleted { run_id: run.id });
        Ok(())
      }
      Err(error) => {
        if is_run_cancelled(&error) {
          RunRepository::update(run.id.clone(), RunStatus::Cancelled).await.context("failed to mark run cancelled")?;
        } else {
          let failure = error.to_string();
          if let Some(turn_id) = turn_id {
            let _ = mark_last_assistant_step(turn_id, TurnStepStatus::Failed).await;
          }
          RunRepository::update(run.id.clone(), RunStatus::Failed(failure.clone()))
            .await
            .context("failed to mark run failed")?;
          emit_adapter_event(AdapterEvent::RunFailed { run_id: run.id, error: failure });
        }
        Err(error)
      }
    }
  }

  async fn execute_turn(
    &self,
    run_id: &RunId,
    turn_id: &TurnId,
    provider: &ProviderSelection,
    prompt: crate::prompt::BuiltPrompt,
    runtime_config: ToolRuntimeConfig,
    project: Option<ProjectRecord>,
    cancel_token: CancellationToken,
  ) -> Result<()> {
    append_text(turn_id.clone(), TurnStepRole::User, prompt.user_prompt.clone()).await?;
    mark_last_step(turn_id.clone(), TurnStepStatus::Completed).await?;

    let client = self.provider_factory.client(provider);
    let system_prompt = prompt.system_prompt;

    loop {
      if cancel_token.is_cancelled() {
        return Err(run_cancelled_error());
      }

      let request = build_provider_request(turn_id.clone(), system_prompt.clone()).await?;
      let reply = client
        .next_reply(request, cancel_token.child_token())
        .await
        .map_err(|error| normalize_runtime_error(error, &cancel_token))?;

      if cancel_token.is_cancelled() {
        return Err(run_cancelled_error());
      }

      let thinking = reply.thinking.clone().filter(|thinking| !thinking.trim().is_empty());
      let text = reply.text.clone().filter(|text| !text.trim().is_empty());
      let has_thinking = thinking.is_some();
      let has_text = text.is_some();

      if thinking.is_none() && text.is_none() && reply.tool_calls.is_empty() {
        return Err(anyhow::anyhow!("provider returned an empty provider reply"));
      }

      if let Some(thinking) = thinking {
        append_thinking(turn_id.clone(), thinking.clone()).await?;
        emit_adapter_event(AdapterEvent::Thinking { run_id: run_id.clone(), thinking });
      }

      if let Some(text) = text {
        append_text(turn_id.clone(), TurnStepRole::Assistant, text.clone()).await?;
        emit_adapter_event(AdapterEvent::Response { run_id: run_id.clone(), response: text });
      }

      for tool_call in &reply.tool_calls {
        append_tool_use(turn_id.clone(), tool_call.clone()).await?;
      }

      if has_thinking || has_text || !reply.tool_calls.is_empty() {
        mark_last_step(turn_id.clone(), TurnStepStatus::Completed).await?;
      }

      if reply.tool_calls.is_empty() {
        return Ok(());
      }

      for tool_call in reply.tool_calls {
        if cancel_token.is_cancelled() {
          return Err(run_cancelled_error());
        }

        let tool_result = self.execute_tool_call(&tool_call, runtime_config.clone(), project.as_ref()).await?;
        append_tool_result(turn_id.clone(), &tool_call, tool_result.clone()).await?;
        mark_last_step(turn_id.clone(), TurnStepStatus::Completed).await?;
        emit_adapter_event(AdapterEvent::ToolDone {
          run_id:  run_id.clone(),
          tool_id: tool_call.tool_id.clone(),
          result:  tool_result.clone(),
        });
      }
    }
  }

  async fn execute_tool_call(
    &self,
    tool_call: &ToolCallSpec,
    runtime_config: ToolRuntimeConfig,
    project: Option<&ProjectRecord>,
  ) -> Result<ToolUseResponse> {
    let args = serde_json::to_string(&tool_call.input).context("failed to serialize tool input")?;
    let tool = Tools::try_from((&tool_call.tool_id, args.as_str())).context("failed to build tool")?;

    let working_directories = tool_working_directories(&runtime_config, project);
    let context = ToolUseContext::new(
      project.map(|record| record.id.clone()),
      AgentKind::Crew,
      working_directories,
      runtime_config,
      Vec::new(),
      SandboxFlags::default(),
      "runtime".to_string(),
      false,
    );

    Ok(tool.maybe_invoke(context).await)
  }
}

fn emit_adapter_event(event: AdapterEvent) {
  if let Err(error) = ADAPTER_EVENTS.emit(event) {
    tracing::warn!(?error, "failed to emit adapter event");
  }
}

fn run_cancelled_error() -> anyhow::Error {
  anyhow::Error::new(RunCancelled)
}

fn is_run_cancelled(error: &anyhow::Error) -> bool {
  error.downcast_ref::<RunCancelled>().is_some()
    || matches!(error.downcast_ref::<ProviderError>(), Some(ProviderError::UserCancelled | ProviderError::Canceled))
}

fn normalize_runtime_error(error: anyhow::Error, cancel_token: &CancellationToken) -> anyhow::Error {
  if cancel_token.is_cancelled() || is_run_cancelled(&error) { run_cancelled_error() } else { error }
}

async fn build_provider_request(turn_id: TurnId, system_prompt: String) -> Result<ProviderRequest> {
  let turn = TurnRepository::get(turn_id).await.context("failed to load turn for provider request")?;
  let mut messages = Vec::new();

  for step in turn.steps {
    let mut contents = Vec::new();
    for content in step.contents.contents {
      match content {
        TurnStepContent::Text(text) => {
          if !text.text.trim().is_empty() {
            contents.push(ProviderMessageContent::Text(text.text));
          }
        }
        TurnStepContent::ToolUse(tool_use) => contents.push(ProviderMessageContent::ToolUse(ToolCallSpec {
          tool_use_id: tool_use.tool_use_id,
          tool_id:     tool_use.tool_id,
          input:       tool_use.input,
        })),
        TurnStepContent::ToolResult(tool_result) => {
          contents.push(ProviderMessageContent::ToolResult(ToolCallResult {
            tool_use_id: tool_result.tool_use_id,
            tool_id:     tool_result.tool_id,
            result:      tool_result.content,
          }));
        }
        TurnStepContent::Thinking(_) | TurnStepContent::Image64(_) => {}
      }
    }

    if !contents.is_empty() {
      messages.push(ProviderMessage { role: step.contents.role, contents });
    }
  }

  Ok(ProviderRequest { system_prompt, messages })
}

async fn load_provider_credentials(record: &ProviderRecord) -> Result<BlprntCredentials> {
  let Some(secret) = vault::get_stronghold_secret(vault::Vault::Key, record.id.uuid()).await else {
    return Err(anyhow::anyhow!("provider {} is missing credentials", record.provider));
  };

  if looks_like_json(&secret) {
    serde_json::from_str(&secret).with_context(|| format!("failed to decode {} credentials", record.provider))
  } else {
    Ok(BlprntCredentials::ApiKey(secret))
  }
}

fn looks_like_json(value: &str) -> bool {
  let trimmed = value.trim();
  (trimmed.starts_with('{') && trimmed.ends_with('}')) || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn runtime_tool_specs() -> Vec<tools::ToolSpec> {
  Tools::schema()
    .into_iter()
    .filter(|spec| {
      matches!(tool_id_from_spec(spec), Some(ToolId::FilesRead) | Some(ToolId::ApplyPatch) | Some(ToolId::Shell))
    })
    .collect()
}

fn tool_id_from_spec(spec: &tools::ToolSpec) -> Option<ToolId> {
  tool_name_from_spec(spec).and_then(|name| ToolId::try_from(name).ok())
}

fn runtime_tool_id_from_name(name: String) -> Result<ToolId> {
  match ToolId::try_from(name.clone())? {
    ToolId::FilesRead | ToolId::ApplyPatch | ToolId::Shell => ToolId::try_from(name).map_err(Into::into),
    other => Err(anyhow::anyhow!("tool {} is not executable in the adapters runtime", other)),
  }
}

fn tool_name_from_spec(spec: &tools::ToolSpec) -> Option<String> {
  spec.name.as_str().map(ToOwned::to_owned).or_else(|| serde_json::from_value(spec.name.clone()).ok())
}

fn value_to_string(value: &serde_json::Value) -> String {
  value.as_str().map(ToOwned::to_owned).unwrap_or_else(|| value.to_string())
}

fn build_openai_request_body(request: &ProviderRequest, model_slug: &str) -> serde_json::Value {
  let tools = runtime_tool_specs()
    .into_iter()
    .filter_map(|spec| {
      let name = tool_name_from_spec(&spec)?;
      Some(serde_json::json!({
        "type": "function",
        "name": name,
        "description": value_to_string(&spec.description),
        "parameters": spec.params,
      }))
    })
    .collect::<Vec<_>>();

  serde_json::json!({
    "model": model_slug,
    "instructions": request.system_prompt,
    "input": openai_input_items(request),
    "parallel_tool_calls": true,
    "stream": false,
    "tools": tools,
  })
}

fn openai_input_items(request: &ProviderRequest) -> Vec<serde_json::Value> {
  let mut input = Vec::new();

  for message in &request.messages {
    let mut content = Vec::new();
    for item in &message.contents {
      match item {
        ProviderMessageContent::Text(text) => {
          let kind = if message.role == TurnStepRole::Assistant { "output_text" } else { "input_text" };
          content.push(serde_json::json!({ "type": kind, "text": text }));
        }
        ProviderMessageContent::ToolUse(tool_call) => {
          if !content.is_empty() {
            input.push(openai_message_input(message.role.clone(), std::mem::take(&mut content)));
          }

          input.push(serde_json::json!({
            "type": "function_call",
            "call_id": tool_call.tool_use_id,
            "name": tool_call.tool_id.to_string(),
            "arguments": serialize_json_value(&tool_call.input),
          }));
        }
        ProviderMessageContent::ToolResult(tool_result) => {
          if !content.is_empty() {
            input.push(openai_message_input(message.role.clone(), std::mem::take(&mut content)));
          }

          input.push(serde_json::json!({
            "type": "function_call_output",
            "call_id": tool_result.tool_use_id,
            "output": tool_result.result.clone().into_llm_payload().to_string(),
            "status": "completed",
          }));
        }
      }
    }

    if !content.is_empty() {
      input.push(openai_message_input(message.role.clone(), content));
    }
  }

  input
}

fn openai_message_input(role: TurnStepRole, content: Vec<serde_json::Value>) -> serde_json::Value {
  let role = match role {
    TurnStepRole::User => "user",
    TurnStepRole::Assistant => "assistant",
  };

  serde_json::json!({
    "type": "message",
    "role": role,
    "content": content,
  })
}

async fn parse_openai_response(response: reqwest::Response, cancel_token: &CancellationToken) -> Result<ProviderReply> {
  let status = response.status();
  let body = response.text().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?;

  if !status.is_success() {
    return Err(anyhow::anyhow!("openai-compatible provider request failed: {} {}", status, body));
  }

  let value =
    serde_json::from_str::<serde_json::Value>(&body).context("failed to decode openai-compatible response")?;
  let mut reply = ProviderReply::default();
  let mut text_chunks = Vec::new();
  let mut thinking_chunks = Vec::new();

  for item in value.get("output").and_then(serde_json::Value::as_array).into_iter().flatten() {
    match item.get("type").and_then(serde_json::Value::as_str) {
      Some("message") => {
        for content in item.get("content").and_then(serde_json::Value::as_array).into_iter().flatten() {
          if content.get("type").and_then(serde_json::Value::as_str) == Some("output_text")
            && let Some(text) = content.get("text").and_then(serde_json::Value::as_str)
            && !text.is_empty()
          {
            text_chunks.push(text.to_string());
          }
        }
      }
      Some("function_call") => {
        let tool_use_id = item
          .get("call_id")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("openai-compatible tool call is missing call_id")?;
        let tool_name = item
          .get("name")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("openai-compatible tool call is missing name")?;
        let arguments = item
          .get("arguments")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .unwrap_or_else(|| "{}".to_string());

        reply.tool_calls.push(ToolCallSpec {
          tool_use_id,
          tool_id: runtime_tool_id_from_name(tool_name)?,
          input: serde_json::from_str(&arguments).context("failed to decode openai-compatible tool arguments")?,
        });
      }
      Some("reasoning") => {
        if let Some(summary) = item.get("summary").and_then(serde_json::Value::as_array) {
          for part in summary {
            if let Some(text) = part.get("text").and_then(serde_json::Value::as_str)
              && !text.is_empty()
            {
              thinking_chunks.push(text.to_string());
            }
          }
        }
      }
      _ => {}
    }
  }

  if text_chunks.is_empty()
    && let Some(text) = value.get("output_text").and_then(serde_json::Value::as_str)
    && !text.is_empty()
  {
    text_chunks.push(text.to_string());
  }

  if !thinking_chunks.is_empty() {
    reply.thinking = Some(thinking_chunks.join("\n"));
  }

  if !text_chunks.is_empty() {
    reply.text = Some(text_chunks.join("\n"));
  }

  Ok(reply)
}

fn build_anthropic_request_body(request: &ProviderRequest, model_slug: &str) -> serde_json::Value {
  let tools = runtime_tool_specs()
    .into_iter()
    .filter_map(|spec| {
      let name = tool_name_from_spec(&spec)?;
      Some(serde_json::json!({
        "name": name,
        "description": value_to_string(&spec.description),
        "input_schema": spec.params,
      }))
    })
    .collect::<Vec<_>>();

  let messages = request.messages.iter().filter_map(|message| anthropic_message_input(message)).collect::<Vec<_>>();

  serde_json::json!({
    "model": model_slug,
    "system": [{ "type": "text", "text": request.system_prompt }],
    "messages": messages,
    "tools": tools,
    "max_tokens": 4096,
    "stream": false,
  })
}

fn anthropic_message_input(message: &ProviderMessage) -> Option<serde_json::Value> {
  let role = match message.role {
    TurnStepRole::User => "user",
    TurnStepRole::Assistant => "assistant",
  };

  let content = message
    .contents
    .iter()
    .map(|item| match item {
      ProviderMessageContent::Text(text) => serde_json::json!({ "type": "text", "text": text }),
      ProviderMessageContent::ToolUse(tool_call) => serde_json::json!({
        "type": "tool_use",
        "id": tool_call.tool_use_id,
        "name": tool_call.tool_id.to_string(),
        "input": tool_call.input,
      }),
      ProviderMessageContent::ToolResult(tool_result) => serde_json::json!({
        "type": "tool_result",
        "tool_use_id": tool_result.tool_use_id,
        "content": tool_result.result.clone().into_llm_payload().to_string(),
      }),
    })
    .collect::<Vec<_>>();

  (!content.is_empty()).then_some(serde_json::json!({ "role": role, "content": content }))
}

fn apply_anthropic_auth_headers(headers: &mut HeaderMap, credentials: Option<&BlprntCredentials>) -> Result<()> {
  match credentials.context("anthropic provider is missing credentials")? {
    BlprntCredentials::ApiKey(api_key) => {
      headers.insert("x-api-key", HeaderValue::from_str(api_key).context("invalid anthropic api key header")?);
    }
    BlprntCredentials::OauthToken(OauthToken::Anthropic(token)) => {
      headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token.access_token)).context("invalid anthropic oauth header")?,
      );
    }
    other => {
      return Err(anyhow::anyhow!("unsupported anthropic credential shape: {:?}", other));
    }
  }

  Ok(())
}

async fn parse_anthropic_response(
  response: reqwest::Response,
  cancel_token: &CancellationToken,
) -> Result<ProviderReply> {
  let status = response.status();
  let body = response.text().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?;

  if !status.is_success() {
    return Err(anyhow::anyhow!("anthropic provider request failed: {} {}", status, body));
  }

  let value = serde_json::from_str::<serde_json::Value>(&body).context("failed to decode anthropic response")?;
  let mut reply = ProviderReply::default();
  let mut text_chunks = Vec::new();
  let mut thinking_chunks = Vec::new();

  for item in value.get("content").and_then(serde_json::Value::as_array).into_iter().flatten() {
    match item.get("type").and_then(serde_json::Value::as_str) {
      Some("text") => {
        if let Some(text) = item.get("text").and_then(serde_json::Value::as_str)
          && !text.is_empty()
        {
          text_chunks.push(text.to_string());
        }
      }
      Some("thinking") => {
        if let Some(thinking) = item.get("thinking").and_then(serde_json::Value::as_str)
          && !thinking.is_empty()
        {
          thinking_chunks.push(thinking.to_string());
        }
      }
      Some("tool_use") => {
        let tool_use_id = item
          .get("id")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("anthropic tool use is missing id")?;
        let tool_name = item
          .get("name")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("anthropic tool use is missing name")?;

        reply.tool_calls.push(ToolCallSpec {
          tool_use_id,
          tool_id: runtime_tool_id_from_name(tool_name)?,
          input: item.get("input").cloned().unwrap_or_else(|| serde_json::json!({})),
        });
      }
      _ => {}
    }
  }

  if !thinking_chunks.is_empty() {
    reply.thinking = Some(thinking_chunks.join("\n"));
  }

  if !text_chunks.is_empty() {
    reply.text = Some(text_chunks.join("\n"));
  }

  Ok(reply)
}

fn serialize_json_value(value: &serde_json::Value) -> String {
  serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

fn bearer_auth_header(credentials: Option<&BlprntCredentials>, provider: Provider) -> Result<HeaderValue> {
  let token = match credentials.context(format!("{provider} provider is missing credentials"))? {
    BlprntCredentials::ApiKey(api_key) => api_key.clone(),
    BlprntCredentials::OauthToken(OauthToken::OpenAi(token)) if provider == Provider::OpenAi => {
      token.access_token.clone()
    }
    BlprntCredentials::OauthToken(OauthToken::OpenAi(token)) if provider == Provider::OpenRouter => {
      token.access_token.clone()
    }
    other => return Err(anyhow::anyhow!("unsupported {provider} credential shape: {:?}", other)),
  };

  HeaderValue::from_str(&format!("Bearer {token}")).context("invalid authorization header")
}

fn normalize_provider_request_error(error: reqwest::Error, cancel_token: &CancellationToken) -> anyhow::Error {
  if cancel_token.is_cancelled() {
    run_cancelled_error()
  } else if error.is_timeout() {
    ProviderError::timeout().into()
  } else if error.is_request() || error.is_connect() || error.is_body() || error.is_decode() {
    ProviderError::ExternalNetwork { context: "adapters::provider_request".to_string(), message: error.to_string() }
      .into()
  } else {
    anyhow::Error::new(error)
  }
}

async fn load_provider_selection(employee: &EmployeeRecord) -> Result<ProviderSelection> {
  let provider_config = employee.provider_config.clone().unwrap_or_default();
  if provider_config.provider == Provider::Mock
    || !matches!(provider_config.provider, Provider::OpenAi | Provider::OpenRouter | Provider::Anthropic)
  {
    return Ok(ProviderSelection {
      provider:    provider_config.provider,
      model_slug:  provider_config.slug,
      base_url:    None,
      credentials: None,
    });
  }

  let record = ProviderRepository::get_by_provider(provider_config.provider)
    .await
    .with_context(|| format!("provider {} is not configured", provider_config.provider))?;
  let credentials = load_provider_credentials(&record).await?;

  Ok(ProviderSelection {
    provider:    provider_config.provider,
    model_slug:  provider_config.slug,
    base_url:    record.base_url,
    credentials: Some(credentials),
  })
}

fn ensure_supported_provider(selection: &ProviderSelection) -> Result<()> {
  match selection.provider {
    Provider::Mock | Provider::OpenAi | Provider::OpenRouter | Provider::Anthropic => Ok(()),
    _ => Err(anyhow::anyhow!("provider {} is not supported by the active runtime yet", selection.provider)),
  }
}

async fn load_issue_context(trigger: &RunTrigger) -> Result<Option<IssueRecord>> {
  match trigger {
    RunTrigger::IssueAssignment { issue_id } => {
      Ok(Some(IssueRepository::get(issue_id.clone()).await.context("failed to load trigger issue")?))
    }
    RunTrigger::Manual | RunTrigger::Timer => Ok(None),
  }
}

async fn load_project_context(issue: Option<&IssueRecord>) -> Result<Option<ProjectRecord>> {
  let Some(project_id) = issue.and_then(|record| record.project.clone()) else {
    return Ok(None);
  };

  Ok(Some(ProjectRepository::get(project_id).await.context("failed to load trigger project")?))
}

fn agent_home_for_employee(employee_id: &EmployeeId) -> Result<PathBuf> {
  Ok(
    env::current_dir()
      .context("failed to resolve current directory")?
      .join("memories")
      .join("employees")
      .join(employee_id.uuid().to_string()),
  )
}

fn project_home_for_run(project: Option<&ProjectRecord>) -> Option<PathBuf> {
  project.and_then(|record| record.working_directories.first().map(PathBuf::from))
}

fn tool_working_directories(runtime_config: &ToolRuntimeConfig, project: Option<&ProjectRecord>) -> Vec<PathBuf> {
  let mut directories = Vec::new();

  if let Some(project) = project {
    directories.extend(project.working_directories.iter().map(PathBuf::from));
  } else if let Some(project_home) = &runtime_config.project_home {
    directories.push(project_home.clone());
  }

  if let Some(agent_home) = &runtime_config.agent_home
    && !directories.iter().any(|path| path == agent_home)
  {
    directories.push(agent_home.clone());
  }

  if directories.is_empty() {
    directories.push(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
  }

  directories
}

async fn append_text(turn_id: TurnId, role: TurnStepRole, text: String) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    role,
    TurnStepContent::Text(TurnStepText { text, signature: None, visibility: ContentsVisibility::Full }),
  )
  .await
  .context("failed to append text")?;
  Ok(())
}

async fn append_thinking(turn_id: TurnId, thinking: String) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepRole::Assistant,
    TurnStepContent::Thinking(TurnStepThinking {
      thinking,
      signature: String::new(),
      visibility: ContentsVisibility::Assistant,
    }),
  )
  .await
  .context("failed to append thinking")?;
  Ok(())
}

async fn append_tool_use(turn_id: TurnId, tool_call: ToolCallSpec) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepRole::Assistant,
    TurnStepContent::ToolUse(TurnStepToolUse {
      tool_use_id: tool_call.tool_use_id,
      tool_id:     tool_call.tool_id,
      input:       tool_call.input,
      visibility:  ContentsVisibility::Full,
    }),
  )
  .await
  .context("failed to append tool use")?;
  Ok(())
}

async fn append_tool_result(turn_id: TurnId, tool_call: &ToolCallSpec, result: ToolUseResponse) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepRole::User,
    TurnStepContent::ToolResult(TurnStepToolResult {
      tool_use_id: tool_call.tool_use_id.clone(),
      tool_id:     tool_call.tool_id.clone(),
      content:     result,
      visibility:  ContentsVisibility::Full,
    }),
  )
  .await
  .context("failed to append tool result")?;
  Ok(())
}

async fn mark_last_step(turn_id: TurnId, status: TurnStepStatus) -> Result<()> {
  let mut turn = TurnRepository::get(turn_id.clone()).await.context("failed to reload turn")?;
  let Some(step) = turn.steps.last_mut() else {
    return Ok(());
  };

  step.status = status;
  step.completed_at.get_or_insert(Utc::now());

  TurnRepository::update(turn_id, turn.steps).await.context("failed to update step status")?;
  Ok(())
}

async fn mark_last_assistant_step(turn_id: TurnId, status: TurnStepStatus) -> Result<()> {
  let turn = TurnRepository::get(turn_id.clone()).await.context("failed to reload turn")?;
  if !matches!(turn.steps.last().map(|step| &step.contents.role), Some(TurnStepRole::Assistant)) {
    return Ok(());
  }

  mark_last_step(turn_id, status).await
}
