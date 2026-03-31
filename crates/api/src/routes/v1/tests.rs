use std::env;
use std::fs;
use std::sync::LazyLock;
use std::sync::Mutex;

use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use chrono::Local;
use events::API_EVENTS;
use events::ApiEvent;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ListIssuesParams;
use persistence::prelude::ProjectModel;
use persistence::prelude::ProjectRepository;
use persistence::prelude::ProviderModel;
use persistence::prelude::ProviderRepository;
use persistence::prelude::RunFilter;
use persistence::prelude::RunModel;
use persistence::prelude::RunRepository;
use persistence::prelude::RunTrigger;
use serde_json::Value;
use shared::agent::Provider;
use tempfile::TempDir;
use tower::ServiceExt;

static ENV_LOCK: Mutex<()> = Mutex::new(());
static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
  tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("failed to create test runtime")
});

struct HomeGuard {
  previous_home: Option<String>,
}

struct EnvVarGuard {
  key:      &'static str,
  previous: Option<String>,
}

impl EnvVarGuard {
  fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
    let previous = env::var(key).ok();
    unsafe { env::set_var(key, value) };
    Self { key, previous }
  }
}

impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    match &self.previous {
      Some(value) => unsafe { env::set_var(self.key, value) },
      None => unsafe { env::remove_var(self.key) },
    }
  }
}

impl HomeGuard {
  fn set(temp_home: &TempDir) -> Self {
    let previous_home = std::env::var("HOME").ok();
    unsafe { std::env::set_var("HOME", temp_home.path()) };
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

struct MemoryBaseDirGuard {
  previous_base_dir: Option<std::ffi::OsString>,
}

impl MemoryBaseDirGuard {
  fn set(path: &TempDir) -> Self {
    let previous_base_dir = std::env::var_os("BLPRNT_MEMORY_BASE_DIR");
    unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", path.path()) };
    Self { previous_base_dir }
  }
}

impl Drop for MemoryBaseDirGuard {
  fn drop(&mut self) {
    match self.previous_base_dir.take() {
      Some(path) => unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", path) },
      None => unsafe { std::env::remove_var("BLPRNT_MEMORY_BASE_DIR") },
    }
  }
}

struct TestContext {
  _home:       TempDir,
  _guard:      HomeGuard,
  _memory_dir: MemoryBaseDirGuard,
  _cwd_guard:  CwdGuard,
  employee_id: String,
  project_id:  String,
}

struct CwdGuard {
  previous_cwd: std::path::PathBuf,
}

impl CwdGuard {
  fn set(path: &TempDir) -> Self {
    let previous_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(path.path()).unwrap();
    Self { previous_cwd }
  }
}

impl Drop for CwdGuard {
  fn drop(&mut self) {
    std::env::set_current_dir(&self.previous_cwd).unwrap();
  }
}

async fn setup_context() -> TestContext {
  let home = TempDir::new().unwrap();
  let guard = HomeGuard::set(&home);
  let memory_dir = MemoryBaseDirGuard::set(&home);
  let cwd_guard = CwdGuard::set(&home);

  let employee = EmployeeRepository::create(EmployeeModel {
    name: "Memory Tester".to_string(),
    kind: EmployeeKind::Agent,
    role: EmployeeRole::Custom("engineer".to_string()),
    title: "Memory Tester".to_string(),
    ..Default::default()
  })
  .await
  .unwrap();

  let project = ProjectRepository::create(ProjectModel::new("Memory Project".to_string(), vec![])).await.unwrap();

  TestContext {
    _home:       home,
    _guard:      guard,
    _memory_dir: memory_dir,
    _cwd_guard:  cwd_guard,
    employee_id: employee.id.uuid().to_string(),
    project_id:  project.id.uuid().to_string(),
  }
}

async fn create_owner() -> String {
  EmployeeRepository::create(EmployeeModel {
    name: "Owner".to_string(),
    kind: EmployeeKind::Person,
    role: EmployeeRole::Owner,
    title: "Owner".to_string(),
    ..Default::default()
  })
  .await
  .unwrap()
  .id
  .uuid()
  .to_string()
}

fn request_with_employee(builder: axum::http::request::Builder, employee_id: &str) -> axum::http::request::Builder {
  builder.header("x-blprnt-employee-id", employee_id)
}

async fn response_json(response: axum::response::Response) -> Value {
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  serde_json::from_slice(&bytes).unwrap()
}

fn test_app() -> Router {
  Router::new().nest("/api", super::routes())
}

fn write_employee_repo(root: &std::path::Path, slug: &str, role: &str, skills: &[&str]) {
  let employee_dir = root.join("employees").join(slug);
  fs::create_dir_all(&employee_dir).unwrap();
  let skills_yaml = if skills.is_empty() {
    String::new()
  } else {
    format!("skills:\n{}", skills.iter().map(|skill| format!("  - {skill}\n")).collect::<String>())
  };
  fs::write(
    employee_dir.join("blprnt.yml"),
    format!("name: {}\nrole: {role}\ncapabilities:\n  - execution\n{skills_yaml}", slug.replace('-', " ")),
  )
  .unwrap();
  fs::write(employee_dir.join("AGENTS.md"), format!("You are {slug}.\n")).unwrap();
  fs::write(employee_dir.join("HEARTBEAT.md"), "Do the work.\n").unwrap();
  fs::write(employee_dir.join("SOUL.md"), "Stay pragmatic.\n").unwrap();
  fs::write(employee_dir.join("TOOLS.md"), "Use the API.\n").unwrap();

  for skill in skills {
    let skill_dir = root.join("skills").join(skill);
    fs::create_dir_all(skill_dir.join("references")).unwrap();
    fs::write(
      skill_dir.join("SKILL.md"),
      format!("---\nname: {skill}\ndescription: imported skill\n---\n\n# {skill}\n"),
    )
    .unwrap();
    fs::write(skill_dir.join("references").join("notes.md"), "reference\n").unwrap();
  }
}

#[test]
fn skills_route_lists_builtin_and_user_skills() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let user_skill_dir = shared::paths::agents_skills_dir().join("user-skill");
    std::fs::create_dir_all(&user_skill_dir).unwrap();
    std::fs::write(
      user_skill_dir.join("SKILL.md"),
      "---\nname: user-skill\ndescription: user skill\n---\n\n# User Skill\n",
    )
    .unwrap();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/skills"), &context.employee_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    if status != StatusCode::OK {
      panic!("unexpected response {status}: {payload}");
    }
    assert!(payload.as_array().unwrap().iter().any(|skill| skill["name"] == "blprnt"));
    assert!(payload.as_array().unwrap().iter().any(|skill| skill["name"] == "user-skill"));
  });
}

#[test]
fn create_employee_normalizes_skill_stack_paths() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let _events = API_EVENTS.subscribe();
    let builtin_skill =
      skills::list_skills().unwrap().into_iter().find(|skill| skill.name == "blprnt").expect("builtin skill");

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(format!(
          r##"{{
            "name":"Frontend Engineer",
            "kind":"agent",
            "role":"staff",
            "title":"Frontend Engineer",
            "icon":"bot",
            "color":"#123456",
            "capabilities":[],
            "provider_config":{{"provider":"mock","slug":"frontend-engineer"}},
            "runtime_config":{{
              "heartbeat_interval_sec":60,
              "heartbeat_prompt":"Build frontend features.",
              "wake_on_demand":true,
              "max_concurrent_runs":1,
              "skill_stack":[{{"name":"blprnt","path":"{}"}}]
            }}
          }}"##,
          shared::paths::blprnt_builtin_skills_mirror_dir().join("blprnt").join("SKILL.md").display()
        )))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    if status != StatusCode::OK {
      panic!("unexpected response {status}: {payload}");
    }
    assert_eq!(payload["runtime_config"]["skill_stack"][0]["name"], "blprnt");
    assert_eq!(payload["runtime_config"]["skill_stack"][0]["path"], builtin_skill.path.to_string_lossy().as_ref());
  });
}

#[test]
fn import_employee_route_creates_employee_from_repo() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let repo = TempDir::new().unwrap();
    write_employee_repo(repo.path(), "data-analyst", "staff", &["analytics-tracking"]);
    let _repo_guard = EnvVarGuard::set("BLPRNT_EMPLOYEES_REPO", repo.path());
    let mut events = API_EVENTS.subscribe();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees/import").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(r#"{"slug":"data-analyst"}"#))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected import response: {payload}");
    assert_eq!(payload["name"], "data analyst");
    assert_eq!(payload["role"], "staff");
    assert_eq!(payload["runtime_config"]["skill_stack"][0]["name"], "analytics-tracking");

    match events.recv().await.unwrap() {
      ApiEvent::AddEmployee { employee_id } => {
        assert_eq!(employee_id.uuid().to_string(), payload["id"].as_str().unwrap())
      }
      event => panic!("unexpected event: {:?}", event),
    }
  });
}

#[test]
fn import_employee_route_force_updates_existing_ceo() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let repo = TempDir::new().unwrap();
    write_employee_repo(repo.path(), "ceo", "ceo", &[]);
    let _repo_guard = EnvVarGuard::set("BLPRNT_EMPLOYEES_REPO", repo.path());
    let mut events = API_EVENTS.subscribe();

    let existing_ceo = EmployeeRepository::create(EmployeeModel {
      name: "Existing CEO".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Ceo,
      title: "CEO".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees/import").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(r#"{"slug":"ceo","force":true}"#))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected import response: {payload}");
    assert_eq!(payload["id"], existing_ceo.id.uuid().to_string());
    assert_eq!(payload["name"], "ceo");

    match events.recv().await.unwrap() {
      ApiEvent::UpdateEmployee { employee_id } => assert_eq!(employee_id, existing_ceo.id),
      event => panic!("unexpected event: {:?}", event),
    }
  });
}

#[test]
fn memory_routes_support_project_memory_default_path_flow() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/projects/{}/memory", context.project_id))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(r##"{"content":"# Launch Notes\n\nShip the project memory API."}"##))
        .unwrap(),
      )
      .await
      .unwrap();

    let create_status = create_response.status();
    let created = response_json(create_response).await;
    assert_eq!(create_status, StatusCode::OK, "unexpected create response: {created}");
    let path = created["path"].as_str().unwrap().to_string();
    assert_eq!(path, "SUMMARY.md");

    let list_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{}/memory", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let listed = response_json(list_response).await;
    assert_eq!(listed["root_path"], "$PROJECT_HOME");
    assert_eq!(listed["nodes"], serde_json::json!([{ "type": "file", "name": "SUMMARY.md", "path": "SUMMARY.md" }]));

    let read_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/projects/{}/memory/file?path={}", context.project_id, path)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], path);
    assert_eq!(read["content"], "# Launch Notes\n\nShip the project memory API.");

    let update_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/projects/{}/memory/file", context.project_id))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(format!(
          "{{\"path\":\"{}\",\"content\":\"# Updated Notes\\n\\nSearch should find this change.\"}}",
          path
        )))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = response_json(update_response).await;
    assert_eq!(updated["content"], "# Updated Notes\n\nSearch should find this change.");

    let search_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/projects/{}/memory/search", context.project_id))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(r#"{"query":"find this change","limit":5}"#))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(search_response.status(), StatusCode::OK);
    let search = response_json(search_response).await;
    assert_eq!(search["memories"][0]["content"], "# Updated Notes\n\nSearch should find this change.");
  });
}

#[test]
fn memory_routes_reject_traversal_paths() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/projects/{}/memory/file?path=../secret.md", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let payload = response_json(response).await;
    assert_eq!(payload["code"], "BAD_REQUEST");
  });
}

#[test]
fn memory_routes_support_employee_memory_default_path_flow() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/employees/me/memory")
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(r##"{"content":"# Runtime Notes\n\nTrack provider interruptions."}"##))
        .unwrap(),
      )
      .await
      .unwrap();

    let create_status = create_response.status();
    let created = response_json(create_response).await;
    assert_eq!(create_status, StatusCode::OK, "unexpected create response: {created}");
    let path = created["path"].as_str().unwrap().to_string();
    let expected_date = Local::now().date_naive().format("%Y-%m-%d").to_string();
    assert_eq!(path, format!("memory/{expected_date}.md"));

    let list_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri("/api/v1/employees/me/memory"),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let listed = response_json(list_response).await;
    assert_eq!(listed["root_path"], "$AGENT_HOME");
    assert_eq!(
      listed["nodes"],
      serde_json::json!([{
        "type": "directory",
        "name": "memory",
        "path": "memory",
        "children": [{
          "type": "file",
          "name": format!("{expected_date}.md"),
          "path": path,
        }]
      }])
    );

    let read_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/employees/me/memory/file?path={path}")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], path);
    assert_eq!(read["content"], "# Runtime Notes\n\nTrack provider interruptions.");

    let update_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri("/api/v1/employees/me/memory/file")
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(format!(
          "{{\"path\":\"{path}\",\"content\":\"# Runtime Notes\\n\\nAsk-question flow is now covered.\"}}"
        )))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = response_json(update_response).await;
    assert_eq!(updated["content"], "# Runtime Notes\n\nAsk-question flow is now covered.");

    let search_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/employees/me/memory/search")
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(r#"{"query":"ask-question flow","limit":5}"#))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(search_response.status(), StatusCode::OK);
    let search = response_json(search_response).await;
    assert_eq!(search["memories"][0]["content"], "# Runtime Notes\n\nAsk-question flow is now covered.");
  });
}

#[test]
fn memory_routes_allow_project_create_with_explicit_scope_relative_path() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let path = "resources/runtime/summary.md";

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/projects/{}/memory", context.project_id))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(format!("{{\"path\":\"{path}\",\"content\":\"# Runtime\\n\\nProvider streaming notes.\"}}")))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    assert_eq!(created["path"], path);

    let read_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/projects/{}/memory/file?path={path}", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], path);
    assert_eq!(read["content"], "# Runtime\n\nProvider streaming notes.");
  });
}

#[test]
fn memory_routes_allow_employee_create_with_explicit_scope_relative_path() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let path = ".learnings/ERRORS.md";

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/employees/me/memory")
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(format!(
          "{{\"path\":\"{path}\",\"content\":\"# Errors\\n\\nInterrupt cleanup needs coverage.\"}}"
        )))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    assert_eq!(created["path"], path);

    let read_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/employees/me/memory/file?path={path}")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], path);
    assert_eq!(read["content"], "# Errors\n\nInterrupt cleanup needs coverage.");
  });
}

#[test]
fn memory_routes_require_existing_projects() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let missing_project_id = persistence::Uuid::new_v4().to_string();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{missing_project_id}/memory")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let payload = response_json(response).await;
    assert_eq!(payload["code"], "PROJECT_NOT_FOUND");
  });
}

#[test]
fn project_routes_create_project_and_fetch_by_id() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/projects").header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(r#"{"name":"Runtime Hardening","working_directories":["/tmp/runtime","/tmp/providers"]}"#))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    assert_eq!(created["name"], "Runtime Hardening");
    assert_eq!(created["working_directories"], serde_json::json!(["/tmp/runtime", "/tmp/providers"]));

    let project_id = created["id"].as_str().unwrap();
    let get_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{project_id}")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);
    let fetched = response_json(get_response).await;
    assert_eq!(fetched["id"], project_id);
    assert_eq!(fetched["name"], "Runtime Hardening");
    assert_eq!(fetched["working_directories"], serde_json::json!(["/tmp/runtime", "/tmp/providers"]));
  });
}

#[cfg(debug_assertions)]
#[test]
fn dev_routes_nuke_database_requires_owner_permissions() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("DELETE").uri("/api/v1/dev/database"), &context.employee_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "unexpected nuke response: {payload}");
  });
}

#[cfg(debug_assertions)]
#[test]
fn dev_routes_nuke_database_clears_all_records() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    ProviderRepository::create(ProviderModel::new(Provider::OpenAi)).await.unwrap();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let owner_uuid = persistence::Uuid::parse_str(&owner_id).unwrap();
    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();

    IssueRepository::create(IssueModel {
      title: "Nuke test issue".to_string(),
      description: "Ensures the debug reset route clears persisted data.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(owner_uuid.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("DELETE").uri("/api/v1/dev/database"), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let response_status = response.status();
    let response_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body = String::from_utf8_lossy(&response_bytes);
    assert_eq!(response_status, StatusCode::NO_CONTENT, "unexpected nuke response body: {response_body}");
    assert!(EmployeeRepository::list().await.unwrap().is_empty());
    assert!(ProjectRepository::list().await.unwrap().is_empty());
    assert!(ProviderRepository::list().await.unwrap().is_empty());
    assert!(IssueRepository::list(ListIssuesParams::default()).await.unwrap().is_empty());
    assert!(RunRepository::list(RunFilter { employee: None, status: None, trigger: None }).await.unwrap().is_empty());
  });
}

#[test]
fn issue_routes_create_respects_explicit_status() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let payload = serde_json::json!({
      "title": "Bootstrap CEO",
      "description": "Create the first leadership issue.",
      "status": "todo",
      "priority": "medium",
      "project": context.project_id,
      "assignee": context.employee_id
    })
    .to_string();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/issues")
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(payload))
        .unwrap(),
      )
      .await
      .unwrap();

    let response_status = response.status();
    let response_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body = String::from_utf8_lossy(&response_bytes);
    assert_eq!(response_status, StatusCode::OK, "unexpected create response body: {response_body}");

    let created: Value = serde_json::from_str(&response_body).unwrap();
    assert_eq!(created["status"], "todo");
  });
}

#[test]
fn issue_routes_patch_update_nullable_fields_and_record_action() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();
    let issue = IssueRepository::create(IssueModel {
      title: "Original runtime issue".to_string(),
      description: "Needs controller lifecycle coverage.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(employee_id.into()),
      priority: IssuePriority::Medium,
      ..Default::default()
    })
    .await
    .unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();
    let run_id = run.id.uuid().to_string();
    let patch = serde_json::to_string(&IssuePatch {
      title: Some("Updated runtime issue".to_string()),
      description: Some("Provider streaming interruption coverage is tracked.".to_string()),
      status: Some(IssueStatus::Blocked),
      project: Some(None),
      assignee: Some(None),
      priority: Some(IssuePriority::Critical),
      ..Default::default()
    })
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/issues/{}", issue.id.uuid()))
            .header("content-type", "application/json")
            .header("x-blprnt-run-id", &run_id),
          &context.employee_id,
        )
        .body(Body::from(patch))
        .unwrap(),
      )
      .await
      .unwrap();

    let response_status = response.status();
    let response_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body = String::from_utf8_lossy(&response_bytes);
    assert_eq!(response_status, StatusCode::OK, "unexpected update response body: {response_body}");

    let stored = IssueRepository::get(issue.id.clone()).await.unwrap();
    assert_eq!(stored.title, "Updated runtime issue");
    assert_eq!(stored.description, "Provider streaming interruption coverage is tracked.");
    assert_eq!(stored.status, IssueStatus::Blocked);
    assert_eq!(stored.priority, IssuePriority::Critical);
    assert!(stored.project.is_none());
    assert!(stored.assignee.is_none());

    let actions = IssueRepository::list_actions(issue.id.clone()).await.unwrap();
    assert!(actions.iter().any(|action| {
      matches!(action.action_kind, IssueActionKind::Update)
        && action.creator.uuid().to_string() == context.employee_id
        && action.run_id.as_ref().map(|run| run.uuid().to_string()).as_deref() == Some(run_id.as_str())
    }));
  });
}

#[test]
fn issue_routes_assign_clears_existing_checkout_before_handoff() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let assignee = EmployeeRepository::create(EmployeeModel {
      name: "Handoff Target".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Custom("engineer".to_string()),
      title: "Handoff Target".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();
    let issue = IssueRepository::create(IssueModel {
      title: "Escalation handoff issue".to_string(),
      description: "Ensures reassignment drops the previous checkout lock.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(employee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let checkout_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri(format!("/api/v1/issues/{}/checkout", issue.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let checkout_status = checkout_response.status();
    let checkout_body = response_json(checkout_response).await;
    assert_eq!(checkout_status, StatusCode::OK, "unexpected checkout response: {checkout_body}");

    let assign_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/assign", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(serde_json::json!({ "employee_id": assignee.id.uuid().to_string() }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();

    let assign_status = assign_response.status();
    let assign_body = response_json(assign_response).await;
    assert_eq!(assign_status, StatusCode::OK, "unexpected assign response: {assign_body}");

    let stored = IssueRepository::get(issue.id.clone()).await.unwrap();
    assert_eq!(stored.assignee, Some(assignee.id.clone()));
    assert!(stored.checked_out_by.is_none());

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id, assignee.id);
        assert!(matches!(trigger, RunTrigger::IssueAssignment { .. }));
      }
      event => panic!("unexpected event: {event:?}"),
    }
  });
}

#[test]
fn issue_routes_list_child_issues_by_parent() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();
    let parent = IssueRepository::create(IssueModel {
      title: "Parent runtime issue".to_string(),
      description: "Tracks the rollout of child issue support.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let child = IssueRepository::create(IssueModel {
      title: "Child runtime issue".to_string(),
      description: "Exposes sub-issue progress in the detail page.".to_string(),
      status: IssueStatus::InProgress,
      project: Some(project_id.into()),
      parent_id: Some(parent.id.clone()),
      priority: IssuePriority::Medium,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/issues/{}/children", parent.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let response_status = response.status();
    let payload = response_json(response).await;

    assert_eq!(response_status, StatusCode::OK, "unexpected children response: {payload}");

    let children = payload.as_array().expect("children response should be an array");
    assert_eq!(children.len(), 1);
    assert_eq!(children[0]["id"], child.id.uuid().to_string());
    assert_eq!(children[0]["parent_id"], parent.id.uuid().to_string());
    assert_eq!(children[0]["title"], "Child runtime issue");
  });
}

#[test]
fn issue_routes_list_issues_filter_by_assignee() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();
    let assignee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let other_employee = EmployeeRepository::create(EmployeeModel {
      name: "Other Assignee".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Custom("engineer".to_string()),
      title: "Other Assignee".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let assigned_issue = IssueRepository::create(IssueModel {
      title: "Assigned runtime issue".to_string(),
      description: "Should be returned by assignee filter.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let _other_issue = IssueRepository::create(IssueModel {
      title: "Other employee issue".to_string(),
      description: "Should not be returned by assignee filter.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(other_employee.id.clone()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/issues?assignee={}", context.employee_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let response_status = response.status();
    let response_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body = String::from_utf8_lossy(&response_bytes);
    assert_eq!(response_status, StatusCode::OK, "unexpected issue list response body: {response_body}");
    let payload: Value = serde_json::from_slice(&response_bytes).unwrap();

    let issues = payload.as_array().expect("issue list response should be an array");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["id"], assigned_issue.id.uuid().to_string());
    assert_eq!(issues[0]["assignee"], context.employee_id);
  });
}

#[test]
fn employee_routes_require_update_permissions() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let target = EmployeeRepository::create(EmployeeModel {
      name: "Runtime Target".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Staff,
      title: "Runtime Target".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();
    let patch =
      serde_json::to_string(&EmployeePatch { title: Some("Updated Runtime Title".to_string()), ..Default::default() })
        .unwrap();

    let forbidden_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}", target.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(patch.clone()))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
    let forbidden_payload = response_json(forbidden_response).await;
    assert_eq!(forbidden_payload["code"], "FORBIDDEN");

    let target_id = target.id.clone();
    let unchanged = EmployeeRepository::get(target_id.clone()).await.unwrap();
    assert_eq!(unchanged.title, "Runtime Target");

    let allowed_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}", target.id.uuid()))
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(patch))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(allowed_response.status(), StatusCode::OK);

    let updated = EmployeeRepository::get(target_id).await.unwrap();
    assert_eq!(updated.title, "Updated Runtime Title");
  });
}

fn create_agent_employee_payload(name: &str, role: &str) -> String {
  serde_json::json!({
    "name": name,
    "kind": "agent",
    "role": role,
    "title": format!("{name} Title"),
    "icon": "bot",
    "color": "#3b82f6",
    "capabilities": ["runtime work"],
    "provider_config": {
      "provider": "mock",
      "slug": name.to_lowercase().replace(' ', "-")
    },
    "runtime_config": {
      "heartbeat_interval_sec": 1800,
      "heartbeat_prompt": format!("Handle {name} work."),
      "wake_on_demand": true,
      "max_concurrent_runs": 1
    }
  })
  .to_string()
}

#[test]
fn employee_routes_ceo_can_hire_manager_and_staff() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();
    let _events = API_EVENTS.subscribe();

    let ceo = EmployeeRepository::create(EmployeeModel {
      name: "CEO".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Ceo,
      title: "CEO".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    for role in ["manager", "staff"] {
      let response = app
        .clone()
        .oneshot(
          request_with_employee(
            Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
            &ceo.id.uuid().to_string(),
          )
          .body(Body::from(create_agent_employee_payload(&format!("{role} hire"), role)))
          .unwrap(),
        )
        .await
        .unwrap();

      let response_status = response.status();
      let response_body = response_json(response).await;
      assert_eq!(response_status, StatusCode::OK, "unexpected create employee response: {response_body}");
    }

    let forbidden_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &ceo.id.uuid().to_string(),
        )
        .body(Body::from(create_agent_employee_payload("ceo hire", "ceo")))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
  });
}

#[test]
fn employee_routes_owner_can_hire_any_non_owner_role() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();
    let _events = API_EVENTS.subscribe();
    let owner_id = create_owner().await;

    for role in ["ceo", "manager", "staff"] {
      let response = app
        .clone()
        .oneshot(
          request_with_employee(
            Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
            &owner_id,
          )
          .body(Body::from(create_agent_employee_payload(&format!("{role} hire"), role)))
          .unwrap(),
        )
        .await
        .unwrap();

      let response_status = response.status();
      let response_body = response_json(response).await;
      assert_eq!(response_status, StatusCode::OK, "unexpected create employee response: {response_body}");
    }
  });
}

#[test]
fn employee_routes_manager_can_only_hire_staff() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();
    let _events = API_EVENTS.subscribe();

    let manager = EmployeeRepository::create(EmployeeModel {
      name: "Manager".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Manager,
      title: "Manager".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let ok_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &manager.id.uuid().to_string(),
        )
        .body(Body::from(create_agent_employee_payload("staff hire", "staff")))
        .unwrap(),
      )
      .await
      .unwrap();

    let ok_status = ok_response.status();
    let ok_body = response_json(ok_response).await;
    assert_eq!(ok_status, StatusCode::OK, "unexpected create employee response: {ok_body}");

    let forbidden_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &manager.id.uuid().to_string(),
        )
        .body(Body::from(create_agent_employee_payload("manager hire", "manager")))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
  });
}

#[test]
fn employee_routes_staff_cannot_hire() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();

    let staff = EmployeeRepository::create(EmployeeModel {
      name: "Staff".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Staff".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &staff.id.uuid().to_string(),
        )
        .body(Body::from(create_agent_employee_payload("staff hire", "staff")))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  });
}

#[test]
fn run_routes_list_without_request_body() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/runs"), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    let runs = payload["items"].as_array().unwrap();
    assert!(runs.iter().any(|entry| entry["id"] == run.id.uuid().to_string()));
  });
}

#[test]
fn run_routes_cancel_via_cancel_suffix() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();
    let run_id = run.id.uuid().to_string();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("DELETE").uri(format!("/api/v1/runs/{run_id}/cancel")),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    match events.recv().await.unwrap() {
      ApiEvent::CancelRun { run_id: cancelled_run_id, .. } => {
        assert_eq!(cancelled_run_id.uuid().to_string(), run_id);
      }
      event => panic!("unexpected event: {event:?}"),
    }
  });
}

#[test]
fn run_routes_get_by_uuid_path() {
  let _lock = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();
    let run_id = run.id.uuid().to_string();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri(format!("/api/v1/runs/{run_id}")), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    assert_eq!(payload["id"], run_id);
  });
}
