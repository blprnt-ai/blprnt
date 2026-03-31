pub mod prompt;
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
  use persistence::prelude::EmployeeProviderConfig;
  use persistence::prelude::EmployeeRepository;
  use persistence::prelude::EmployeeRole;
  use persistence::prelude::EmployeeRuntimeConfig;
  use persistence::prelude::EmployeeSkillRef;
  use persistence::prelude::IssueModel;
  use persistence::prelude::IssuePriority;
  use persistence::prelude::IssueRepository;
  use persistence::prelude::IssueStatus;
  use persistence::prelude::ProjectModel;
  use persistence::prelude::ProjectRepository;
  use persistence::prelude::ProviderModel;
  use persistence::prelude::ProviderPatch;
  use persistence::prelude::ProviderRecord;
  use persistence::prelude::ProviderRepository;
  use persistence::prelude::RunModel;
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

  async fn reset_test_db() {
    SurrealConnection::reset().await.expect("test database should reset");
  }

  fn unique_temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", persistence::Uuid::new_v4()));
    fs::create_dir_all(&path).expect("failed to create temp dir");
    path
  }

  struct HomeGuard {
    previous_home: Option<String>,
  }

  impl HomeGuard {
    fn set(path: &std::path::Path) -> Self {
      let previous_home = std::env::var("HOME").ok();
      unsafe { std::env::set_var("HOME", path) };
      Self { previous_home }
    }
  }

  impl Drop for HomeGuard {
    fn drop(&mut self) {
      match &self.previous_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
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
        max_concurrent_runs:    1,
        skill_stack:            None,
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
      fs::write(agent_home.join("AGENTS.md"), "agent instructions").expect("agents file");
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
        project_workdirs:     vec![home.join("workspace-a"), home.join("workspace-b")],
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
        issue_id:             Some(issue_id.uuid()),
      }
      .build();

      let stub_index =
        prompt.system_prompt.find("You operate as a blprnt employee inside the blprnt system.").expect("stub prompt");
      let os_index = prompt.system_prompt.find("Operating system: macos").expect("os metadata");
      let project_home_index = prompt.system_prompt.find("PROJECT_HOME:").expect("project home metadata");
      let project_workdirs_index =
        prompt.system_prompt.find("Project Working Directories").expect("project workdirs metadata");
      let heartbeat_index = prompt.system_prompt.find("heartbeat instructions").expect("heartbeat prompt");
      let agents_index = prompt.system_prompt.find("agent instructions").expect("agents prompt");
      let runtime_index = prompt.system_prompt.find("runtime prompt").expect("runtime prompt");
      let available_skills_index = prompt.system_prompt.find("Available Runtime Skills").expect("available skills");
      let injected_skill_index = prompt.system_prompt.find("Employee Skill Stack: custom-skill").expect("skill stack");

      assert!(stub_index < os_index);
      assert!(os_index < project_home_index);
      assert!(project_home_index < project_workdirs_index);
      assert!(project_workdirs_index < heartbeat_index);
      assert!(heartbeat_index < agents_index);
      assert!(agents_index < runtime_index);
      assert!(runtime_index < available_skills_index);
      assert!(available_skills_index < injected_skill_index);
      assert!(prompt.system_prompt.contains("Use PROJECT_HOME for blprnt-managed project metadata only"));
      assert!(prompt.system_prompt.contains("PROJECT_HOME/plans stores plan documents"));
      assert!(prompt.system_prompt.contains("These are the actual project source/work directories"));
      assert!(prompt.user_prompt.contains("Use the blprnt API to continue your blprnt work."));
      assert!(prompt.user_prompt.contains("Trigger: issue_assignment"));
      assert!(prompt.user_prompt.contains(&issue_id_text));
      assert!(prompt.system_prompt.contains(skill_dir.join("SKILL.md").to_string_lossy().as_ref()));
      assert!(prompt.system_prompt.contains("# Custom Skill"));
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
      let expected_agent_home = std::env::var("HOME").unwrap() + &format!("/.blprnt/employees/{}", employee_id.uuid());
      let expected_project_home = std::env::var("HOME").unwrap() + &format!("/.blprnt/projects/{}", project.id.uuid());
      assert!(serialized.contains("Run completed"));
      assert!(serialized.contains("tool-1"));
      assert!(serialized.contains("http://127.0.0.1:3100"));
      assert!(serialized.contains(&employee_id.uuid().to_string()));
      assert!(serialized.contains(&project.id.uuid().to_string()));
      assert!(serialized.contains(&run.id.uuid().to_string()));
      assert!(serialized.contains(&expected_project_home));
      assert!(serialized.contains(&expected_agent_home));
    });
  }

  #[test]
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
              "diff": "*** Begin Patch\n*** Update File: AGENTS.md\n@@\n-before\n+after\n*** End Patch"
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
  fn apply_patch_can_write_inside_project_home_plans() {
    let _lock = test_lock();
    TEST_RUNTIME.block_on(async {
      reset_test_db().await;
      let home = unique_temp_dir("adapter-project-home-root");
      let _home_guard = HomeGuard::set(&home);

      let project_root = unique_temp_dir("adapter-project-workspace");
      let project = ProjectRepository::create(ProjectModel::new(
        "Runtime Project".to_string(),
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
      fs::create_dir_all(project_home.join("plans")).expect("plans dir should exist");
      fs::write(project_home.join("plans").join("plan.md"), "before\n").expect("plan file");

      let run = RunRepository::create(RunModel::new(
        employee_id.clone(),
        RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
      ))
      .await
      .expect("run should create");

      let provider = ScriptedProviderFactory::new(vec![
        ScriptedProviderReply::tool_call(
          "Patch project plans".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-1".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "workspace_index": 2,
              "diff": "*** Begin Patch\n*** Update File: plan.md\n@@\n-before\n+after\n*** End Patch"
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(project_home.join("plans").join("plan.md")).expect("plan file"), "after\n");
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
              "workspace_index": 1,
              "diff": "*** Begin Patch\n*** Update File: HEARTBEAT.md\n@@\n-before\n+after\n*** End Patch"
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "Patch project workspace".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-2".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "workspace_index": 2,
              "diff": "*** Begin Patch\n*** Update File: main.rs\n@@\n-before\n+after\n*** End Patch"
            }),
          },
        ),
        ScriptedProviderReply::tool_call(
          "Patch project home".to_string(),
          ToolCallSpec {
            tool_use_id: "tool-3".to_string(),
            tool_id:     ToolId::ApplyPatch,
            input:       serde_json::json!({
              "workspace_index": 3,
              "diff": "*** Begin Patch\n*** Update File: SUMMARY.md\n@@\n-before\n+after\n*** End Patch"
            }),
          },
        ),
        ScriptedProviderReply::final_text("Run completed".to_string()),
      ]);

      let runtime = AdapterRuntime::new_for_tests(provider, "http://127.0.0.1:3100".to_string());
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
      assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
      assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "after\n");
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
      runtime.execute_run(run.id.clone(), CancellationToken::new()).await.expect("run should complete");

      assert_eq!(fs::read_to_string(target_employee_home.join("HEARTBEAT.md")).expect("heartbeat file"), "after\n");
      assert_eq!(fs::read_to_string(target_project_workspace.join("main.rs")).expect("workspace file"), "after\n");
      assert_eq!(fs::read_to_string(project_home.join("SUMMARY.md")).expect("summary file"), "after\n");
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
      assert!(error.to_string().contains("run cancelled"));

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

      let provider = ScriptedProviderFactory::new(vec![ScriptedProviderReply::tool_call(
        "Inspect runtime env".to_string(),
        ToolCallSpec {
          tool_use_id: "tool-1".to_string(),
          tool_id:     ToolId::Shell,
          input:       serde_json::json!({
            "command": "sleep",
            "args": ["1"],
            "timeout": 5
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

      let error = runtime.execute_run(run.id.clone(), cancel_token).await.expect_err("run should be cancelled");
      delayed_cancel.await.expect("cancel task should complete");
      assert!(error.to_string().contains("run cancelled"));

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
            "status": "completed",
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
            "status": "completed",
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
