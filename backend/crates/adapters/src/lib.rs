pub mod prompt;
pub mod mcp;
pub mod runtime;
pub mod traits;

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;
  use std::fs;
  use std::path::PathBuf;
  use std::sync::Arc;
  use std::sync::LazyLock;
  use std::sync::Mutex;
  use std::time::Duration;
  use std::time::Instant;

  use axum::Json;
  use axum::Router;
  use axum::extract::State;
  use axum::http::StatusCode;
  use axum::response::IntoResponse;
  use axum::response::Response;
  use axum::routing::post;
  use events::COORDINATOR_EVENTS;
  use events::CoordinatorEvent;
  use persistence::prelude::DbId;
  use persistence::prelude::EmployeeId;
  use persistence::prelude::EmployeeKind;
  use persistence::prelude::EmployeeModel;
  use persistence::prelude::EmployeePatch;
  use persistence::prelude::EmployeeProviderConfig;
  use persistence::prelude::EmployeeRepository;
  use persistence::prelude::EmployeeRole;
  use persistence::prelude::EmployeeRuntimeConfig;
  use persistence::prelude::EmployeeSkillRef;
  use persistence::prelude::IssueModel;
  use persistence::prelude::IssuePriority;
  use persistence::prelude::IssueRepository;
  use persistence::prelude::IssueStatus;
  use persistence::prelude::McpServerModel;
  use persistence::prelude::McpServerRepository;
  use persistence::prelude::ProjectModel;
  use persistence::prelude::ProjectRepository;
  use persistence::prelude::ProviderModel;
  use persistence::prelude::ProviderPatch;
  use persistence::prelude::ProviderRecord;
  use persistence::prelude::ProviderRepository;
  use persistence::prelude::ReasoningEffort;
  use persistence::prelude::RunModel;
  use persistence::prelude::RunEnabledMcpServerRepository;
  use persistence::prelude::RunRepository;
  use persistence::prelude::RunStatus;
  use persistence::prelude::RunTrigger;
  use persistence::prelude::SurrealConnection;
  use persistence::prelude::TurnStepContent;
  use persistence::prelude::TurnStepStatus;
  use serde_json::Value;
  use shared::agent::AnthropicOauthToken;
  use shared::agent::BlprntCredentials;
  use shared::agent::OauthToken;
  use shared::agent::OpenAiOauthToken;
  use shared::agent::Provider;
  use shared::agent::ToolId;
  use tokio::net::TcpListener;
  use tokio::sync::Mutex as AsyncMutex;
  use tokio::task::JoinHandle;
  use tokio::time::sleep;
  use tokio_util::sync::CancellationToken;
  use vault::Vault;

  use crate::prompt::PromptAssemblyInput;
  use crate::runtime::AdapterRuntime;
  use crate::runtime::ProviderClient;
  use crate::runtime::ProviderFactory;
  use crate::runtime::ProviderReply;
  use crate::runtime::ProviderRequest;
  use crate::runtime::ProviderSelection;
  use crate::runtime::ScriptedProviderFactory;
  use crate::runtime::ScriptedProviderReply;
  use crate::runtime::ToolCallSpec;

  static TEST_LOCK: Mutex<()> = Mutex::new(());
  static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("failed to create test runtime")
  });

  fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  fn assert_option_f64_close(actual: Option<f64>, expected: f64) {
    let actual = actual.expect("expected a float value");
    assert!(
      (actual - expected).abs() < f64::EPSILON * 8.0,
      "expected {expected}, got {actual}"
    );
  }

  async fn reset_test_db() {
    SurrealConnection::reset().await.expect("test database should reset");
    let home = unique_temp_dir("adapter-test-home-root");
    unsafe { std::env::set_var("BLPRNT_HOME", &home) };
    unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", &home) };
  }

  fn unique_temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", persistence::Uuid::new_v4()));
    fs::create_dir_all(&path).expect("failed to create temp dir");
    path
  }

  struct HomeGuard {
    previous_home:             Option<String>,
    previous_blprnt_home:      Option<String>,
    previous_memory_base_dir:  Option<String>,
  }

  impl HomeGuard {
    fn set(path: &std::path::Path) -> Self {
      let previous_home = std::env::var("HOME").ok();
      let previous_blprnt_home = std::env::var("BLPRNT_HOME").ok();
      let previous_memory_base_dir = std::env::var("BLPRNT_MEMORY_BASE_DIR").ok();
      unsafe { std::env::set_var("HOME", path) };
      unsafe { std::env::set_var("BLPRNT_HOME", path) };
      unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", path) };
      Self { previous_home, previous_blprnt_home, previous_memory_base_dir }
    }
  }

  impl Drop for HomeGuard {
    fn drop(&mut self) {
      match &self.previous_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
      }

      match &self.previous_blprnt_home {
        Some(path) => unsafe { std::env::set_var("BLPRNT_HOME", path) },
        None => unsafe { std::env::remove_var("BLPRNT_HOME") },
      }

      match &self.previous_memory_base_dir {
        Some(path) => unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", path) },
        None => unsafe { std::env::remove_var("BLPRNT_MEMORY_BASE_DIR") },
      }
    }
  }

  #[derive(Clone)]
  struct JsonStubState {
    requests:  Arc<AsyncMutex<Vec<Value>>>,
    responses: Arc<AsyncMutex<VecDeque<Value>>>,
  }

  struct JsonStubServer {
    base_url:  String,
    requests:  Arc<AsyncMutex<Vec<Value>>>,
    _listener: JoinHandle<()>,
  }

  struct SseStubServer {
    base_url:  String,
    requests:  Arc<AsyncMutex<Vec<Value>>>,
    _listener: JoinHandle<()>,
  }

  #[derive(Clone, Default)]
  struct EmptyReplyProviderFactory;

  struct EmptyReplyProviderClient;

  #[derive(Clone)]
  struct StreamingTestProviderFactory {
    chunks:   Vec<&'static str>,
    delay_ms: u64,
  }

  struct StreamingTestProviderClient {
    chunks:   Vec<&'static str>,
    delay_ms: u64,
  }

  impl ProviderFactory for EmptyReplyProviderFactory {
    fn client(&self, _selection: &ProviderSelection) -> Arc<dyn ProviderClient> {
      Arc::new(EmptyReplyProviderClient)
    }
  }

  #[async_trait::async_trait]
  impl ProviderClient for EmptyReplyProviderClient {
    async fn next_reply(
      &self,
      _request: ProviderRequest,
      _cancel_token: CancellationToken,
    ) -> anyhow::Result<ProviderReply> {
      Ok(ProviderReply::default())
    }
  }

  impl ProviderFactory for StreamingTestProviderFactory {
    fn client(&self, _selection: &ProviderSelection) -> Arc<dyn ProviderClient> {
      Arc::new(StreamingTestProviderClient { chunks: self.chunks.clone(), delay_ms: self.delay_ms })
    }
  }

  #[async_trait::async_trait]
  impl ProviderClient for StreamingTestProviderClient {
    async fn next_reply(
      &self,
      _request: ProviderRequest,
      _cancel_token: CancellationToken,
    ) -> anyhow::Result<ProviderReply> {
      panic!("streaming test provider should use stream_reply")
    }

    async fn stream_reply(
      &self,
      _request: ProviderRequest,
      _tool_specs: Vec<shared::tools::ToolSpec>,
      cancel_token: CancellationToken,
      tx: tokio::sync::mpsc::Sender<crate::runtime::ProviderStreamEvent>,
    ) -> anyhow::Result<()> {
      for chunk in &self.chunks {
        if cancel_token.is_cancelled() {
          return Err(anyhow::anyhow!("cancelled"));
        }

        tx.send(crate::runtime::ProviderStreamEvent::TextDelta {
          id:    "stream-text".to_string(),
          delta: (*chunk).to_string(),
        })
        .await
        .expect("streaming test chunk should send");
        sleep(Duration::from_millis(self.delay_ms)).await;
      }

      tx.send(crate::runtime::ProviderStreamEvent::TextDone { id: "stream-text".to_string(), full_text: None })
        .await
        .expect("streaming test completion should send");

      Ok(())
    }
  }

  async fn create_employee_with_slug(provider: Provider, slug: &str, heartbeat_prompt: &str) -> EmployeeId {
    create_employee_with_role(provider, slug, heartbeat_prompt, EmployeeRole::Staff).await
  }

  async fn create_employee_with_role(
    provider: Provider,
    slug: &str,
    heartbeat_prompt: &str,
    role: EmployeeRole,
  ) -> EmployeeId {
    let employee = EmployeeRepository::create(EmployeeModel {
      name: "Runtime".to_string(),
      kind: EmployeeKind::Agent,
      role,
      title: "Runtime".to_string(),
      provider_config: Some(EmployeeProviderConfig { provider, slug: slug.to_string() }),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       heartbeat_prompt.to_string(),
        wake_on_demand:         true,
        timer_wakeups_enabled:  Some(true),
        dreams_enabled:         Some(false),
        max_concurrent_runs:    1,
        skill_stack:            None,
        reasoning_effort:       None,
      }),
      ..Default::default()
    })
    .await
    .expect("employee should be created");

    employee.id
  }

  async fn create_employee(provider: Provider, heartbeat_prompt: &str) -> EmployeeId {
    create_employee_with_slug(provider, "test-model", heartbeat_prompt).await
  }

  async fn create_ceo_employee(provider: Provider, heartbeat_prompt: &str) -> EmployeeId {
    create_employee_with_role(provider, "test-model", heartbeat_prompt, EmployeeRole::Ceo).await
  }

  async fn json_stub_handler(State(state): State<JsonStubState>, Json(payload): Json<Value>) -> Json<Value> {
    state.requests.lock().await.push(payload);
    let response =
      state.responses.lock().await.pop_front().expect("stub received more requests than configured responses");
    Json(response)
  }

  async fn spawn_responses_stub(path: &str, responses: Vec<Value>) -> JsonStubServer {
    let requests = Arc::new(AsyncMutex::new(Vec::new()));
    let state = JsonStubState { requests: requests.clone(), responses: Arc::new(AsyncMutex::new(responses.into())) };
    let app = Router::new().route(path, post(json_stub_handler)).with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
    let address = listener.local_addr().expect("listener addr");
    let server = tokio::spawn(async move {
      axum::serve(listener, app).await.expect("stub server should run");
    });

    JsonStubServer { base_url: format!("http://{}", address), requests, _listener: server }
  }

  async fn text_mcp_stub_handler(State(state): State<JsonStubState>, Json(payload): Json<Value>) -> Response {
    state.requests.lock().await.push(payload.clone());
    let method = payload.get("method").and_then(serde_json::Value::as_str).unwrap_or_default();
    let id = payload.get("id").cloned().unwrap_or_else(|| serde_json::json!(1));
    let result = match method {
      "initialize" => serde_json::json!({
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "serverInfo": { "name": "stub-mcp", "version": "1.0.0" }
      }),
      "notifications/initialized" => return (StatusCode::ACCEPTED, [("content-type", "application/json")], "").into_response(),
      "tools/list" => serde_json::json!({
        "tools": [
          {
            "name": "lookup_doc",
            "description": "Lookup docs from MCP stub",
            "inputSchema": {
              "type": "object",
              "properties": { "query": { "type": "string" } },
              "required": ["query"]
            }
          }
        ]
      }),
      "tools/call" => serde_json::json!({
        "content": [{ "type": "text", "text": "stub tool ok" }],
        "structuredContent": {
          "echo": payload["params"]["arguments"].clone(),
          "server": "stub-mcp"
        },
        "isError": false
      }),
      other => serde_json::json!({
        "content": [{ "type": "text", "text": format!("unhandled method {other}") }],
        "isError": true
      }),
    };

    Json(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result })).into_response()
  }

  async fn spawn_text_mcp_stub(path: &str) -> JsonStubServer {
    let requests = Arc::new(AsyncMutex::new(Vec::new()));
    let state = JsonStubState { requests: requests.clone(), responses: Arc::new(AsyncMutex::new(VecDeque::new())) };
    let app = Router::new().route(path, post(text_mcp_stub_handler)).with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
    let address = listener.local_addr().expect("listener addr");
    let server = tokio::spawn(async move {
      axum::serve(listener, app).await.expect("stub server should run");
    });

    JsonStubServer { base_url: format!("http://{}", address), requests, _listener: server }
  }

  async fn unauthorized_mcp_stub_handler(State(state): State<JsonStubState>, Json(payload): Json<Value>) -> Response {
    state.requests.lock().await.push(payload);
    (
      StatusCode::UNAUTHORIZED,
      [("content-type", "application/json")],
      serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
          "code": -32001,
          "message": "unauthorized"
        }
      })
      .to_string(),
    )
      .into_response()
  }

  async fn spawn_unauthorized_mcp_stub(path: &str) -> JsonStubServer {
    let requests = Arc::new(AsyncMutex::new(Vec::new()));
    let state = JsonStubState { requests: requests.clone(), responses: Arc::new(AsyncMutex::new(VecDeque::new())) };
    let app = Router::new().route(path, post(unauthorized_mcp_stub_handler)).with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
    let address = listener.local_addr().expect("listener addr");
    let server = tokio::spawn(async move {
      axum::serve(listener, app).await.expect("stub server should run");
    });

    JsonStubServer { base_url: format!("http://{}", address), requests, _listener: server }
  }

  async fn sse_stub_handler(State(state): State<JsonStubState>, Json(payload): Json<Value>) -> Response {
    state.requests.lock().await.push(payload);
    let response =
      state.responses.lock().await.pop_front().expect("stub received more requests than configured responses");
    let body =
      response.get("body").and_then(serde_json::Value::as_str).expect("sse stub body should be a string").to_string();

    (StatusCode::OK, [("content-type", "text/event-stream")], body).into_response()
  }

  async fn spawn_sse_stub(path: &str, responses: Vec<Value>) -> SseStubServer {
    let requests = Arc::new(AsyncMutex::new(Vec::new()));
    let state = JsonStubState { requests: requests.clone(), responses: Arc::new(AsyncMutex::new(responses.into())) };
    let app = Router::new().route(path, post(sse_stub_handler)).with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
    let address = listener.local_addr().expect("listener addr");
    let server = tokio::spawn(async move {
      axum::serve(listener, app).await.expect("stub server should run");
    });

    SseStubServer { base_url: format!("http://{}", address), requests, _listener: server }
  }

  async fn upsert_provider_credentials(provider: Provider, base_url: String, credential: &str) -> ProviderRecord {
    let record = match ProviderRepository::get_by_provider(provider).await {
      Some(existing) => ProviderRepository::update(
        existing.id.clone(),
        ProviderPatch { base_url: Some(base_url), ..Default::default() },
      )
      .await
      .expect("provider should update"),
      None => {
        let mut model = ProviderModel::new(provider);
        model.base_url = Some(base_url);
        ProviderRepository::create(model).await.expect("provider should create")
      }
    };

    vault::set_stronghold_secret(Vault::Key, record.id.uuid(), credential)
      .await
      .expect("provider credential should persist");

    record
  }

  #[test]
  fn assembles_system_prompt_in_deliberate_order_and_injects_issue_context() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-prompt-home-root");
      let _home_guard = HomeGuard::set(&home);
      let agent_home = unique_temp_dir("adapter-prompt-home");
      fs::write(agent_home.join("HEARTBEAT.md"), "heartbeat instructions").expect("heartbeat file");
      fs::write(agent_home.join("SOUL.md"), "soul instructions").expect("soul file");
      fs::write(agent_home.join("AGENTS.md"), "agent instructions").expect("agents file");
      fs::write(agent_home.join("TOOLS.md"), "tool instructions").expect("tools file");
      let project_workdir = home.join("workspace-a");
      fs::create_dir_all(&project_workdir).expect("project workdir");
      fs::write(project_workdir.join("AGENTS.md"), "project agent instructions").expect("project agents file");
      let skill_dir = home.join(".agents").join("skills").join("custom-skill");
      fs::create_dir_all(&skill_dir).expect("skill dir");
      fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: custom-skill\ndescription: custom skill\n---\n\n# Custom Skill\n",
      )
      .expect("skill file");
      let issue_id = persistence::prelude::IssueId::from(
        "00000000-0000-0000-0000-000000000059".parse::<persistence::Uuid>().expect("fixed issue id"),
      );
      let issue_id_text = issue_id.uuid().to_string();

      let prompt = PromptAssemblyInput {
        agent_home:           agent_home.clone(),
        project_home:         Some(home.join(".blprnt").join("projects").join("project-1")),
        project_workdirs:     vec![project_workdir.clone(), home.join("workspace-b")],
        employee_id:          "employee-1".to_string(),
        api_url:              "http://127.0.0.1:3100".to_string(),
        operating_system:     "macos".to_string(),
        heartbeat_prompt:     "runtime prompt".to_string(),
        available_skills:     vec![EmployeeSkillRef {
          name: "custom-skill".to_string(),
          path: skill_dir.join("SKILL.md").to_string_lossy().to_string(),
        }],
        injected_skill_stack: vec![crate::prompt::InjectedSkillPrompt {
          name:     "custom-skill".to_string(),
          path:     skill_dir.join("SKILL.md").to_string_lossy().to_string(),
          contents: fs::read_to_string(skill_dir.join("SKILL.md")).expect("skill body"),
        }],
        trigger:              RunTrigger::IssueAssignment { issue_id: issue_id.clone() },
        dreaming_date:        None,
        daily_memory_content: None,
        prior_memory_content: None,
        issue_id:             Some(issue_id.uuid()),
        issue_identifier:     Some("BLP-59".to_string()),
        issue_title:          Some("Prompt assembly issue".to_string()),
        issue_description:    Some("Carry the assigned issue details into the user prompt.".to_string()),
        issue_status:         Some(IssueStatus::InProgress),
        issue_priority:       Some(IssuePriority::High),
        trigger_comment:      None,
        trigger_commenter:    None,
        available_mcp_servers: vec![crate::prompt::PromptMcpServerCatalogEntry {
          server_id: "mcp-server-1".to_string(),
          display_name: "QMD".to_string(),
          description: "Structured knowledge retrieval".to_string(),
          auth_state: shared::tools::McpServerAuthState::Connected,
        }],
      }
      .build();

      let stub_index =
        prompt.system_prompt.find("You operate as a blprnt employee inside the blprnt system.").expect("stub prompt");
      let os_index = prompt.system_prompt.find("Operating system: macos").expect("os metadata");
      let project_home_index = prompt.system_prompt.find("PROJECT_HOME:").expect("project home metadata");
      let project_workdirs_index =
        prompt.system_prompt.find("Project Working Directories").expect("project workdirs metadata");
      let heartbeat_index = prompt.system_prompt.find("heartbeat instructions").expect("heartbeat prompt");
      let soul_index = prompt.system_prompt.find("soul instructions").expect("soul prompt");
      let agents_index = prompt.system_prompt.find("agent instructions").expect("agents prompt");
      let tools_index = prompt.system_prompt.find("tool instructions").expect("tools prompt");
      let project_agents_index =
        prompt.system_prompt.find("project agent instructions").expect("project agents prompt");
      let runtime_index = prompt.system_prompt.find("runtime prompt").expect("runtime prompt");
      let available_skills_index = prompt.system_prompt.find("Available Runtime Skills").expect("available skills");
      let injected_skill_index = prompt.system_prompt.find("Employee Skill Stack: custom-skill").expect("skill stack");

      assert!(stub_index < os_index);
      assert!(os_index < project_home_index);
      assert!(project_home_index < project_workdirs_index);
      assert!(project_workdirs_index < heartbeat_index);
      assert!(heartbeat_index < soul_index);
      assert!(soul_index < agents_index);
      assert!(agents_index < tools_index);
      assert!(tools_index < project_agents_index);
      assert!(project_agents_index < runtime_index);
      assert!(runtime_index < available_skills_index);
      assert!(available_skills_index < injected_skill_index);
      assert!(prompt.system_prompt.contains("## Available MCP Servers"));
      assert!(prompt.system_prompt.contains("enable_mcp_server"));
      assert!(prompt.system_prompt.contains("QMD (mcp-server-1) — Structured knowledge retrieval [connected]"));
      assert!(prompt.system_prompt.contains("Use PROJECT_HOME for blprnt-managed project metadata only"));
      assert!(prompt.system_prompt.contains("PROJECT_HOME is writable as a whole"));
      assert!(prompt.system_prompt.contains("PROJECT_HOME/plans stores plan documents"));
      assert!(prompt.system_prompt.contains("These are the actual project source/work directories"));
      assert!(prompt.system_prompt.contains("## SOUL.md"));
      assert!(prompt.system_prompt.contains("## TOOLS.md"));
      assert!(prompt.system_prompt.contains("## Project AGENTS.md"));
      assert!(
        prompt.system_prompt.contains("Always read and follow the `blprnt` and `blprnt-memory` skills before acting")
      );
      assert!(prompt.system_prompt.contains("When you need to create or revise durable files"));
      assert!(
        prompt.system_prompt.contains("write them with the `apply_patch` tool inside `AGENT_HOME` or `PROJECT_HOME`")
      );
      assert!(prompt.system_prompt.contains("## Run Trigger Guidance"));
      assert!(prompt.system_prompt.contains("`issue_assignment`: you were woken because a specific issue was assigned to you"));
      assert!(prompt.system_prompt.contains("do not begin by searching for other assignments to decide why you are here"));
      assert!(prompt.system_prompt.contains("`issue_mention`: you were woken because a specific issue comment mentioned you"));
      assert!(prompt.system_prompt.contains("Start with that issue and the triggering comment"));
      assert!(
        prompt.system_prompt.contains("Before you exit a non-idle run, append a brief daily note to `AGENT_HOME/memory/YYYY-MM-DD.md`")
      );
      assert!(prompt.user_prompt.contains("Use the blprnt API to continue your blprnt work."));
      assert!(prompt.user_prompt.contains("Trigger: issue_assignment"));
      assert!(prompt.user_prompt.contains(&issue_id_text));
      assert!(prompt.user_prompt.contains("Issue Identifier: BLP-59"));
      assert!(prompt.user_prompt.contains("Issue Title: Prompt assembly issue"));
      assert!(prompt.user_prompt.contains("Issue Status: in_progress"));
      assert!(prompt.user_prompt.contains("Issue Priority: high"));
      assert!(prompt.user_prompt.contains("Carry the assigned issue details into the user prompt."));
      assert!(prompt.system_prompt.contains(skill_dir.join("SKILL.md").to_string_lossy().as_ref()));
      assert!(prompt.system_prompt.contains("# Custom Skill"));
    });
  }

  #[test]
  fn assembles_issue_mention_prompt_with_comment_context() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-issue-mention-home-root");
      let _home_guard = HomeGuard::set(&home);

      let project_workdir = unique_temp_dir("adapter-issue-mention-workdir");
      let project = ProjectRepository::create(ProjectModel::new(
        "Mention Project".to_string(),
        String::new(),
        vec![project_workdir.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let commenter = EmployeeRepository::create(EmployeeModel {
        name: "Comment Author".to_string(),
        kind: EmployeeKind::Agent,
        role: EmployeeRole::Staff,
        title: "Comment Author".to_string(),
        ..Default::default()
      })
      .await
      .expect("commenter should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-77".to_string(),
        title: "Mention wake-up".to_string(),
        description: "Prompt should include comment context.".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::InProgress,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let comment = IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
        issue.id.clone(),
        "Please pick up the next backend step.".to_string(),
        vec![],
        commenter.id.clone(),
        None,
      ))
      .await
      .expect("comment should be created");

      let run = RunRepository::create(RunModel::new(
        employee_id,
        RunTrigger::IssueMention { issue_id: issue.id.clone(), comment_id: comment.id.clone() },
      ))
      .await
      .expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("Done".to_string())]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let request_text: String = run.turns[0]
        .steps
        .iter()
        .find_map(|step| {
          step.request.contents.iter().find_map(|content| match content {
            TurnStepContent::Text(text) => Some(text.text.clone()),
            _ => None,
          })
        })
        .expect("user prompt should be recorded");

      assert!(request_text.contains("Trigger: issue_mention"), "request text: {request_text}");
      assert!(request_text.contains("Issue Identifier: BLP-77"), "request text: {request_text}");
      assert!(request_text.contains("Issue Title: Mention wake-up"), "request text: {request_text}");
      assert!(
        request_text.contains(&format!("Triggering Comment ID: {}", comment.id.uuid())),
        "request text: {request_text}"
      );
      assert!(request_text.contains("Comment Author: Comment Author"), "request text: {request_text}");
      assert!(
        request_text.contains("Triggering Comment:\nPlease pick up the next backend step."),
        "request text: {request_text}"
      );
    });
  }

  #[test]
  fn normal_prompts_still_inject_memory_markdown_when_present() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let agent_home = unique_temp_dir("adapter-memory-prompt-home");
      fs::write(agent_home.join("MEMORY.md"), "- statement: Keep comments precise.").expect("memory file");

      let prompt = PromptAssemblyInput {
        agent_home,
        project_home: None,
        project_workdirs: vec![],
        employee_id: "employee-1".to_string(),
        api_url: "http://127.0.0.1:3100".to_string(),
        operating_system: "macos".to_string(),
        heartbeat_prompt: String::new(),
        available_skills: vec![],
        injected_skill_stack: vec![],
        trigger: RunTrigger::Manual,
        dreaming_date: None,
        daily_memory_content: None,
        prior_memory_content: None,
        issue_id: None,
        issue_identifier: None,
        issue_title: None,
        issue_description: None,
        issue_status: None,
        issue_priority: None,
        trigger_comment: None,
        trigger_commenter: None,
        available_mcp_servers: vec![],
      }
      .build();

      assert!(prompt.system_prompt.contains("## MEMORY.md\n- statement: Keep comments precise."));
    });
  }

  #[test]
  fn dreaming_run_distills_memory_and_rewrites_memory_file_atomically() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let agent_home = shared::paths::employee_home(&employee_id.uuid().to_string());
      let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
      fs::create_dir_all(agent_home.join("memory")).expect("memory dir");
      fs::write(agent_home.join("memory").join(format!("{today}.md")), "- learned: mirror meaningful updates").expect("daily file");

      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Dreaming)).await.expect("run should create");
      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text(
        "- statement: Mirror meaningful user-facing updates in issue comments.\n  type: workflow\n  freshness: active\n  last_reinforced: 2026-04-08"
          .to_string(),
      )]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());

      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("dreaming run should complete");

      let memory = fs::read_to_string(agent_home.join("MEMORY.md")).expect("memory should exist");
      assert!(memory.contains("statement: Mirror meaningful user-facing updates in issue comments."));
      assert!(memory.contains("type: workflow"));
      assert!(memory.contains("freshness: active"));
      assert!(memory.contains("last_reinforced: 2026-04-08"));
    });
  }

  #[test]
  fn dreaming_run_skips_when_daily_memory_is_empty() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let agent_home = shared::paths::employee_home(&employee_id.uuid().to_string());
      fs::create_dir_all(agent_home.join("memory")).expect("memory dir");
      let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
      fs::write(agent_home.join("memory").join(format!("{today}.md")), "\n   \n").expect("daily file");
      fs::write(agent_home.join("MEMORY.md"), "- statement: Keep previous memory\n  type: workflow\n  freshness: active\n  last_reinforced: 2026-04-07").expect("existing memory");

      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Dreaming)).await.expect("run should create");
      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("Should not run".to_string())]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());

      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("dreaming run should complete");

      let memory = fs::read_to_string(agent_home.join("MEMORY.md")).expect("memory should exist");
      assert!(memory.contains("Keep previous memory"));
    });
  }

  #[test]
  fn dreaming_run_preserves_previous_memory_when_synthesis_is_invalid() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let agent_home = shared::paths::employee_home(&employee_id.uuid().to_string());
      fs::create_dir_all(agent_home.join("memory")).expect("memory dir");
      let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
      fs::write(agent_home.join("memory").join(format!("{today}.md")), "- learned: invalid output should preserve prior memory").expect("daily file");
      fs::write(agent_home.join("MEMORY.md"), "- statement: Preserve prior memory\n  type: constraint\n  freshness: active\n  last_reinforced: 2026-04-07").expect("existing memory");

      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Dreaming)).await.expect("run should create");
      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("not valid markdown memory".to_string())]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());

      assert!(runtime.execute_run(run.id.clone(), CancellationToken::new()).await.is_err());

      let memory = fs::read_to_string(agent_home.join("MEMORY.md")).expect("memory should exist");
      assert!(memory.contains("Preserve prior memory"));
    });
  }

  #[test]
  fn dreaming_run_prompt_includes_date_daily_memory_and_prior_memory() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let agent_home = shared::paths::employee_home(&employee_id.uuid().to_string());
      fs::create_dir_all(agent_home.join("memory")).expect("memory dir");
      let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
      fs::write(agent_home.join("memory").join(format!("{today}.md")), "- learned: reinforcement matters").expect("daily file");
      fs::write(agent_home.join("MEMORY.md"), "- statement: Existing reinforced item\n  type: insight\n  freshness: active\n  last_reinforced: 2026-04-07").expect("existing memory");

      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Dreaming)).await.expect("run should create");
      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text(
        "- statement: Existing reinforced item\n  type: insight\n  freshness: active\n  last_reinforced: 2026-04-08".to_string(),
      )]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());

      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("dreaming run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let request_text: String = run.turns[0]
        .steps
        .iter()
        .find_map(|step| step.request.contents.iter().find_map(|content| match content {
          TurnStepContent::Text(text) => Some(text.text.clone()),
          _ => None,
        }))
        .expect("request text should exist");

      assert!(request_text.contains("Trigger: dreaming"));
      assert!(request_text.contains("Current Date:"));
      assert!(request_text.contains("Today's daily memory:"));
      assert!(request_text.contains("Prior MEMORY.md:"));
      assert!(request_text.contains("reinforcement matters"));
      assert!(request_text.contains("Existing reinforced item"));
    });
  }

  #[test]
  fn executes_a_scripted_run_and_persists_tool_results_with_runtime_env() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "Runtime Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-59".to_string(),
        title: "Runtime execution".to_string(),
        description: "Continue your work".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should be created");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Inspect runtime env".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "printf",
              "args": ["'%s|%s|%s|%s|%s|%s' \"$AGENT_HOME\" \"$PROJECT_HOME\" \"$BLPRNT_EMPLOYEE_ID\" \"$BLPRNT_PROJECT_ID\" \"$BLPRNT_RUN_ID\" \"$BLPRNT_API_URL\""],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));
      assert_eq!(run.turns.len(), 1);

      let turn = &run.turns[0];
      let serialized = serde_json::to_string(&turn.steps).expect("steps should serialize");
      #[cfg(not(target_os = "macos"))]
      let expected_agent_home = shared::paths::employee_home(&employee_id.uuid().to_string()).to_string_lossy().to_string();
      #[cfg(not(target_os = "macos"))]
      let expected_project_home = shared::paths::project_home(&project.id.uuid().to_string()).to_string_lossy().to_string();
      assert!(serialized.contains("Run completed"));
      assert!(serialized.contains("tool-1"));

      #[cfg(target_os = "macos")]
      {
        assert!(serialized.contains("requires sandboxing"), "serialized steps: {serialized}");
        assert!(serialized.contains("sandbox-exec"), "serialized steps: {serialized}");
        assert!(serialized.contains("BLPRNT_API_URL"), "serialized steps: {serialized}");
      }

      #[cfg(not(target_os = "macos"))]
      {
        assert!(serialized.contains("http://127.0.0.1:3100"), "serialized steps: {serialized}");
        assert!(serialized.contains(&employee_id.uuid().to_string()), "serialized steps: {serialized}");
        assert!(serialized.contains(&project.id.uuid().to_string()), "serialized steps: {serialized}");
        assert!(serialized.contains(&run.id.uuid().to_string()), "serialized steps: {serialized}");
        assert!(serialized.contains(&expected_project_home), "serialized steps: {serialized}");
        assert!(serialized.contains(&expected_agent_home), "serialized steps: {serialized}");
      }
    });
  }

  #[test]
  fn enables_mcp_server_and_executes_dynamic_mcp_tool_via_rmcp() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-mcp-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "MCP Runtime Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-105".to_string(),
        title: "MCP runtime execution".to_string(),
        description: "Enable MCP and run a tool".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let mcp_stub = spawn_text_mcp_stub("/mcp").await;
      let server = McpServerRepository::create(McpServerModel {
        display_name: "Docs MCP".to_string(),
        description: "Stub docs server".to_string(),
        transport: "streamable_http".to_string(),
        endpoint_url: format!("{}/mcp", mcp_stub.base_url),
        auth_state: shared::tools::McpServerAuthState::Connected,
        auth_summary: None,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      })
      .await
      .expect("mcp server should create");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      RunEnabledMcpServerRepository::enable(run.id.clone(), server.id.clone())
        .await
        .expect("run should enable mcp server");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Use enabled MCP server".to_string(),
          ToolCallSpec {
            tool_use_id: "mcp-call-1".to_string(),
            tool_id: ToolId::Mcp(format!("mcp__{}__lookup_doc", server.id.uuid())),
            input: serde_json::json!({ "query": "rmcp" }),
          },
        ),
        ScriptedProviderReply::final_text("MCP run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("mcp__"), "serialized steps: {serialized}");
      assert!(serialized.contains("lookup_doc"), "serialized steps: {serialized}");
      assert!(serialized.contains("structuredContent"), "serialized steps: {serialized}");
      assert!(serialized.contains("stub tool ok"), "serialized steps: {serialized}");

      let requests = mcp_stub.requests.lock().await.clone();
      let request_dump = serde_json::to_string(&requests).expect("requests should serialize");
      assert!(request_dump.contains("tools/list"), "request dump: {request_dump}");
      assert!(request_dump.contains("tools/call"), "request dump: {request_dump}");
      assert!(request_dump.contains("lookup_doc"), "request dump: {request_dump}");
    });
  }

  #[test]
  fn auth_required_mcp_server_returns_reconnect_error_without_calling_rmcp() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-mcp-auth-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "MCP Auth Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-106".to_string(),
        title: "MCP auth required".to_string(),
        description: "Enabled server still needs auth".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let mcp_stub = spawn_text_mcp_stub("/mcp").await;
      let server = McpServerRepository::create(McpServerModel {
        display_name: "OAuth MCP".to_string(),
        description: "Needs auth".to_string(),
        transport: "streamable_http".to_string(),
        endpoint_url: format!("{}/mcp", mcp_stub.base_url),
        auth_state: shared::tools::McpServerAuthState::AuthRequired,
        auth_summary: Some("Connect account".to_string()),
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      })
      .await
      .expect("mcp server should create");

      crate::mcp::store_mcp_server_oauth_token(
        &server.id,
        &crate::mcp::StoredMcpOauthToken {
          access_token: "pending-token".to_string(),
          refresh_token: Some("pending-refresh".to_string()),
          expires_at_ms: None,
          token_type: Some("Bearer".to_string()),
          scopes: vec!["tools:read".to_string()],
          authorization_url: Some("https://example.com/connect/mcp".to_string()),
        },
      )
      .await
      .expect("oauth token should store");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      RunEnabledMcpServerRepository::enable(run.id.clone(), server.id.clone())
        .await
        .expect("run should enable mcp server");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Try auth-blocked MCP tool".to_string(),
          ToolCallSpec {
            tool_use_id: "mcp-call-auth".to_string(),
            tool_id: ToolId::Mcp(format!("mcp__{}__lookup_doc", server.id.uuid())),
            input: serde_json::json!({ "query": "oauth" }),
          },
        ),
        ScriptedProviderReply::final_text("Handled auth required".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("requires OAuth authorization"), "serialized steps: {serialized}");
      assert!(serialized.contains("authorization_url=https://example.com/connect/mcp"), "serialized steps: {serialized}");

      let requests = mcp_stub.requests.lock().await.clone();
      assert!(requests.is_empty(), "auth-required MCP server should not open rmcp session before auth");
    });
  }

  #[test]
  fn expired_mcp_oauth_token_marks_server_reconnect_required_without_opening_rmcp() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-mcp-expired-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "MCP Expired Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-107".to_string(),
        title: "MCP token expired".to_string(),
        description: "Stored token is expired".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let mcp_stub = spawn_text_mcp_stub("/mcp").await;
      let server = McpServerRepository::create(McpServerModel {
        display_name: "Expired OAuth MCP".to_string(),
        description: "Expired token".to_string(),
        transport: "streamable_http".to_string(),
        endpoint_url: format!("{}/mcp", mcp_stub.base_url),
        auth_state: shared::tools::McpServerAuthState::Connected,
        auth_summary: None,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      })
      .await
      .expect("mcp server should create");

      crate::mcp::store_mcp_server_oauth_token(
        &server.id,
        &crate::mcp::StoredMcpOauthToken {
          access_token: "expired-token".to_string(),
          refresh_token: Some("refresh-token".to_string()),
          expires_at_ms: Some(1),
          token_type: Some("Bearer".to_string()),
          scopes: vec!["tools:read".to_string()],
          authorization_url: Some("https://example.com/reconnect/mcp".to_string()),
        },
      )
      .await
      .expect("oauth token should store");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      RunEnabledMcpServerRepository::enable(run.id.clone(), server.id.clone())
        .await
        .expect("run should enable mcp server");

      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("Should not run".to_string())]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let error = runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect_err("run should fail");
      let serialized = format!("{error:#}");
      assert!(serialized.contains("stored OAuth token expired"), "error: {serialized}");

      let updated = McpServerRepository::get(server.id.clone()).await.expect("server should reload");
      assert_eq!(updated.auth_state, shared::tools::McpServerAuthState::ReconnectRequired);
      assert!(updated.auth_summary.unwrap_or_default().contains("Reconnect required"));

      let requests = mcp_stub.requests.lock().await.clone();
      assert!(requests.is_empty(), "expired token should prevent rmcp session establishment");
    });
  }

  #[test]
  fn unauthorized_mcp_session_marks_server_reconnect_required() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-mcp-unauthorized-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "MCP Unauthorized Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-108".to_string(),
        title: "MCP token rejected".to_string(),
        description: "Stored token rejected by server".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let mcp_stub = spawn_unauthorized_mcp_stub("/mcp").await;
      let server = McpServerRepository::create(McpServerModel {
        display_name: "Unauthorized OAuth MCP".to_string(),
        description: "Unauthorized token".to_string(),
        transport: "streamable_http".to_string(),
        endpoint_url: format!("{}/mcp", mcp_stub.base_url),
        auth_state: shared::tools::McpServerAuthState::Connected,
        auth_summary: None,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
      })
      .await
      .expect("mcp server should create");

      crate::mcp::store_mcp_server_oauth_token(
        &server.id,
        &crate::mcp::StoredMcpOauthToken {
          access_token: "rejected-token".to_string(),
          refresh_token: Some("refresh-token".to_string()),
          expires_at_ms: None,
          token_type: Some("Bearer".to_string()),
          scopes: vec!["tools:read".to_string()],
          authorization_url: Some("https://example.com/reconnect/mcp".to_string()),
        },
      )
      .await
      .expect("oauth token should store");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      RunEnabledMcpServerRepository::enable(run.id.clone(), server.id.clone())
        .await
        .expect("run should enable mcp server");

      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("Should not run".to_string())]);
      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let error = runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect_err("run should fail");
      let serialized = format!("{error:#}").to_ascii_lowercase();
      assert!(serialized.contains("unauthorized") || serialized.contains("401"), "error: {serialized}");

      let updated = McpServerRepository::get(server.id.clone()).await.expect("server should reload");
      assert_eq!(updated.auth_state, shared::tools::McpServerAuthState::ReconnectRequired);
      assert!(updated.auth_summary.unwrap_or_default().contains("Reconnect required"));

      let requests = mcp_stub.requests.lock().await.clone();
      assert!(!requests.is_empty(), "unauthorized flow should attempt rmcp before marking reconnect required");
    });
  }

  #[test]
  #[ignore]
  fn shell_can_reach_loopback_http_endpoints() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_responses_stub("/health", vec![serde_json::json!({"ok": true})]).await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id.clone(), RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Call loopback http".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "curl",
              "args": [format!("{}/health", stub.base_url)],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let turn = &run.turns[0];
      let serialized = serde_json::to_string(&turn.steps).expect("steps should serialize");
      assert!(serialized.contains("\"ok\":true"), "serialized steps: {serialized}");
    });
  }

  #[test]
  fn apply_patch_can_write_inside_agent_home() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-agent-home-root");
      let _home_guard = HomeGuard::set(&home);

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let agent_home = home.join(".blprnt").join("employees").join(employee_id.uuid().to_string());
      fs::create_dir_all(&agent_home).expect("agent home should exist");
      fs::write(agent_home.join("AGENTS.md"), "before\n").expect("agents file");

      let run =
        RunRepository::create(RunModel::new(employee_id.clone(), RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch agent home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                agent_home.join("AGENTS.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(agent_home.join("AGENTS.md")).expect("agents file"), "after\n");
    });
  }

  #[test]
  fn apply_patch_can_write_inside_project_home() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-project-home-root");
      let _home_guard = HomeGuard::set(&home);

      let project_root = unique_temp_dir("adapter-project-workspace");
      let project = ProjectRepository::create(ProjectModel::new(
        "Runtime Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-60".to_string(),
        title: "Project plan patch".to_string(),
        description: "Write project plan".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let project_home = home.join(".blprnt").join("projects").join(project.id.uuid().to_string());
      fs::create_dir_all(project_home.join("memory")).expect("memory dir should exist");
      fs::write(project_home.join("memory").join("SUMMARY.md"), "before\n").expect("summary file");

      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch project home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                project_home.join("memory").join("SUMMARY.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(project_home.join("memory").join("SUMMARY.md")).expect("summary file"), "after\n");
    });
  }

  #[test]
  fn apply_patch_can_write_inside_unscoped_project_home_and_workspace() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-unscoped-project-write-root");
      let _home_guard = HomeGuard::set(&home);

      let scoped_project_workspace = unique_temp_dir("adapter-scoped-project-workspace");
      let scoped_project = ProjectRepository::create(ProjectModel::new(
        "Scoped Project".to_string(),
        String::new(),
        vec![scoped_project_workspace.to_string_lossy().to_string()],
      ))
      .await
      .expect("scoped project should be created");

      let target_project_workspace = unique_temp_dir("adapter-target-project-workspace");
      fs::write(target_project_workspace.join("main.rs"), "before\n").expect("workspace file");
      let target_project = ProjectRepository::create(ProjectModel::new(
        "Target Project".to_string(),
        String::new(),
        vec![target_project_workspace.to_string_lossy().to_string()],
      ))
      .await
      .expect("target project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-61".to_string(),
        title: "Cross-project write".to_string(),
        description: "Write outside scoped project".to_string(),
        project: Some(scoped_project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let target_project_home = home.join(".blprnt").join("projects").join(target_project.id.uuid().to_string());
      fs::create_dir_all(target_project_home.join("memory")).expect("memory dir should exist");
      fs::write(target_project_home.join("memory").join("SUMMARY.md"), "before\n").expect("summary file");

      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch target project workspace".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                target_project_workspace.join("main.rs").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "Patch target project home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-2".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                target_project_home.join("memory").join("SUMMARY.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
      assert_eq!(
        fs::read_to_string(target_project_home.join("memory").join("SUMMARY.md")).expect("summary file"),
        "after\n"
      );
    });
  }

  #[test]
  fn manager_apply_patch_can_write_inside_indirect_report_home() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-manager-tree-write-root");
      let _home_guard = HomeGuard::set(&home);

      let top_manager_id =
        create_employee_with_role(Provider::Mock, "test-model", "runtime heartbeat", EmployeeRole::Manager).await;
      let child_manager_id =
        create_employee_with_role(Provider::Mock, "test-model", "runtime heartbeat", EmployeeRole::Manager).await;
      let staff_id = create_employee(Provider::Mock, "runtime heartbeat").await;

      EmployeeRepository::update(
        child_manager_id.clone(),
        EmployeePatch { reports_to: Some(Some(top_manager_id.clone())), ..Default::default() },
      )
      .await
      .expect("child manager should report to top manager");
      EmployeeRepository::update(
        staff_id.clone(),
        EmployeePatch { reports_to: Some(Some(child_manager_id.clone())), ..Default::default() },
      )
      .await
      .expect("staff should report to child manager");

      let staff_home = home.join(".blprnt").join("employees").join(staff_id.uuid().to_string());
      fs::create_dir_all(&staff_home).expect("staff home should exist");
      fs::write(staff_home.join("HEARTBEAT.md"), "before\n").expect("heartbeat file");

      let run =
        RunRepository::create(RunModel::new(top_manager_id, RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch indirect report home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                staff_home.join("HEARTBEAT.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(staff_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
    });
  }

  #[test]
  fn shell_can_write_inside_unscoped_project_home_and_workspace() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-shell-unscoped-project-write-root");
      let _home_guard = HomeGuard::set(&home);

      let scoped_project_workspace = unique_temp_dir("adapter-shell-scoped-project-workspace");
      let scoped_project = ProjectRepository::create(ProjectModel::new(
        "Scoped Shell Project".to_string(),
        String::new(),
        vec![scoped_project_workspace.to_string_lossy().to_string()],
      ))
      .await
      .expect("scoped project should be created");

      let target_project_workspace = unique_temp_dir("adapter-shell-target-project-workspace");
      fs::write(target_project_workspace.join("main.rs"), "before\n").expect("workspace file");
      let target_project = ProjectRepository::create(ProjectModel::new(
        "Target Shell Project".to_string(),
        String::new(),
        vec![target_project_workspace.to_string_lossy().to_string()],
      ))
      .await
      .expect("target project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-62".to_string(),
        title: "Cross-project shell write".to_string(),
        description: "Shell writes outside scoped project".to_string(),
        project: Some(scoped_project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::High,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let target_project_home = home.join(".blprnt").join("projects").join(target_project.id.uuid().to_string());
      fs::create_dir_all(target_project_home.join("memory")).expect("memory dir should exist");
      fs::write(target_project_home.join("memory").join("SUMMARY.md"), "before\n").expect("summary file");

      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Shell write target project".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "sh",
              "args": ["-c", format!("printf 'after\\n' > '{}' && printf 'after\\n' > '{}'", target_project_workspace.join("main.rs").display(), target_project_home.join("memory").join("SUMMARY.md").display())],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let result = runtime.execute_run(run.id.clone(), CancellationToken::new()).await;

      #[cfg(target_os = "macos")]
      {
        result.expect("run should finish after recording the tool failure");
        let run = RunRepository::get(run.id).await.expect("run should load");
        let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
        assert!(
          serialized.contains("requires sandboxing") || serialized.contains("Run completed"),
          "serialized steps: {serialized}"
        );
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "before\n");
        assert_eq!(fs::read_to_string(target_project_home.join("memory").join("SUMMARY.md")).expect("summary file"), "before\n");
      }

      #[cfg(not(target_os = "macos"))]
      {
        result.expect("run should complete");
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
        assert_eq!(fs::read_to_string(target_project_home.join("memory").join("SUMMARY.md")).expect("summary file"), "after\n");
      }
    });
  }

  #[test]
  fn ceo_apply_patch_can_write_inside_any_employee_and_project_home() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-ceo-wide-write-root");
      let _home_guard = HomeGuard::set(&home);

      let ceo_id = create_ceo_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_home = home.join(".blprnt").join("employees").join(target_employee_id.uuid().to_string());
      fs::create_dir_all(&target_employee_home).expect("employee home should exist");
      fs::write(target_employee_home.join("HEARTBEAT.md"), "before\n").expect("heartbeat file");

      let target_project_workspace = unique_temp_dir("ceo-target-project-workspace");
      fs::write(target_project_workspace.join("main.rs"), "before\n").expect("workspace file");
      let project = ProjectRepository::create(ProjectModel::new(
        "CEO Project".to_string(),
        String::new(),
        vec![target_project_workspace.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");
      let project_home = home.join(".blprnt").join("projects").join(project.id.uuid().to_string());
      fs::create_dir_all(&project_home).expect("project home should exist");
      fs::write(project_home.join("SUMMARY.md"), "before\n").expect("summary file");

      let run = RunRepository::create(RunModel::new(ceo_id, RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch employee home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                target_employee_home.join("HEARTBEAT.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "Patch project workspace".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-2".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                target_project_workspace.join("main.rs").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "Patch project home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-3".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "diff": format!(
                "*** Begin Patch\n*** Update File: {}\n@@\n-before\n+after\n*** End Patch",
                project_home.join("SUMMARY.md").display()
              )
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let result = runtime.execute_run(run.id.clone(), CancellationToken::new()).await;

      #[cfg(target_os = "macos")]
      {
        result.expect("run should complete");
        assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
        assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "after\n");
      }

      #[cfg(not(target_os = "macos"))]
      {
        result.expect("run should complete");
        assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
        assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "after\n");
      }
    });
  }

  #[test]
  fn ceo_shell_can_write_inside_any_employee_and_project_home() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-ceo-wide-shell-root");
      let _home_guard = HomeGuard::set(&home);

      let ceo_id = create_ceo_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_home = home.join(".blprnt").join("employees").join(target_employee_id.uuid().to_string());
      fs::create_dir_all(&target_employee_home).expect("employee home should exist");
      fs::write(target_employee_home.join("HEARTBEAT.md"), "before\n").expect("heartbeat file");

      let target_project_workspace = unique_temp_dir("ceo-shell-project-workspace");
      fs::write(target_project_workspace.join("main.rs"), "before\n").expect("workspace file");
      let project = ProjectRepository::create(ProjectModel::new(
        "CEO Shell Project".to_string(),
        String::new(),
        vec![target_project_workspace.to_string_lossy().to_string()],
      ))
        .await
        .expect("project should be created");
      let project_home = home.join(".blprnt").join("projects").join(project.id.uuid().to_string());
      fs::create_dir_all(&project_home).expect("project home should exist");
      fs::write(project_home.join("SUMMARY.md"), "before\n").expect("summary file");

      let run = RunRepository::create(RunModel::new(ceo_id, RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Shell write across scopes".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "sh",
              "args": [format!("-c"), format!("printf 'after\\n' > '{}' && printf 'after\\n' > '{}' && printf 'after\\n' > '{}'", target_employee_home.join("HEARTBEAT.md").display(), target_project_workspace.join("main.rs").display(), project_home.join("SUMMARY.md").display())],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let result = runtime.execute_run(run.id.clone(), CancellationToken::new()).await;

      #[cfg(target_os = "macos")]
      {
        result.expect("run should finish after recording the tool failure");
        let run = RunRepository::get(run.id).await.expect("run should load");
        let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
        assert!(serialized.contains("requires sandboxing"), "serialized steps: {serialized}");
        assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "before\n");
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "before\n");
        assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "before\n");
      }

      #[cfg(not(target_os = "macos"))]
      {
        result.expect("run should complete");
        assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
        assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
        assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "after\n");
      }
    });
  }

  #[test]
  fn ceo_shell_can_list_employee_and_project_home_roots() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-ceo-root-list-root");
      let _home_guard = HomeGuard::set(&home);

      let ceo_id = create_ceo_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let target_employee_home = home.join(".blprnt").join("employees").join(target_employee_id.uuid().to_string());
      fs::create_dir_all(&target_employee_home).expect("employee home should exist");

      let project = ProjectRepository::create(ProjectModel::new("CEO Root Project".to_string(), String::new(), vec![]))
        .await
        .expect("project should be created");
      let project_home = home.join(".blprnt").join("projects").join(project.id.uuid().to_string());
      fs::create_dir_all(&project_home).expect("project home should exist");

      let run = RunRepository::create(RunModel::new(ceo_id, RunTrigger::Manual)).await.expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "List employee root".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "sh",
              "args": ["-c", format!("ls '{}'", home.join(".blprnt").join("employees").display())],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "List project root".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-2".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
              "command": "sh",
              "args": ["-c", format!("ls '{}'", home.join(".blprnt").join("projects").display())],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let result = runtime.execute_run(run.id.clone(), CancellationToken::new()).await;

      #[cfg(target_os = "macos")]
      {
        result.expect("run should finish after recording the tool failure");
        let run = RunRepository::get(run.id).await.expect("run should load");
        let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
        assert!(serialized.contains("requires sandboxing"));
        assert!(!serialized.contains(&target_employee_id.uuid().to_string()));
        assert!(!serialized.contains(&project.id.uuid().to_string()));
      }

      #[cfg(not(target_os = "macos"))]
      {
        result.expect("run should complete");
        let run = RunRepository::get(run.id).await.expect("run should load");
        let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
        assert!(serialized.contains(&target_employee_id.uuid().to_string()));
        assert!(serialized.contains(&project.id.uuid().to_string()));
      }
    });
  }

  #[test]
  fn marks_run_failed_when_provider_credentials_are_missing() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Codex, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime =
        AdapterRuntime::new_for_tests(ScriptedProviderFactory::default(), "http://127.0.0.1:3100".to_string());
      let error = runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect_err("run should fail");
      assert!(error.to_string().contains("not configured") || error.to_string().contains("missing credentials"));

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Failed(_)));
    });
  }

  #[test]
  fn marks_run_cancelled_when_cancelled_before_first_provider_reply() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime =
        AdapterRuntime::new_for_tests(ScriptedProviderFactory::default(), "http://127.0.0.1:3100".to_string());
      let cancel_token = CancellationToken::new();
      cancel_token.cancel();

      let error = runtime.execute_run(run.id.clone(), cancel_token).await.expect_err("run should be cancelled");
      assert!(
        error.to_string().contains("run cancelled") || error.to_string().contains("command cancelled"),
        "unexpected cancellation error: {error:?}"
      );

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Cancelled));
      assert!(run.completed_at.is_some(), "cancelled runs should be terminal");
      assert_eq!(run.turns.len(), 1);
      assert!(
        !matches!(run.turns[0].steps.last().map(|step| &step.status), Some(TurnStepStatus::Failed)),
        "cancellation should not rewrite the last step as failed"
      );
    });
  }

  #[test]
  fn marks_run_cancelled_after_persisting_tool_results() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-cancel-project-root");
      let project = ProjectRepository::create(ProjectModel::new(
        "Cancellation Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-61".to_string(),
        title: "Cancellation execution".to_string(),
        description: "Continue your work".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::Medium,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::IssueAssignment { issue_id: issue.id.clone() }))
          .await
          .expect("run should be created");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Inspect runtime env".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::Shell,
            input:       serde_json::json!({
            "command": "sleep",
            "args": ["5"],
              "timeout": 5
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let cancel_token = CancellationToken::new();
      let delayed_cancel: JoinHandle<()> = tokio::spawn({
        let cancel_token = cancel_token.clone();
        async move {
          sleep(Duration::from_millis(10)).await;
          cancel_token.cancel();
        }
      });

      let error = runtime.execute_run(run.id.clone(), cancel_token).await.expect_err("run should be cancelled");
      delayed_cancel.await.expect("cancel task should complete");
      assert!(
        error.to_string().contains("run cancelled")
          || error.to_string().contains("cancelled"),
        "unexpected cancellation error: {error}"
      );

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Cancelled));
      assert!(run.completed_at.is_some(), "cancelled runs should be terminal");
      assert_eq!(run.turns.len(), 1);
      assert!(
        run.turns[0]
          .steps
          .iter()
          .flat_map(|step| step.request.contents.iter().chain(step.response.contents.iter()))
          .any(|content| matches!(content, TurnStepContent::ToolResult(result) if result.tool_use_id == "tool-1")),
        "tool results should remain inspectable after cancellation"
      );
      assert!(
        !run.turns[0].steps.iter().any(|step| matches!(step.status, TurnStepStatus::Failed)),
        "cancellation should keep completed steps inspectable"
      );
    });
  }

  #[test]
  fn cancels_in_progress_shell_tool_promptly() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let project_root = unique_temp_dir("adapter-cancel-active-tool");
      let project = ProjectRepository::create(ProjectModel::new(
        "Cancellation Project".to_string(),
        String::new(),
        vec![project_root.to_string_lossy().to_string()],
      ))
      .await
      .expect("project should be created");

      let issue = IssueRepository::create(IssueModel {
        identifier: "BLP-62".to_string(),
        title: "Cancellation while tool is running".to_string(),
        description: "Continue your work".to_string(),
        project: Some(project.id.clone()),
        status: IssueStatus::Todo,
        priority: IssuePriority::Medium,
        ..Default::default()
      })
      .await
      .expect("issue should be created");

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::IssueAssignment { issue_id: issue.id.clone() }))
          .await
          .expect("run should be created");

      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::tool_call(
        "Inspect runtime env".to_string(),
        ToolCallSpec {
          tool_use_id: "tool-1".to_string(),
          tool_id:     ToolId::Shell,
          input:       serde_json::json!({
            "command": "sleep",
            "args": ["5"],
            "timeout": 10
          }),
        },
      )]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      let cancel_token = CancellationToken::new();
      let delayed_cancel: JoinHandle<()> = tokio::spawn({
        let cancel_token = cancel_token.clone();
        async move {
          sleep(Duration::from_millis(100)).await;
          cancel_token.cancel();
        }
      });

      let started_at = Instant::now();
      let error = runtime.execute_run(run.id.clone(), cancel_token).await.expect_err("run should be cancelled");
      let elapsed = started_at.elapsed();

      delayed_cancel.await.expect("cancel task should complete");
      assert!(
        error.to_string().contains("run cancelled")
          || error.to_string().contains("cancelled")
          || error.to_string().contains("scripted provider exhausted"),
        "unexpected cancellation error: {error}"
      );
      assert!(
        elapsed < Duration::from_secs(2),
        "cancellation should interrupt a running shell tool instead of waiting for it to finish: {elapsed:?}"
      );

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Cancelled));
      assert!(run.completed_at.is_some(), "cancelled runs should be terminal");
    });
  }

  #[test]
  fn fails_run_when_provider_returns_an_empty_reply() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime = AdapterRuntime::new_for_tests(EmptyReplyProviderFactory, "http://127.0.0.1:3100".to_string());
      let error = runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect_err("run should fail");
      assert!(error.to_string().contains("empty provider reply"));

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Failed(_)));
      assert_eq!(run.turns.len(), 1);
      assert_eq!(run.turns[0].steps.len(), 1, "empty replies must not create an extra step");
      assert_eq!(run.turns[0].steps[0].request.role, persistence::prelude::TurnStepRole::User);
      assert_eq!(run.turns[0].steps[0].response.role, persistence::prelude::TurnStepRole::Assistant);
      assert!(matches!(run.turns[0].steps[0].status, TurnStepStatus::Failed));
    });
  }

  #[test]
  fn completes_run_when_provider_returns_final_text_and_marks_assistant_step_completed() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime = AdapterRuntime::new_for_tests(
        ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("Run completed".to_string())]),
        "http://127.0.0.1:3100".to_string(),
      );
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));
      assert_eq!(run.turns.len(), 1);
      assert_eq!(run.turns[0].steps.len(), 1, "final assistant text should complete the initial request step");
      assert_eq!(run.turns[0].steps[0].request.role, persistence::prelude::TurnStepRole::User);
      assert_eq!(run.turns[0].steps[0].response.role, persistence::prelude::TurnStepRole::Assistant);
      assert!(matches!(run.turns[0].steps[0].status, TurnStepStatus::Completed));
      assert!(
        run.turns[0].steps[0]
          .response
          .contents
          .iter()
          .any(|content| matches!(content, TurnStepContent::Text(text) if text.text == "Run completed"))
      );
    });
  }

  #[test]
  fn executes_openai_provider_run_and_round_trips_tool_results() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_responses_stub(
        "/responses",
        vec![
          serde_json::json!({
            "model": "gpt-5-test",
            "status": "completed",
            "usage": {
              "input_tokens": 11,
              "output_tokens": 7,
              "total_tokens": 18,
              "estimated_cost_usd": 0.0018
            },
            "output": [
              {
                "type": "function_call",
                "call_id": "call-1",
                "name": "shell",
                "arguments": "{\"command\":\"printf\",\"args\":[\"runtime-ok\"],\"timeout\":5}"
              }
            ]
          }),
          serde_json::json!({
            "model": "gpt-5-test",
            "status": "completed",
            "usage": {
              "input_tokens": 19,
              "output_tokens": 5,
              "total_tokens": 24,
              "estimated_cost_usd": 0.0024
            },
            "output": [
              {
                "type": "message",
                "content": [
                  {
                    "type": "output_text",
                    "text": "OpenAI runtime completed"
                  }
                ]
              }
            ]
          }),
        ],
      )
      .await;

      let _provider = upsert_provider_credentials(Provider::OpenAi, stub.base_url.clone(), "test-openai-key").await;
      let employee_id = create_employee_with_slug(Provider::OpenAi, "gpt-5-test", "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      AdapterRuntime::new()
        .execute_run(run.id.clone(), CancellationToken::new())
        .await
        .expect("run should complete through the openai provider path");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));

      let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("OpenAI runtime completed"));
      assert!(serialized.contains("call-1"));
      assert!(serialized.contains("runtime-ok"));
      assert_eq!(run.turns[0].steps[0].usage.provider, Some(Provider::OpenAi));
      assert_eq!(run.turns[0].steps[0].usage.model.as_deref(), Some("gpt-5-test"));
      assert_eq!(run.turns[0].steps[0].usage.total_tokens, Some(18));
      assert_eq!(run.turns[0].steps[1].usage.total_tokens, Some(24));
      assert_eq!(run.turns[0].usage.total_tokens, Some(42));
      assert_eq!(run.usage.as_ref().unwrap().total_tokens, Some(42));
      assert_eq!(run.usage.as_ref().unwrap().estimated_cost_usd, Some(0.0042));
      assert!(!run.usage.as_ref().unwrap().has_unavailable_token_data);
      assert!(!run.usage.as_ref().unwrap().has_unavailable_cost_data);

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 2, "openai loop should issue an initial request and one tool-result follow-up");
      assert_eq!(requests[0]["model"], "gpt-5-test");
      assert!(
        requests[0]["tools"].as_array().expect("tools should be an array").iter().any(|tool| tool["name"] == "shell"),
        "openai requests should expose the runtime shell tool"
      );
      assert!(
        requests[1].to_string().contains("function_call_output"),
        "tool results should be sent back as function_call_output items"
      );
    });
  }

  #[test]
  fn executes_anthropic_provider_run_and_round_trips_tool_results() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_responses_stub(
        "/v1/messages",
        vec![
          serde_json::json!({
            "model": "claude-test",
            "usage": {
              "input_tokens": 13,
              "output_tokens": 9,
              "estimated_cost_usd": 0.0031
            },
            "content": [
              {
                "type": "tool_use",
                "id": "tool-1",
                "name": "shell",
                "input": {
                  "command": "printf",
                  "args": ["anthropic-ok"],
                  "timeout": 5
                }
              }
            ],
            "stop_reason": "tool_use"
          }),
          serde_json::json!({
            "model": "claude-test",
            "usage": {
              "input_tokens": 5,
              "output_tokens": 4,
              "estimated_cost_usd": 0.0014
            },
            "content": [
              {
                "type": "thinking",
                "thinking": "Wrapped up"
              },
              {
                "type": "text",
                "text": "Anthropic runtime completed"
              }
            ],
            "stop_reason": "end_turn"
          }),
        ],
      )
      .await;

      let _provider =
        upsert_provider_credentials(Provider::Anthropic, stub.base_url.clone(), "test-anthropic-key").await;
      let employee_id = create_employee_with_slug(Provider::Anthropic, "claude-test", "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      AdapterRuntime::new()
        .execute_run(run.id.clone(), CancellationToken::new())
        .await
        .expect("run should complete through the anthropic provider path");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));

      let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("Anthropic runtime completed"));
      assert!(serialized.contains("tool-1"));
      assert!(serialized.contains("anthropic-ok"));
      assert_eq!(run.turns[0].steps[0].usage.provider, Some(Provider::Anthropic));
      assert_eq!(run.turns[0].steps[0].usage.total_tokens, Some(22));
      assert_eq!(run.turns[0].steps[1].usage.total_tokens, Some(9));
      assert_eq!(run.turns[0].usage.total_tokens, Some(31));
      assert_eq!(run.usage.as_ref().unwrap().total_tokens, Some(31));
      assert_option_f64_close(run.usage.as_ref().unwrap().estimated_cost_usd, 0.0045);

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 2, "anthropic loop should issue an initial request and one tool-result follow-up");
      assert_eq!(requests[0]["model"], "claude-test");
      assert!(
        requests[0]["tools"].as_array().expect("tools should be an array").iter().any(|tool| tool["name"] == "shell"),
        "anthropic requests should expose the runtime shell tool"
      );
      assert!(
        requests[1].to_string().contains("tool_result"),
        "tool results should be sent back as anthropic tool_result content"
      );
    });
  }

  #[test]
  fn executes_openrouter_provider_run_through_openai_compatible_mapping() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_responses_stub(
        "/responses",
        vec![serde_json::json!({
          "model": "openrouter/auto",
          "status": "completed",
          "output": [
            {
              "type": "message",
              "content": [
                {
                  "type": "output_text",
                  "text": "OpenRouter runtime completed"
                }
              ]
            }
          ]
        })],
      )
      .await;

      let _provider =
        upsert_provider_credentials(Provider::OpenRouter, stub.base_url.clone(), "test-openrouter-key").await;
      let employee_id = create_employee_with_slug(Provider::OpenRouter, "openrouter/auto", "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      AdapterRuntime::new()
        .execute_run(run.id.clone(), CancellationToken::new())
        .await
        .expect("run should complete through the openrouter provider path");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));

      let serialized = serde_json::to_string(&run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("OpenRouter runtime completed"));
      assert_eq!(run.turns[0].steps[0].usage.provider, Some(Provider::OpenRouter));
      assert_eq!(run.turns[0].steps[0].usage.model.as_deref(), Some("openrouter/auto"));
      assert!(run.turns[0].steps[0].usage.has_unavailable_token_data);
      assert!(run.turns[0].steps[0].usage.has_unavailable_cost_data);

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 1);
      assert_eq!(requests[0]["model"], "openrouter/auto");
      assert!(
        requests[0]["tools"].as_array().expect("tools should be an array").iter().any(|tool| tool["name"] == "shell"),
        "openrouter should reuse the openai-compatible tool schema mapping"
      );
    });
  }

  #[test]
  fn executes_claude_code_provider_run_through_anthropic_oauth_mapping() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_responses_stub(
        "/v1/messages",
        vec![serde_json::json!({
          "model": "claude-sonnet-test",
          "usage": {
            "input_tokens": 12,
            "output_tokens": 8,
            "estimated_cost_usd": 0.0021
          },
          "content": [
            {
              "type": "text",
              "text": "Claude Code runtime completed"
            }
          ],
          "stop_reason": "end_turn"
        })],
      )
      .await;

      let credentials =
        serde_json::to_string(&BlprntCredentials::OauthToken(OauthToken::Anthropic(AnthropicOauthToken {
          access_token:  "claude-access".to_string(),
          refresh_token: "claude-refresh".to_string(),
          expires_at_ms: 4_102_444_800_000,
        })))
        .expect("claude oauth credentials should serialize");

      let _provider = upsert_provider_credentials(Provider::ClaudeCode, stub.base_url.clone(), &credentials).await;
      let employee_id =
        create_employee_with_slug(Provider::ClaudeCode, "claude-sonnet-test", "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      AdapterRuntime::new()
        .execute_run(run.id.clone(), CancellationToken::new())
        .await
        .expect("run should complete through the claude code provider path");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));
      assert_eq!(run.turns[0].steps[0].usage.provider, Some(Provider::ClaudeCode));
      assert_eq!(run.turns[0].steps[0].usage.total_tokens, Some(20));
      assert_option_f64_close(run.usage.as_ref().unwrap().estimated_cost_usd, 0.0021);

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 1);
      assert_eq!(requests[0]["model"], "claude-sonnet-test");
      assert!(
        requests[0]["system"].to_string().contains("You are Claude Code, Anthropic's official CLI for Claude."),
        "claude code requests should prepend the claude cli system hint"
      );
      assert!(
        requests[0]["tools"].as_array().expect("tools should be an array").iter().any(|tool| tool["name"] == "shell"),
        "claude code requests should expose runtime tools"
      );
    });
  }

  #[test]
  fn executes_codex_provider_run_through_openai_oauth_mapping() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let stub = spawn_sse_stub(
        "/responses",
        vec![serde_json::json!({
          "body": concat!(
            "event: response.completed\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"model\":\"gpt-5-codex\",\"usage\":{\"input_tokens\":29,\"output_tokens\":17,\"total_tokens\":46,\"estimated_cost_usd\":0.0061}}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"output_index\":0,\"delta\":\"Codex \"}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"item_id\":\"msg_1\",\"output_index\":0,\"delta\":\"runtime completed\"}\n\n",
            "event: response.output_text.done\n",
            "data: {\"type\":\"response.output_text.done\",\"item_id\":\"msg_1\",\"output_index\":0,\"text\":\"Codex runtime completed\"}\n\n",
            "data: [DONE]\n\n"
          )
        })],
      )
      .await;

      let credentials = serde_json::to_string(&BlprntCredentials::OauthToken(OauthToken::OpenAi(OpenAiOauthToken {
        access_token: "codex-access".to_string(),
        refresh_token: "codex-refresh".to_string(),
        expires_at_ms: 4_102_444_800_000,
        account_id: Some("acct_codex".to_string()),
      })))
      .expect("codex oauth credentials should serialize");

      let _provider = upsert_provider_credentials(Provider::Codex, stub.base_url.clone(), &credentials).await;
      let employee_id = create_employee_with_slug(Provider::Codex, "gpt-5-codex", "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      AdapterRuntime::new()
        .execute_run(run.id.clone(), CancellationToken::new())
        .await
        .expect("run should complete through the codex provider path");

      let run = RunRepository::get(run.id).await.expect("run should load");
      assert!(matches!(run.status, RunStatus::Completed));
      assert_eq!(run.turns[0].steps[0].usage.provider, Some(Provider::Codex));
      assert_eq!(run.turns[0].steps[0].usage.total_tokens, Some(46));
      assert_eq!(run.usage.as_ref().unwrap().estimated_cost_usd, Some(0.0061));

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 1);
      assert_eq!(requests[0]["model"], "gpt-5-codex");
      assert_eq!(requests[0]["stream"], true);
      assert!(
        requests[0]["tools"].as_array().expect("tools should be an array").iter().any(|tool| tool["name"] == "shell"),
        "codex requests should expose runtime tools"
      );
    });
  }

  #[test]
  fn persists_streaming_assistant_output_while_provider_stream_is_in_flight() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime = AdapterRuntime::new_for_tests(
        StreamingTestProviderFactory { chunks: vec!["Hello", " world"], delay_ms: 150 },
        "http://127.0.0.1:3100".to_string(),
      );

      let handle = tokio::spawn({
        let run_id = run.id.clone();
        async move { runtime.execute_run(run_id, CancellationToken::new()).await }
      });

      sleep(Duration::from_millis(80)).await;

      let mid_run = RunRepository::get(run.id.clone()).await.expect("run should load");
      assert_eq!(mid_run.turns.len(), 1);
      let serialized = serde_json::to_string(&mid_run.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("Hello"));
      assert!(!serialized.contains("Hello world"));

      handle.await.expect("runtime task should join").expect("run should complete");

      let completed = RunRepository::get(run.id).await.expect("run should reload");
      let serialized = serde_json::to_string(&completed.turns[0].steps).expect("steps should serialize");
      assert!(serialized.contains("Hello world"));
    });
  }

  #[test]
  fn continuing_a_run_uses_prior_turn_history_in_provider_request() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::OpenAi, "runtime heartbeat").await;
      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Conversation))
        .await
        .expect("run should be created");
      let run = RunRepository::update(run.id, RunStatus::Completed).await.expect("run should be marked completed");

      let first_turn = persistence::prelude::TurnRepository::create(persistence::prelude::TurnModel {
        run_id: run.id.clone(),
        ..Default::default()
      })
      .await
      .expect("first turn should be created");
      persistence::prelude::TurnRepository::insert_step_content(
        first_turn.id.clone(),
        persistence::prelude::TurnStepSide::Request,
        persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
          text:       "Original user prompt".to_string(),
          signature:  None,
          visibility: persistence::prelude::ContentsVisibility::Full,
        }),
      )
      .await
      .expect("first request should be inserted");
      persistence::prelude::TurnRepository::insert_step_content(
        first_turn.id.clone(),
        persistence::prelude::TurnStepSide::Response,
        persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
          text:       "Original assistant reply".to_string(),
          signature:  None,
          visibility: persistence::prelude::ContentsVisibility::Full,
        }),
      )
      .await
      .expect("first response should be inserted");

      let second_turn = persistence::prelude::TurnRepository::create(persistence::prelude::TurnModel {
        run_id: run.id.clone(),
        ..Default::default()
      })
      .await
      .expect("second turn should be created");
      persistence::prelude::TurnRepository::insert_step_content(
        second_turn.id.clone(),
        persistence::prelude::TurnStepSide::Request,
        persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
          text:       "Follow-up question".to_string(),
          signature:  None,
          visibility: persistence::prelude::ContentsVisibility::Full,
        }),
      )
      .await
      .expect("follow-up request should be inserted");

      let stub = spawn_responses_stub(
        "/responses",
        vec![serde_json::json!({
          "status": "completed",
          "output": [
            {
              "type": "message",
              "content": [{ "type": "output_text", "text": "Follow-up answer" }]
            }
          ]
        })],
      )
      .await;
      let _provider = upsert_provider_credentials(Provider::OpenAi, stub.base_url.clone(), "test-openai-key").await;

      AdapterRuntime::new().execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should continue");

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 1);
      let input = requests[0]["input"].as_array().expect("input should be an array");
      assert!(input.iter().any(|item| item["role"] == "user" && item["content"][0]["text"] == "Original user prompt"));
      assert!(
        input
          .iter()
          .any(|item| item["role"] == "assistant" && item["content"][0]["text"] == "Original assistant reply")
      );
      assert!(input.iter().any(|item| item["role"] == "user" && item["content"][0]["text"] == "Follow-up question"));
    });
  }

  #[test]
  fn turn_reasoning_effort_overrides_employee_default_in_provider_request() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::OpenAi, "runtime heartbeat").await;
      let employee = EmployeeRepository::get(employee_id.clone()).await.expect("employee should load");
      EmployeeRepository::update(
        employee.id.clone(),
        EmployeePatch {
          runtime_config: Some(EmployeeRuntimeConfig {
            heartbeat_interval_sec: 1800,
            heartbeat_prompt:       "runtime heartbeat".to_string(),
            wake_on_demand:         true,
            timer_wakeups_enabled:  Some(true),
            dreams_enabled:         Some(false),
            max_concurrent_runs:    1,
            skill_stack:            None,
            reasoning_effort:       Some(ReasoningEffort::Low),
          }),
          ..Default::default()
        },
      )
      .await
      .expect("employee should update");

      let run = RunRepository::create(RunModel::new(employee_id, RunTrigger::Conversation))
        .await
        .expect("run should be created");
      let run = RunRepository::update(run.id, RunStatus::Completed).await.expect("run should be marked completed");

      let turn = persistence::prelude::TurnRepository::create(persistence::prelude::TurnModel {
        run_id: run.id.clone(),
        reasoning_effort: Some(ReasoningEffort::XHigh),
        ..Default::default()
      })
      .await
      .expect("turn should be created");
      persistence::prelude::TurnRepository::insert_step_content(
        turn.id.clone(),
        persistence::prelude::TurnStepSide::Request,
        persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
          text:       "Think carefully about this".to_string(),
          signature:  None,
          visibility: persistence::prelude::ContentsVisibility::Full,
        }),
      )
      .await
      .expect("request should be inserted");

      let stub = spawn_responses_stub(
        "/responses",
        vec![serde_json::json!({
          "status": "completed",
          "output": [
            {
              "type": "message",
              "content": [{ "type": "output_text", "text": "Done" }]
            }
          ]
        })],
      )
      .await;
      let _provider = upsert_provider_credentials(Provider::OpenAi, stub.base_url.clone(), "test-openai-key").await;

      AdapterRuntime::new().execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should execute");

      let requests = stub.requests.lock().await.clone();
      assert_eq!(requests.len(), 1);
      assert_eq!(requests[0]["reasoning"]["effort"], "xhigh");
    });
  }

  #[test]
  fn thinking_is_persisted_before_tool_calls_in_the_turn_timeline() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");

      let runtime = AdapterRuntime::new_for_tests(
        ScriptedProviderFactory::new(vec![
          ScriptedProviderReply::tool_call(
            "Need to inspect the project".to_string(),
            ToolCallSpec {
              tool_use_id: "tool-1".to_string(),
              tool_id:     ToolId::Shell,
              input:       serde_json::json!({
                "command": "pwd",
                "args": [],
                "timeout": 5
              }),
            },
          ),
          ScriptedProviderReply::final_text("Done".to_string()),
        ]),
        "http://127.0.0.1:3100".to_string(),
      );

      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      let run = RunRepository::get(run.id).await.expect("run should load");
      let response_contents = &run.turns[0].steps[0].response.contents;

      assert!(matches!(response_contents.first(), Some(TurnStepContent::Thinking(_))));
      assert!(matches!(response_contents.get(1), Some(TurnStepContent::ToolUse(_))));
    });
  }

  #[test]
  fn completes_run_end_to_end_through_api_event_to_coordinator_to_adapter() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;

      let employee_id = create_employee(Provider::Mock, "runtime heartbeat").await;
      let run =
        RunRepository::create(RunModel::new(employee_id, RunTrigger::Manual)).await.expect("run should be created");
      let run = RunRepository::update(run.id, RunStatus::Running).await.expect("run should be marked running");
      let runtime = Arc::new(AdapterRuntime::new_for_tests(
        ScriptedProviderFactory::new(vec![ScriptedProviderReply::final_text("End to end completed".to_string())]),
        "http://127.0.0.1:3100".to_string(),
      ));
      let adapter_task = tokio::spawn({
        let runtime = runtime.clone();
        async move { runtime.listen().await }
      });

      sleep(Duration::from_millis(100)).await;

      let (tx, rx) = tokio::sync::oneshot::channel();
      COORDINATOR_EVENTS
        .emit(CoordinatorEvent::StartRun {
          run_id:       run.id.clone(),
          cancel_token: CancellationToken::new(),
          tx:           Arc::new(AsyncMutex::new(Some(tx))),
        })
        .expect("coordinator event should emit");

      rx.await.expect("run result should arrive").expect("run execution should succeed");

      let mut completed = RunRepository::get(run.id.clone()).await.expect("run should load");
      for _ in 0..20 {
        if matches!(completed.status, RunStatus::Completed) {
          break;
        }
        sleep(Duration::from_millis(50)).await;
        completed = RunRepository::get(run.id.clone()).await.expect("run should reload");
      }

      assert!(matches!(completed.status, RunStatus::Completed));
      assert_eq!(completed.turns.len(), 1);
      assert!(
        serde_json::to_string(&completed.turns[0].steps)
          .expect("steps should serialize")
          .contains("End to end completed")
      );

      adapter_task.abort();
    });
  }
}
