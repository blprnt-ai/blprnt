use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs;
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
use persistence::prelude::EmployeeRepository;
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
use persistence::prelude::TurnStepSide;
use persistence::prelude::TurnStepStatus;
use persistence::prelude::TurnStepText;
use persistence::prelude::TurnStepThinking;
use persistence::prelude::TurnStepToolResult;
use persistence::prelude::TurnStepToolUse;
use reqwest::header::ACCEPT;
use reqwest::header::ACCEPT_ENCODING;
use reqwest::header::ACCEPT_LANGUAGE;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::USER_AGENT;
use sandbox::RunSandbox;
use shared::agent::AgentKind;
use shared::agent::BlprntCredentials;
use shared::agent::OauthToken;
use shared::agent::Provider;
use shared::agent::ToolId;
use shared::errors::ProviderError;
use shared::tools::ToolUseResponse;
use shared::tools::config::ToolRuntimeConfig;
use tokio::sync::Mutex;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tools::Tool;
use tools::Tools;
use tools::tool_use::ToolUseContext;

use crate::prompt::InjectedSkillPrompt;
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

#[derive(Clone, Debug)]
pub enum ProviderStreamEvent {
  TextDelta { id: String, delta: String },
  TextDone { id: String, full_text: Option<String> },
  ThinkingDelta { id: String, delta: String },
  ThinkingDone { id: String, full_thinking: Option<String>, signature: Option<String> },
  ToolCall(ToolCallSpec),
}

#[async_trait::async_trait]
pub trait ProviderClient: Send + Sync {
  async fn next_reply(&self, request: ProviderRequest, cancel_token: CancellationToken) -> Result<ProviderReply>;

  async fn stream_reply(
    &self,
    request: ProviderRequest,
    cancel_token: CancellationToken,
    tx: mpsc::Sender<ProviderStreamEvent>,
  ) -> Result<()> {
    let reply = self.next_reply(request, cancel_token).await?;

    if let Some(thinking) = reply.thinking.filter(|thinking| !thinking.trim().is_empty()) {
      tx.send(ProviderStreamEvent::ThinkingDelta { id: "thinking-0".to_string(), delta: thinking.clone() })
        .await
        .context("failed to send synthetic thinking delta")?;
      tx.send(ProviderStreamEvent::ThinkingDone {
        id:            "thinking-0".to_string(),
        full_thinking: Some(thinking),
        signature:     None,
      })
      .await
      .context("failed to send synthetic thinking completion")?;
    }

    if let Some(text) = reply.text.filter(|text| !text.trim().is_empty()) {
      tx.send(ProviderStreamEvent::TextDelta { id: "text-0".to_string(), delta: text.clone() })
        .await
        .context("failed to send synthetic text delta")?;
      tx.send(ProviderStreamEvent::TextDone { id: "text-0".to_string(), full_text: Some(text) })
        .await
        .context("failed to send synthetic text completion")?;
    }

    for tool_call in reply.tool_calls {
      tx.send(ProviderStreamEvent::ToolCall(tool_call)).await.context("failed to send synthetic tool call")?;
    }

    Ok(())
  }
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
        Provider::Anthropic,
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
      Provider::ClaudeCode => Arc::new(AnthropicProviderClient::new(
        Provider::ClaudeCode,
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
      Provider::Codex => Arc::new(OpenAiCompatibleProviderClient::new(
        Provider::Codex,
        selection.model_slug.clone(),
        selection.base_url.clone(),
        selection.credentials.clone(),
      )),
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
      Provider::Codex if matches!(credentials, Some(BlprntCredentials::OauthToken(OauthToken::OpenAi(_)))) => {
        "https://chatgpt.com/backend-api/codex"
      }
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
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert("OpenAI-Beta", HeaderValue::from_static("responses=experimental"));
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

  async fn stream_reply(
    &self,
    request: ProviderRequest,
    cancel_token: CancellationToken,
    tx: mpsc::Sender<ProviderStreamEvent>,
  ) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    headers.insert("OpenAI-Beta", HeaderValue::from_static("responses=experimental"));
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

    stream_openai_response(self.provider, response, &cancel_token, tx).await
  }
}

struct AnthropicProviderClient {
  provider:    Provider,
  model_slug:  String,
  base_url:    String,
  credentials: Option<BlprntCredentials>,
  http:        reqwest::Client,
}

impl AnthropicProviderClient {
  fn new(
    provider: Provider,
    model_slug: String,
    base_url: Option<String>,
    credentials: Option<BlprntCredentials>,
  ) -> Self {
    Self {
      provider,
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
    apply_anthropic_auth_headers(&mut headers, self.provider, self.credentials.as_ref())?;

    let body = build_anthropic_request_body(&request, &self.model_slug, self.provider, self.credentials.as_ref());
    let url = anthropic_messages_url(self.base_url.as_str(), self.provider, self.credentials.as_ref());

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

  async fn stream_reply(
    &self,
    request: ProviderRequest,
    cancel_token: CancellationToken,
    tx: mpsc::Sender<ProviderStreamEvent>,
  ) -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    apply_anthropic_auth_headers(&mut headers, self.provider, self.credentials.as_ref())?;

    let mut body = build_anthropic_request_body(&request, &self.model_slug, self.provider, self.credentials.as_ref());
    body["stream"] = serde_json::json!(true);
    let url = anthropic_messages_url(self.base_url.as_str(), self.provider, self.credentials.as_ref());

    let response = self
      .http
      .post(url)
      .headers(headers)
      .json(&body)
      .send()
      .await
      .map_err(|error| normalize_provider_request_error(error, &cancel_token))?;

    stream_anthropic_response(response, &cancel_token, tx).await
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
      let available_skills = skills::list_skills()
        .context("failed to discover skills")?
        .into_iter()
        .map(|skill| persistence::prelude::EmployeeSkillRef {
          name: skill.name,
          path: skill.path.to_string_lossy().to_string(),
        })
        .collect::<Vec<_>>();
      let injected_skill_stack = employee
        .runtime_config
        .as_ref()
        .map(|config| build_injected_skill_stack(config.skill_stack.clone()))
        .transpose()?
        .unwrap_or_default();

      ensure_supported_provider(&provider)?;
      emit_adapter_event(AdapterEvent::RunStarted { run_id: run.id.clone() });

      let agent_home = agent_home_for_employee(&employee.id)?;
      let project_home = project_home_for_run(project.as_ref());
      let prompt = PromptAssemblyInput {
        agent_home: agent_home.clone(),
        project_home: project_home.clone(),
        project_workdirs: project
          .as_ref()
          .map(|record| record.working_directories.iter().map(PathBuf::from).collect())
          .unwrap_or_default(),
        employee_id: employee.id.uuid().to_string(),
        api_url: self.api_url.clone(),
        operating_system: env::consts::OS.to_string(),
        heartbeat_prompt: employee
          .runtime_config
          .as_ref()
          .map(|config| config.heartbeat_prompt.clone())
          .unwrap_or_default(),
        available_skills,
        injected_skill_stack,
        trigger: run.trigger.clone(),
        issue_id: issue.as_ref().map(|record| record.id.uuid()),
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
            project_id:   project.as_ref().map(|record| record.id.uuid().to_string()),
            run_id:       Some(run.id.uuid().to_string()),
            api_url:      Some(self.api_url.clone()),
          },
          employee.is_ceo(),
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
          emit_adapter_event(AdapterEvent::RunCancelled { run_id: run.id.clone() });
        } else {
          let failure = error.to_string();
          if let Some(turn_id) = turn_id {
            let _ = mark_last_step(turn_id, TurnStepStatus::Failed).await;
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
    is_ceo: bool,
    project: Option<ProjectRecord>,
    cancel_token: CancellationToken,
  ) -> Result<()> {
    append_request_text(turn_id.clone(), prompt.user_prompt.clone()).await?;

    let client = self.provider_factory.client(provider);
    let system_prompt = prompt.system_prompt;
    ensure_runtime_homes(&runtime_config)?;
    let working_directories = tool_working_directories(&runtime_config, project.as_ref());
    let run_sandbox = Arc::new(RunSandbox::new(&working_directories).await?);

    loop {
      if cancel_token.is_cancelled() {
        return Err(run_cancelled_error());
      }

      let request = build_provider_request(turn_id.clone(), system_prompt.clone()).await?;
      let (tx, mut rx) = mpsc::channel(128);
      let client = client.clone();
      let stream_cancel_token = cancel_token.child_token();
      let stream_task = tokio::spawn(async move { client.stream_reply(request, stream_cancel_token, tx).await });
      let mut stream_state = StreamingAssistantState::default();
      let mut streamed_anything = false;
      let mut tool_calls = Vec::new();

      while let Some(event) = rx.recv().await {
        streamed_anything = true;
        match event {
          ProviderStreamEvent::TextDelta { id, delta } => {
            append_stream_text(turn_id.clone(), &mut stream_state, &id, delta.clone()).await?;
            emit_adapter_event(AdapterEvent::ResponseDelta { run_id: run_id.clone(), delta });
          }
          ProviderStreamEvent::TextDone { id, full_text } => {
            if let Some(full_text) = full_text.filter(|text| !text.is_empty())
              && !stream_state.texts.contains_key(&id)
            {
              append_stream_text(turn_id.clone(), &mut stream_state, &id, full_text).await?;
            }

            if let Some(text) = stream_state.texts.get(&id).cloned().filter(|text| !text.trim().is_empty()) {
              emit_adapter_event(AdapterEvent::Response { run_id: run_id.clone(), response: text });
            }
          }
          ProviderStreamEvent::ThinkingDelta { id, delta } => {
            append_stream_thinking(turn_id.clone(), &mut stream_state, &id, delta.clone()).await?;
            emit_adapter_event(AdapterEvent::ThinkingDelta { run_id: run_id.clone(), delta });
          }
          ProviderStreamEvent::ThinkingDone { id, full_thinking, signature } => {
            if let Some(full_thinking) = full_thinking.filter(|thinking| !thinking.is_empty())
              && !stream_state.thinkings.contains_key(&id)
            {
              append_stream_thinking(turn_id.clone(), &mut stream_state, &id, full_thinking).await?;
            }

            if let Some(signature) = signature {
              set_last_thinking_signature(turn_id.clone(), &mut stream_state, &id, signature).await?;
            }

            if let Some(thinking) =
              stream_state.thinkings.get(&id).cloned().filter(|thinking| !thinking.trim().is_empty())
            {
              emit_adapter_event(AdapterEvent::Thinking { run_id: run_id.clone(), thinking });
            }
          }
          ProviderStreamEvent::ToolCall(tool_call) => {
            stream_state.current_fragment = None;
            append_tool_use(turn_id.clone(), tool_call.clone()).await?;
            tool_calls.push(tool_call);
          }
        }
      }

      stream_task
        .await
        .context("provider stream task join failed")?
        .map_err(|error| normalize_runtime_error(error, &cancel_token))?;

      if cancel_token.is_cancelled() {
        return Err(run_cancelled_error());
      }

      if !streamed_anything {
        return Err(anyhow::anyhow!("provider returned an empty provider reply"));
      }

      if !stream_state.texts.is_empty() || !stream_state.thinkings.is_empty() || !tool_calls.is_empty() {
        mark_last_step(turn_id.clone(), TurnStepStatus::Completed).await?;
      }

      if tool_calls.is_empty() {
        return Ok(());
      }

      for tool_call in tool_calls {
        if cancel_token.is_cancelled() {
          return Err(run_cancelled_error());
        }

        let tool_result = self
          .execute_tool_call(
            &tool_call,
            runtime_config.clone(),
            is_ceo,
            project.as_ref(),
            working_directories.clone(),
            run_sandbox.clone(),
          )
          .await?;
        append_tool_result(turn_id.clone(), &tool_call, tool_result.clone()).await?;
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
    is_ceo: bool,
    project: Option<&ProjectRecord>,
    working_directories: Vec<PathBuf>,
    sandbox: Arc<RunSandbox>,
  ) -> Result<ToolUseResponse> {
    let args = serde_json::to_string(&tool_call.input).context("failed to serialize tool input")?;
    let tool = Tools::try_from((&tool_call.tool_id, args.as_str())).context("failed to build tool")?;

    let (working_directories, sandbox) = if is_ceo && is_ceo_wide_write_tool(&tool_call.tool_id) {
      let working_directories = ceo_tool_working_directories(working_directories).await?;
      let sandbox = Arc::new(RunSandbox::new(&working_directories).await?);
      (working_directories, sandbox)
    } else {
      (working_directories, sandbox)
    };

    let context = ToolUseContext::new(
      project.map(|record| record.id.clone()),
      AgentKind::Crew,
      working_directories,
      runtime_config,
      Vec::new(),
      sandbox,
      false,
    );

    Ok(tool.maybe_invoke(context).await)
  }
}

fn build_injected_skill_stack(
  skill_stack: Option<Vec<persistence::prelude::EmployeeSkillRef>>,
) -> Result<Vec<InjectedSkillPrompt>> {
  skill_stack
    .unwrap_or_default()
    .iter()
    .map(|skill| {
      let metadata = skills::validate_skill_path(PathBuf::from(&skill.path).as_path(), Some(&skill.name))
        .with_context(|| format!("failed to validate employee skill {}", skill.name))?;
      let contents = fs::read_to_string(&metadata.path)
        .with_context(|| format!("failed to read skill {}", metadata.path.display()))?;
      Ok(InjectedSkillPrompt { name: metadata.name, path: metadata.path.to_string_lossy().to_string(), contents })
    })
    .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamFragmentKind {
  Text,
  Thinking,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StreamFragmentKey {
  kind: StreamFragmentKind,
  id:   String,
}

#[derive(Default)]
struct StreamingAssistantState {
  current_fragment: Option<StreamFragmentKey>,
  texts:            HashMap<String, String>,
  thinkings:        HashMap<String, String>,
}

fn emit_adapter_event(event: AdapterEvent) -> usize {
  match ADAPTER_EVENTS.emit(event) {
    Ok(subscriber_count) => subscriber_count,
    Err(error) => match error.downcast::<SendError<AdapterEvent>>() {
      Ok(send_error) => {
        let SendError(event) = send_error;
        tracing::debug!("dropping adapter event without subscribers: {:?}", event);
        0
      }
      Err(error) => {
        tracing::error!("failed to emit adapter event: {:?}", error);
        0
      }
    },
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
    let mut request_contents = Vec::new();
    for content in step.request.contents {
      match content {
        TurnStepContent::Text(text) => {
          if !text.text.trim().is_empty() {
            request_contents.push(ProviderMessageContent::Text(text.text));
          }
        }
        TurnStepContent::ToolUse(tool_use) => request_contents.push(ProviderMessageContent::ToolUse(ToolCallSpec {
          tool_use_id: tool_use.tool_use_id,
          tool_id:     tool_use.tool_id,
          input:       tool_use.input,
        })),
        TurnStepContent::ToolResult(tool_result) => {
          request_contents.push(ProviderMessageContent::ToolResult(ToolCallResult {
            tool_use_id: tool_result.tool_use_id,
            tool_id:     tool_result.tool_id,
            result:      tool_result.content,
          }));
        }
        TurnStepContent::Thinking(_) | TurnStepContent::Image64(_) => {}
      }
    }

    if !request_contents.is_empty() {
      messages.push(ProviderMessage { role: step.request.role, contents: request_contents });
    }

    let mut response_contents = Vec::new();
    for content in step.response.contents {
      match content {
        TurnStepContent::Text(text) => {
          if !text.text.trim().is_empty() {
            response_contents.push(ProviderMessageContent::Text(text.text));
          }
        }
        TurnStepContent::ToolUse(tool_use) => response_contents.push(ProviderMessageContent::ToolUse(ToolCallSpec {
          tool_use_id: tool_use.tool_use_id,
          tool_id:     tool_use.tool_id,
          input:       tool_use.input,
        })),
        TurnStepContent::ToolResult(tool_result) => {
          response_contents.push(ProviderMessageContent::ToolResult(ToolCallResult {
            tool_use_id: tool_result.tool_use_id,
            tool_id:     tool_result.tool_id,
            result:      tool_result.content,
          }));
        }
        TurnStepContent::Thinking(_) | TurnStepContent::Image64(_) => {}
      }
    }

    if !response_contents.is_empty() {
      messages.push(ProviderMessage { role: step.response.role, contents: response_contents });
    }
  }

  Ok(ProviderRequest { system_prompt, messages })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn emit_adapter_event_returns_zero_without_subscribers() {
    let subscriber_count = emit_adapter_event(AdapterEvent::RunStarted { run_id: persistence::Uuid::new_v4().into() });

    assert_eq!(subscriber_count, 0);
  }

  #[test]
  fn emit_adapter_event_returns_subscriber_count_when_listener_exists() {
    let _subscriber = ADAPTER_EVENTS.subscribe();

    let subscriber_count = emit_adapter_event(AdapterEvent::RunStarted { run_id: persistence::Uuid::new_v4().into() });

    assert_eq!(subscriber_count, 1);
  }
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
    "store": false,
    "stream": true,
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

  let value = decode_openai_response_value(&body)
    .inspect_err(|error| tracing::error!("failed to decode openai-compatible response - sse: {:?}", error))?;

  parse_openai_reply_value(value)
}

fn decode_openai_response_value(body: &str) -> Result<serde_json::Value> {
  parse_openai_sse_payload(body).or_else(|_| {
    serde_json::from_str::<serde_json::Value>(body).context("failed to decode openai-compatible response - sse")
  })
}

fn parse_openai_reply_value(value: serde_json::Value) -> Result<ProviderReply> {
  let value = value.get("response").cloned().unwrap_or(value);
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

async fn stream_openai_response(
  provider: Provider,
  mut response: reqwest::Response,
  cancel_token: &CancellationToken,
  tx: mpsc::Sender<ProviderStreamEvent>,
) -> Result<()> {
  let status = response.status();

  if !status.is_success() {
    let body = response.text().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?;
    return Err(anyhow::anyhow!("openai-compatible provider request failed: {} {}", status, body));
  }

  let mut raw_body = String::new();
  let mut frame_buffer = String::new();
  let mut saw_sse_frame = false;
  let mut openrouter_tool_calls: HashMap<u32, (String, String, String)> = HashMap::new();
  let mut reasoning_ids: HashMap<u32, String> = HashMap::new();

  while let Some(chunk) =
    response.chunk().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?
  {
    if cancel_token.is_cancelled() {
      return Err(run_cancelled_error());
    }

    let chunk = String::from_utf8_lossy(&chunk);
    raw_body.push_str(&chunk);
    frame_buffer.push_str(&chunk);

    while let Some(frame) = drain_sse_frame(&mut frame_buffer) {
      saw_sse_frame = true;
      if let Some(value) = parse_sse_data_frame(&frame)? {
        handle_openai_sse_event(provider, value, &tx, &mut openrouter_tool_calls, &mut reasoning_ids).await?;
      }
    }
  }

  if !saw_sse_frame && !raw_body.trim().is_empty() {
    let reply = parse_openai_reply_value(decode_openai_response_value(&raw_body)?)?;
    stream_provider_reply(reply, tx).await?;
  }

  Ok(())
}

fn build_anthropic_request_body(
  request: &ProviderRequest,
  model_slug: &str,
  provider: Provider,
  credentials: Option<&BlprntCredentials>,
) -> serde_json::Value {
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
  let mut system = vec![serde_json::json!({ "type": "text", "text": request.system_prompt })];

  if provider == Provider::ClaudeCode
    && matches!(credentials, Some(BlprntCredentials::OauthToken(OauthToken::Anthropic(_))))
  {
    system.insert(
      0,
      serde_json::json!({
        "type": "text",
        "text": "You are Claude Code, Anthropic's official CLI for Claude."
      }),
    );
  }

  serde_json::json!({
    "model": model_slug,
    "system": system,
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

fn apply_anthropic_auth_headers(
  headers: &mut HeaderMap,
  provider: Provider,
  credentials: Option<&BlprntCredentials>,
) -> Result<()> {
  let mut betas = vec!["claude-code-20250219".to_string()];
  match credentials.context("anthropic provider is missing credentials")? {
    BlprntCredentials::ApiKey(api_key) => {
      headers.insert("x-api-key", HeaderValue::from_str(api_key).context("invalid anthropic api key header")?);
    }
    BlprntCredentials::OauthToken(OauthToken::Anthropic(token)) => {
      headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token.access_token)).context("invalid anthropic oauth header")?,
      );
      headers.insert(USER_AGENT, HeaderValue::from_static("claude-cli/2.0.8 (external, cli)"));
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
      headers.insert("anthropic-dangerous-direct-browser-access", HeaderValue::from_static("true"));
      betas.push("oauth-2025-04-20".to_string());
    }
    other => {
      return Err(anyhow::anyhow!("unsupported anthropic credential shape: {:?}", other));
    }
  }

  if provider == Provider::ClaudeCode {
    let beta_header =
      HeaderValue::from_str(&betas.join(",")).context("invalid anthropic beta header for claude code")?;
    headers.insert("anthropic-beta", beta_header);
  }

  headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

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

async fn stream_anthropic_response(
  mut response: reqwest::Response,
  cancel_token: &CancellationToken,
  tx: mpsc::Sender<ProviderStreamEvent>,
) -> Result<()> {
  let status = response.status();
  let is_sse = response
    .headers()
    .get(CONTENT_TYPE)
    .and_then(|value| value.to_str().ok())
    .map(|value| value.starts_with("text/event-stream"))
    .unwrap_or(false);

  if !status.is_success() {
    let body = response.text().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?;
    return Err(anyhow::anyhow!("anthropic provider request failed: {} {}", status, body));
  }

  if !is_sse {
    let reply = parse_anthropic_response(response, cancel_token).await?;
    stream_provider_reply(reply, tx).await?;
    return Ok(());
  }

  #[derive(Clone, Debug)]
  enum AnthropicPendingBlock {
    Text { id: String },
    Thinking { id: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: String },
  }

  let mut frame_buffer = String::new();
  let mut pending_blocks: HashMap<u64, AnthropicPendingBlock> = HashMap::new();

  while let Some(chunk) =
    response.chunk().await.map_err(|error| normalize_provider_request_error(error, cancel_token))?
  {
    if cancel_token.is_cancelled() {
      return Err(run_cancelled_error());
    }

    frame_buffer.push_str(&String::from_utf8_lossy(&chunk));

    while let Some(frame) = drain_sse_frame(&mut frame_buffer) {
      let Some(value) = parse_sse_data_frame(&frame)? else {
        continue;
      };

      let Some(kind) = value.get("type").and_then(serde_json::Value::as_str) else {
        continue;
      };

      match kind {
        "content_block_start" => {
          let Some(index) = value.get("index").and_then(serde_json::Value::as_u64) else {
            continue;
          };

          let Some(content_block) = value.get("content_block") else {
            continue;
          };

          match content_block.get("type").and_then(serde_json::Value::as_str) {
            Some("text") => {
              let id = format!("text:{index}");
              pending_blocks.insert(index, AnthropicPendingBlock::Text { id: id.clone() });
              if let Some(text) = content_block.get("text").and_then(serde_json::Value::as_str)
                && !text.is_empty()
              {
                tx.send(ProviderStreamEvent::TextDelta { id, delta: text.to_string() })
                  .await
                  .context("failed to send anthropic text delta")?;
              }
            }
            Some("thinking") => {
              let id = format!("thinking:{index}");
              let signature = content_block.get("signature").and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
              pending_blocks
                .insert(index, AnthropicPendingBlock::Thinking { id: id.clone(), signature: signature.clone() });
              if let Some(thinking) = content_block.get("thinking").and_then(serde_json::Value::as_str)
                && !thinking.is_empty()
              {
                tx.send(ProviderStreamEvent::ThinkingDelta { id, delta: thinking.to_string() })
                  .await
                  .context("failed to send anthropic thinking delta")?;
              }
            }
            Some("tool_use") => {
              let tool_use_id = content_block
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .context("anthropic tool use is missing id")?;
              let name = content_block
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .context("anthropic tool use is missing name")?;
              pending_blocks
                .insert(index, AnthropicPendingBlock::ToolUse { id: tool_use_id, name, input: String::new() });
            }
            _ => {}
          }
        }
        "content_block_delta" => {
          let Some(index) = value.get("index").and_then(serde_json::Value::as_u64) else {
            continue;
          };
          let Some(delta) = value.get("delta") else {
            continue;
          };

          match (pending_blocks.get_mut(&index), delta.get("type").and_then(serde_json::Value::as_str)) {
            (Some(AnthropicPendingBlock::Text { id }), Some("text_delta")) => {
              if let Some(text) = delta.get("text").and_then(serde_json::Value::as_str)
                && !text.is_empty()
              {
                tx.send(ProviderStreamEvent::TextDelta { id: id.clone(), delta: text.to_string() })
                  .await
                  .context("failed to send anthropic text delta")?;
              }
            }
            (Some(AnthropicPendingBlock::Thinking { id, .. }), Some("thinking_delta")) => {
              if let Some(thinking) = delta.get("thinking").and_then(serde_json::Value::as_str)
                && !thinking.is_empty()
              {
                tx.send(ProviderStreamEvent::ThinkingDelta { id: id.clone(), delta: thinking.to_string() })
                  .await
                  .context("failed to send anthropic thinking delta")?;
              }
            }
            (Some(AnthropicPendingBlock::Thinking { signature, .. }), Some("signature_delta")) => {
              if let Some(next_signature) = delta.get("signature").and_then(serde_json::Value::as_str) {
                *signature = Some(next_signature.to_string());
              }
            }
            (Some(AnthropicPendingBlock::ToolUse { input, .. }), Some("input_json_delta")) => {
              if let Some(partial_json) = delta.get("partial_json").and_then(serde_json::Value::as_str) {
                input.push_str(partial_json);
              }
            }
            _ => {}
          }
        }
        "content_block_stop" => {
          let Some(index) = value.get("index").and_then(serde_json::Value::as_u64) else {
            continue;
          };

          match pending_blocks.remove(&index) {
            Some(AnthropicPendingBlock::Text { id }) => {
              tx.send(ProviderStreamEvent::TextDone { id, full_text: None })
                .await
                .context("failed to send anthropic text completion")?;
            }
            Some(AnthropicPendingBlock::Thinking { id, signature }) => {
              tx.send(ProviderStreamEvent::ThinkingDone { id, full_thinking: None, signature })
                .await
                .context("failed to send anthropic thinking completion")?;
            }
            Some(AnthropicPendingBlock::ToolUse { id, name, input }) => {
              tx.send(ProviderStreamEvent::ToolCall(ToolCallSpec {
                tool_use_id: id,
                tool_id:     runtime_tool_id_from_name(name)?,
                input:       serde_json::from_str(&input).context("failed to decode anthropic tool input")?,
              }))
              .await
              .context("failed to send anthropic tool call")?;
            }
            None => {}
          }
        }
        "error" => {
          let message = value
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("anthropic stream error");
          return Err(anyhow::anyhow!(message.to_string()));
        }
        _ => {}
      }
    }
  }

  Ok(())
}

fn serialize_json_value(value: &serde_json::Value) -> String {
  serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

async fn stream_provider_reply(reply: ProviderReply, tx: mpsc::Sender<ProviderStreamEvent>) -> Result<()> {
  if let Some(thinking) = reply.thinking.filter(|thinking| !thinking.trim().is_empty()) {
    tx.send(ProviderStreamEvent::ThinkingDelta { id: "thinking-0".to_string(), delta: thinking.clone() })
      .await
      .context("failed to send provider thinking delta")?;
    tx.send(ProviderStreamEvent::ThinkingDone {
      id:            "thinking-0".to_string(),
      full_thinking: Some(thinking),
      signature:     None,
    })
    .await
    .context("failed to send provider thinking completion")?;
  }

  if let Some(text) = reply.text.filter(|text| !text.trim().is_empty()) {
    tx.send(ProviderStreamEvent::TextDelta { id: "text-0".to_string(), delta: text.clone() })
      .await
      .context("failed to send provider text delta")?;
    tx.send(ProviderStreamEvent::TextDone { id: "text-0".to_string(), full_text: Some(text) })
      .await
      .context("failed to send provider text completion")?;
  }

  for tool_call in reply.tool_calls {
    tx.send(ProviderStreamEvent::ToolCall(tool_call)).await.context("failed to send provider tool call")?;
  }

  Ok(())
}

fn drain_sse_frame(buffer: &mut String) -> Option<String> {
  if let Some(index) = buffer.find("\n\n") {
    let frame = buffer[..index].to_string();
    buffer.drain(..index + 2);
    Some(frame)
  } else if let Some(index) = buffer.find("\r\n\r\n") {
    let frame = buffer[..index].to_string();
    buffer.drain(..index + 4);
    Some(frame)
  } else {
    None
  }
}

fn parse_sse_data_frame(frame: &str) -> Result<Option<serde_json::Value>> {
  let mut payload_lines = Vec::new();
  for raw_line in frame.lines() {
    let line = raw_line.trim_end_matches('\r');
    if let Some(data) = line.strip_prefix("data: ") {
      payload_lines.push(data);
    } else if let Some(data) = line.strip_prefix("data:") {
      payload_lines.push(data.trim_start());
    }
  }

  if payload_lines.is_empty() {
    return Ok(None);
  }

  let payload = payload_lines.join("\n");
  if payload.trim() == "[DONE]" {
    return Ok(None);
  }

  Ok(Some(serde_json::from_str(&payload).context("failed to decode sse data payload")?))
}

async fn handle_openai_sse_event(
  provider: Provider,
  value: serde_json::Value,
  tx: &mpsc::Sender<ProviderStreamEvent>,
  openrouter_tool_calls: &mut HashMap<u32, (String, String, String)>,
  reasoning_ids: &mut HashMap<u32, String>,
) -> Result<()> {
  let Some(kind) = value.get("type").and_then(serde_json::Value::as_str) else {
    return Ok(());
  };

  match kind {
    "response.output_item.added" => {
      let Some(output_index) = value.get("output_index").and_then(serde_json::Value::as_u64) else {
        return Ok(());
      };
      let Some(item) = value.get("item") else {
        return Ok(());
      };

      match item.get("type").and_then(serde_json::Value::as_str) {
        Some("reasoning") => {
          let id = item
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("reasoning:{output_index}"));
          reasoning_ids.insert(output_index as u32, id);
        }
        Some("function_call") if provider == Provider::OpenRouter => {
          let tool_use_id = item
            .get("call_id")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .context("openrouter tool call is missing call_id")?;
          let name = item
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .context("openrouter tool call is missing name")?;
          openrouter_tool_calls.insert(output_index as u32, (tool_use_id, name, String::new()));
        }
        _ => {}
      }
    }
    "response.output_text.delta" => {
      let id = value
        .get("item_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .context("openai-compatible output text delta is missing item_id")?;
      let delta = value.get("delta").and_then(serde_json::Value::as_str).map(ToOwned::to_owned).unwrap_or_default();
      if !delta.is_empty() {
        tx.send(ProviderStreamEvent::TextDelta { id, delta })
          .await
          .context("failed to send openai-compatible text delta")?;
      }
    }
    "response.output_text.done" => {
      let id = value
        .get("item_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .context("openai-compatible output text done is missing item_id")?;
      let text = value.get("text").and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
      tx.send(ProviderStreamEvent::TextDone { id, full_text: text })
        .await
        .context("failed to send openai-compatible text completion")?;
    }
    "response.reasoning_text.delta" | "response.reasoning_summary_text.delta" => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      let id = reasoning_ids.get(&output_index).cloned().unwrap_or_else(|| format!("reasoning:{output_index}"));
      let delta = value.get("delta").and_then(serde_json::Value::as_str).map(ToOwned::to_owned).unwrap_or_default();
      if !delta.is_empty() {
        tx.send(ProviderStreamEvent::ThinkingDelta { id, delta })
          .await
          .context("failed to send openai-compatible thinking delta")?;
      }
    }
    "response.reasoning_text.done" => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      let id = reasoning_ids.remove(&output_index).unwrap_or_else(|| format!("reasoning:{output_index}"));
      let thinking = value.get("text").and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
      tx.send(ProviderStreamEvent::ThinkingDone { id, full_thinking: thinking, signature: None })
        .await
        .context("failed to send openai-compatible thinking completion")?;
    }
    "response.reasoning_summary_text.done" => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      let id = reasoning_ids.remove(&output_index).unwrap_or_else(|| format!("reasoning:{output_index}"));
      let thinking = value.get("text").and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
      tx.send(ProviderStreamEvent::ThinkingDone { id, full_thinking: thinking, signature: None })
        .await
        .context("failed to send openai-compatible reasoning summary completion")?;
    }
    "response.reasoning_summary_part.done" => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      let id = reasoning_ids.remove(&output_index).unwrap_or_else(|| format!("reasoning:{output_index}"));
      let thinking =
        value.get("part").and_then(|part| part.get("text")).and_then(serde_json::Value::as_str).map(ToOwned::to_owned);
      tx.send(ProviderStreamEvent::ThinkingDone { id, full_thinking: thinking, signature: None })
        .await
        .context("failed to send openai-compatible reasoning part completion")?;
    }
    "response.function_call_arguments.delta" if provider == Provider::OpenRouter => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      if let Some((_, _, input)) = openrouter_tool_calls.get_mut(&output_index)
        && let Some(delta) = value.get("delta").and_then(serde_json::Value::as_str)
      {
        input.push_str(delta);
      }
    }
    "response.function_call_arguments.done" if provider == Provider::OpenRouter => {
      let output_index =
        value.get("output_index").and_then(serde_json::Value::as_u64).map(|index| index as u32).unwrap_or_default();
      if let Some((tool_use_id, name, input)) = openrouter_tool_calls.remove(&output_index) {
        let arguments =
          value.get("arguments").and_then(serde_json::Value::as_str).map(ToOwned::to_owned).unwrap_or(input);
        tx.send(ProviderStreamEvent::ToolCall(ToolCallSpec {
          tool_use_id,
          tool_id: runtime_tool_id_from_name(name)?,
          input: serde_json::from_str(&arguments).context("failed to decode openrouter tool arguments")?,
        }))
        .await
        .context("failed to send openrouter tool call")?;
      }
    }
    "response.output_item.done" if provider != Provider::OpenRouter => {
      let Some(item) = value.get("item") else {
        return Ok(());
      };
      if item.get("type").and_then(serde_json::Value::as_str) == Some("function_call") {
        let tool_use_id = item
          .get("call_id")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("openai-compatible tool call is missing call_id")?;
        let name = item
          .get("name")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .context("openai-compatible tool call is missing name")?;
        let arguments = item
          .get("arguments")
          .and_then(serde_json::Value::as_str)
          .map(ToOwned::to_owned)
          .unwrap_or_else(|| "{}".to_string());

        tx.send(ProviderStreamEvent::ToolCall(ToolCallSpec {
          tool_use_id,
          tool_id: runtime_tool_id_from_name(name)?,
          input: serde_json::from_str(&arguments).context("failed to decode openai-compatible tool arguments")?,
        }))
        .await
        .context("failed to send openai-compatible tool call")?;
      }
    }
    "response.failed" => {
      let message = value
        .get("response")
        .and_then(|response| response.get("error"))
        .and_then(|error| error.get("message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("openai-compatible stream failed");
      return Err(anyhow::anyhow!(message.to_string()));
    }
    _ => {}
  }

  Ok(())
}

fn bearer_auth_header(credentials: Option<&BlprntCredentials>, provider: Provider) -> Result<HeaderValue> {
  let token = match credentials.context(format!("{provider} provider is missing credentials"))? {
    BlprntCredentials::ApiKey(api_key) => api_key.clone(),
    BlprntCredentials::OauthToken(OauthToken::OpenAi(token))
      if matches!(provider, Provider::OpenAi | Provider::OpenRouter | Provider::Codex) =>
    {
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
  if provider_config.provider == Provider::Mock {
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
    Provider::Mock
    | Provider::OpenAi
    | Provider::OpenRouter
    | Provider::Anthropic
    | Provider::ClaudeCode
    | Provider::Codex => Ok(()),
  }
}

fn anthropic_messages_url(base_url: &str, provider: Provider, credentials: Option<&BlprntCredentials>) -> String {
  let suffix = if provider == Provider::ClaudeCode
    || matches!(credentials, Some(BlprntCredentials::OauthToken(OauthToken::Anthropic(_))))
  {
    "/v1/messages?beta=true"
  } else {
    "/v1/messages"
  };

  format!("{}{}", base_url.trim_end_matches('/'), suffix)
}

fn parse_openai_sse_payload(body: &str) -> Result<serde_json::Value> {
  let mut last_value = None;

  for line in body.lines() {
    let Some(payload) = line.strip_prefix("data: ") else {
      continue;
    };
    if payload.trim() == "[DONE]" {
      continue;
    }

    last_value =
      Some(serde_json::from_str::<serde_json::Value>(payload).context("failed to decode openai-compatible sse event")?);
  }

  last_value.context("openai-compatible sse response did not contain a data payload")
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
  Ok(shared::paths::employee_home(&employee_id.uuid().to_string()))
}

fn project_home_for_run(project: Option<&ProjectRecord>) -> Option<PathBuf> {
  project.map(|record| shared::paths::project_home(&record.id.uuid().to_string()))
}

fn is_ceo_wide_write_tool(tool_id: &ToolId) -> bool {
  matches!(tool_id, ToolId::ApplyPatch | ToolId::Shell)
}

fn tool_working_directories(runtime_config: &ToolRuntimeConfig, project: Option<&ProjectRecord>) -> Vec<PathBuf> {
  let mut directories = Vec::new();

  if let Some(project) = project {
    directories.extend(project.working_directories.iter().map(PathBuf::from));
  }

  if let Some(agent_home) = &runtime_config.agent_home
    && !directories.iter().any(|path| path == agent_home)
  {
    directories.push(agent_home.clone());
  }

  if let Some(project_home) = &runtime_config.project_home {
    if !directories.iter().any(|path| path == project_home) {
      directories.push(project_home.clone());
    }
  }

  if directories.is_empty() {
    directories.push(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
  }

  directories
}

async fn ceo_tool_working_directories(mut directories: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
  let employee_homes_dir = shared::paths::employee_homes_dir();
  fs::create_dir_all(&employee_homes_dir)
    .with_context(|| format!("failed to create {}", employee_homes_dir.display()))?;
  push_unique_path(&mut directories, employee_homes_dir);

  for employee in EmployeeRepository::list().await.context("failed to list employees for ceo tool scope")? {
    let employee_home = shared::paths::employee_home(&employee.id.uuid().to_string());
    fs::create_dir_all(&employee_home).with_context(|| format!("failed to create {}", employee_home.display()))?;
    push_unique_path(&mut directories, employee_home);
  }

  let project_homes_dir = shared::paths::project_homes_dir();
  fs::create_dir_all(&project_homes_dir)
    .with_context(|| format!("failed to create {}", project_homes_dir.display()))?;
  push_unique_path(&mut directories, project_homes_dir);

  for project in ProjectRepository::list().await.context("failed to list projects for ceo tool scope")? {
    for working_directory in &project.working_directories {
      push_unique_path(&mut directories, PathBuf::from(working_directory));
    }

    let project_home = shared::paths::project_home(&project.id.uuid().to_string());
    fs::create_dir_all(&project_home).with_context(|| format!("failed to create {}", project_home.display()))?;
    push_unique_path(&mut directories, project_home);
  }

  Ok(directories)
}

fn push_unique_path(directories: &mut Vec<PathBuf>, path: PathBuf) {
  if !directories.iter().any(|existing| existing == &path) {
    directories.push(path);
  }
}

fn ensure_runtime_homes(runtime_config: &ToolRuntimeConfig) -> Result<()> {
  if let Some(agent_home) = &runtime_config.agent_home {
    fs::create_dir_all(agent_home).with_context(|| format!("failed to create {}", agent_home.display()))?;
  }

  if let Some(project_home) = &runtime_config.project_home {
    fs::create_dir_all(project_home).with_context(|| format!("failed to create {}", project_home.display()))?;

    let memory_dir = project_home.join("memory");
    fs::create_dir_all(&memory_dir).with_context(|| format!("failed to create {}", memory_dir.display()))?;

    let plans_dir = project_home.join("plans");
    fs::create_dir_all(&plans_dir).with_context(|| format!("failed to create {}", plans_dir.display()))?;
  }

  Ok(())
}

async fn append_request_text(turn_id: TurnId, text: String) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepSide::Request,
    TurnStepContent::Text(TurnStepText { text, signature: None, visibility: ContentsVisibility::Full }),
  )
  .await
  .context("failed to append request text")?;
  Ok(())
}

async fn append_response_text(turn_id: TurnId, text: String) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepSide::Response,
    TurnStepContent::Text(TurnStepText { text, signature: None, visibility: ContentsVisibility::Full }),
  )
  .await
  .context("failed to append response text")?;
  Ok(())
}

async fn append_stream_text(
  turn_id: TurnId,
  stream_state: &mut StreamingAssistantState,
  content_id: &str,
  delta: String,
) -> Result<()> {
  let entry = stream_state.texts.entry(content_id.to_string()).or_default();
  entry.push_str(&delta);

  let fragment = StreamFragmentKey { kind: StreamFragmentKind::Text, id: content_id.to_string() };
  if stream_state.current_fragment.as_ref() == Some(&fragment) {
    mutate_last_assistant_content(turn_id, |content| match content {
      TurnStepContent::Text(text) => {
        text.text.push_str(&delta);
        true
      }
      _ => false,
    })
    .await?;
  } else {
    append_response_text(turn_id, delta).await.context("failed to append streaming text")?;
    stream_state.current_fragment = Some(fragment);
  }

  Ok(())
}

async fn append_stream_thinking(
  turn_id: TurnId,
  stream_state: &mut StreamingAssistantState,
  content_id: &str,
  delta: String,
) -> Result<()> {
  let entry = stream_state.thinkings.entry(content_id.to_string()).or_default();
  entry.push_str(&delta);

  let fragment = StreamFragmentKey { kind: StreamFragmentKind::Thinking, id: content_id.to_string() };
  if stream_state.current_fragment.as_ref() == Some(&fragment) {
    mutate_last_assistant_content(turn_id, |content| match content {
      TurnStepContent::Thinking(thinking) => {
        thinking.thinking.push_str(&delta);
        true
      }
      _ => false,
    })
    .await?;
  } else {
    TurnRepository::insert_step_content(
      turn_id,
      TurnStepSide::Response,
      TurnStepContent::Thinking(TurnStepThinking {
        thinking:   delta,
        signature:  String::new(),
        visibility: ContentsVisibility::Assistant,
      }),
    )
    .await
    .context("failed to append streaming thinking")?;
    stream_state.current_fragment = Some(fragment);
  }

  Ok(())
}

async fn set_last_thinking_signature(
  turn_id: TurnId,
  stream_state: &mut StreamingAssistantState,
  content_id: &str,
  signature: String,
) -> Result<()> {
  let fragment = StreamFragmentKey { kind: StreamFragmentKind::Thinking, id: content_id.to_string() };
  if stream_state.current_fragment.as_ref() != Some(&fragment) {
    return Ok(());
  }

  mutate_last_assistant_content(turn_id, |content| match content {
    TurnStepContent::Thinking(thinking) => {
      thinking.signature = signature.clone();
      true
    }
    _ => false,
  })
  .await
}

async fn append_tool_use(turn_id: TurnId, tool_call: ToolCallSpec) -> Result<()> {
  TurnRepository::insert_step_content(
    turn_id,
    TurnStepSide::Response,
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
    TurnStepSide::Request,
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

async fn mutate_last_assistant_content(
  turn_id: TurnId,
  mut mutate: impl FnMut(&mut TurnStepContent) -> bool,
) -> Result<()> {
  let mut turn = TurnRepository::get(turn_id.clone()).await.context("failed to reload turn for streaming update")?;
  let Some(last_step) = turn.steps.last_mut() else {
    return Ok(());
  };

  let Some(last_content) = last_step.response.contents.last_mut() else {
    return Ok(());
  };

  if !mutate(last_content) {
    return Ok(());
  }

  TurnRepository::update(turn_id, turn.steps).await.context("failed to persist streaming content update")?;
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
