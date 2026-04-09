use std::env;
use std::fs;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use axum::http::header;
use events::API_EVENTS;
use events::ApiEvent;
use events::EMPLOYEE_EVENTS;
use events::EmployeeEventKind;
use events::ISSUE_EVENTS;
use events::IssueEventKind;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::EmployeeRuntimeConfig;
use persistence::prelude::EmployeeSkillRef;
use persistence::prelude::EmployeeStatus;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueLabel;
use persistence::prelude::IssueModel;
use persistence::prelude::IssuePatch;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ListIssuesParams;
use persistence::prelude::McpServerModel;
use persistence::prelude::McpServerRepository;
use persistence::prelude::ProjectModel;
use persistence::prelude::ProjectRepository;
use persistence::prelude::ProviderModel;
use persistence::prelude::ProviderRepository;
use persistence::prelude::ReasoningEffort;
use persistence::prelude::RunEnabledMcpServerRepository;
use persistence::prelude::RunFilter;
use persistence::prelude::RunModel;
use persistence::prelude::RunRepository;
use persistence::prelude::RunTrigger;
use persistence::prelude::SurrealConnection;
use persistence::prelude::TelegramConfigRepository;
use persistence::prelude::TelegramCorrelationKind;
use persistence::prelude::TelegramIssueWatchRepository;
use persistence::prelude::TelegramLinkRepository;
use persistence::prelude::TelegramMessageCorrelationModel;
use persistence::prelude::TelegramMessageCorrelationRepository;
use persistence::prelude::TelegramMessageDirection;
use persistence::prelude::TurnModel;
use persistence::prelude::TurnRepository;
use serde_json::Value;
use shared::agent::Provider;
use shared::tools::McpServerAuthState;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::routes::v1::issues::build_comment_snippet_for_labels;

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

#[derive(Clone)]
struct TelegramMockState {
  next_message_id:       Arc<AtomicI64>,
  next_update_id:        Arc<AtomicI64>,
  sent_messages:         Arc<Mutex<Vec<Value>>>,
  webhook_registrations: Arc<Mutex<Vec<Value>>>,
  queued_updates:        Arc<Mutex<Vec<Value>>>,
}

struct CwdGuard {
  previous_cwd: std::path::PathBuf,
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
  ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn telegram_mock_server() -> (String, TelegramMockState, tokio::task::JoinHandle<()>) {
  let state = TelegramMockState {
    next_message_id:       Arc::new(AtomicI64::new(1000)),
    next_update_id:        Arc::new(AtomicI64::new(0)),
    sent_messages:         Arc::new(Mutex::new(Vec::new())),
    webhook_registrations: Arc::new(Mutex::new(Vec::new())),
    queued_updates:        Arc::new(Mutex::new(Vec::new())),
  };
  let app = Router::new()
    .route(
      "/botbot-token/sendMessage",
      axum::routing::post({
        let state = state.clone();
        move |axum::Json(payload): axum::Json<Value>| {
          let state = state.clone();
          async move {
            state.sent_messages.lock().unwrap().push(payload);
            let message_id = state.next_message_id.fetch_add(1, Ordering::SeqCst) + 1;
            axum::Json(serde_json::json!({
              "ok": true,
              "result": { "message_id": message_id }
            }))
          }
        }
      }),
    )
    .route(
      "/botbot-token/getUpdates",
      axum::routing::get({
        let state = state.clone();
        move |query: axum::extract::Query<std::collections::HashMap<String, String>>| {
          let state = state.clone();
          async move {
            let offset = query.0.get("offset").and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
            let updates = state
              .queued_updates
              .lock()
              .unwrap()
              .iter()
              .filter(|payload| payload["update_id"].as_i64().unwrap_or_default() >= offset)
              .cloned()
              .collect::<Vec<_>>();
            axum::Json(serde_json::json!({ "ok": true, "result": updates }))
          }
        }
      }),
    )
    .route(
      "/botconflict-token/getUpdates",
      axum::routing::get(|| async {
        (
          StatusCode::CONFLICT,
          axum::Json(serde_json::json!({
            "ok": false,
            "error_code": 409,
            "description": "Conflict: terminated by other getUpdates request; make sure that only one bot instance is running"
          })),
        )
      }),
    )
    .route(
      "/botbot-token/setWebhook",
      axum::routing::post({
        let state = state.clone();
        move |axum::Json(payload): axum::Json<Value>| {
          let state = state.clone();
          async move {
            state.webhook_registrations.lock().unwrap().push(payload);
            axum::Json(serde_json::json!({ "ok": true, "result": true }))
          }
        }
      }),
    )
    .route(
      "/botdegraded-token/getUpdates",
      axum::routing::get({
        let state = state.clone();
        move |query: axum::extract::Query<std::collections::HashMap<String, String>>| {
          let state = state.clone();
          async move {
            let offset = query.0.get("offset").and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
            let updates = state
              .queued_updates
              .lock()
              .unwrap()
              .iter()
              .filter(|payload| payload["update_id"].as_i64().unwrap_or_default() >= offset)
              .cloned()
              .collect::<Vec<_>>();
            axum::Json(serde_json::json!({ "ok": true, "result": updates }))
          }
        }
      }),
    )
    .route(
      "/botdegraded-token/setWebhook",
      axum::routing::post({
        let state = state.clone();
        move |axum::Json(payload): axum::Json<Value>| {
          let state = state.clone();
          async move {
            state.webhook_registrations.lock().unwrap().push(payload);
            axum::Json(serde_json::json!({ "ok": true, "result": true }))
          }
        }
      }),
    );

  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();
  let handle = tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
  });

  (format!("http://{}", addr), state, handle)
}

fn queue_telegram_update(state: &TelegramMockState, message: Value) {
  let update_id = state.next_update_id.fetch_add(1, Ordering::SeqCst) + 1;
  state.queued_updates.lock().unwrap().push(serde_json::json!({
    "update_id": update_id,
    "message": message,
  }));
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
  SurrealConnection::reset().await.unwrap();

  let employee = EmployeeRepository::create(EmployeeModel {
    name: "Memory Tester".to_string(),
    kind: EmployeeKind::Agent,
    role: EmployeeRole::Custom("engineer".to_string()),
    title: "Memory Tester".to_string(),
    ..Default::default()
  })
  .await
  .unwrap();

  let project =
    ProjectRepository::create(ProjectModel::new("Memory Project".to_string(), String::new(), vec![])).await.unwrap();

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

fn request_with_employee_alias(
  builder: axum::http::request::Builder,
  employee_id: &str,
) -> axum::http::request::Builder {
  builder.header("x-employee-id", employee_id)
}

fn session_cookie_from_response(response: &axum::response::Response) -> String {
  response
    .headers()
    .get_all(header::SET_COOKIE)
    .iter()
    .find_map(|value| value.to_str().ok())
    .and_then(|cookie| cookie.split(';').next())
    .map(str::to_string)
    .expect("expected session cookie")
}

async fn response_json(response: axum::response::Response) -> Value {
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  serde_json::from_slice(&bytes).unwrap()
}

fn test_app() -> Router {
  Router::new().nest("/api", super::routes())
}

#[test]
fn mcp_server_routes_create_update_and_list_configured_servers() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/mcp-servers").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "display_name": "QMD",
            "description": "Structured search",
            "transport": "stdio",
            "endpoint_url": "qmd mcp",
            "auth_state": "connected",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    assert_eq!(created["display_name"], "QMD");
    assert_eq!(created["auth_state"], "connected");
    assert!(created.get("project_id").is_none());

    let list_response = app
      .clone()
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/mcp-servers"), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let listed = response_json(list_response).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let update_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/mcp-servers/{}", created["id"].as_str().unwrap()))
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "description": "Structured search and retrieval",
            "auth_state": "auth_required",
            "auth_summary": "Needs reconnect"
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = response_json(update_response).await;
    assert_eq!(updated["description"], "Structured search and retrieval");
    assert_eq!(updated["auth_state"], "auth_required");
    assert_eq!(updated["auth_summary"], "Needs reconnect");
  });
}

#[test]
fn run_mcp_enablement_route_lists_run_scoped_state_separately_from_configured_servers() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let server = McpServerRepository::create(McpServerModel {
      display_name: "QMD".to_string(),
      description:  "Structured search".to_string(),
      transport:    "stdio".to_string(),
      endpoint_url: "qmd mcp".to_string(),
      auth_state:   McpServerAuthState::Connected,
      auth_summary: None,
      enabled:      true,
      created_at:   chrono::Utc::now(),
      updated_at:   chrono::Utc::now(),
    })
    .await
    .unwrap();

    let run = RunRepository::create(RunModel::new(
      context.employee_id.parse::<persistence::Uuid>().unwrap().into(),
      RunTrigger::Manual,
    ))
    .await
    .unwrap();
    RunEnabledMcpServerRepository::enable(run.id.clone(), server.id.clone()).await.unwrap();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/runs/{}/mcp-servers", run.id.uuid())),
          &context.employee_id,
        )
        .header("x-blprnt-run-id", run.id.uuid().to_string())
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    assert_eq!(payload.as_array().unwrap().len(), 1);
    assert_eq!(payload[0]["server_id"], server.id.uuid().to_string());
  });
}

#[test]
fn mcp_oauth_routes_require_owner_and_expose_status_contract() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let server = McpServerRepository::create(McpServerModel {
      display_name: "OAuth MCP".to_string(),
      description:  "OAuth-backed MCP server".to_string(),
      transport:    "http".to_string(),
      endpoint_url: "https://example.com/mcp".to_string(),
      auth_state:   McpServerAuthState::ReconnectRequired,
      auth_summary: Some("Reconnect required".to_string()),
      enabled:      true,
      created_at:   chrono::Utc::now(),
      updated_at:   chrono::Utc::now(),
    })
    .await
    .unwrap();

    let forbidden_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/mcp-servers/{}/oauth", server.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);

    let status_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/mcp-servers/{}/oauth", server.id.uuid())),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(status_response.status(), StatusCode::OK);
    let status = response_json(status_response).await;
    assert_eq!(status["server_id"], server.id.uuid().to_string());
    assert_eq!(status["auth_state"], "reconnect_required");
    assert_eq!(status["auth_summary"], "Reconnect required");
    assert_eq!(status["has_token"], false);
  });
}

#[test]
fn openapi_includes_mcp_oauth_routes() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let response = test_app()
      .oneshot(Request::builder().method("GET").uri("/api/v1/mcp-servers/openapi.json").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let paths = body["paths"].as_object().expect("openapi paths should be an object");
    assert!(paths.contains_key("/mcp-servers/{server_id}/oauth"));
    assert!(paths.contains_key("/mcp-servers/{server_id}/oauth/launch"));
    assert!(paths.contains_key("/mcp-servers/{server_id}/oauth/reconnect"));
    assert!(paths.contains_key("/mcp-servers/{server_id}/oauth/complete"));
    assert!(paths.contains_key("/mcp-servers/{server_id}/oauth/callback"));
    let create_required = body["components"]["schemas"]["CreateMcpServerPayload"]["required"]
      .as_array()
      .expect("required fields should be an array")
      .iter()
      .filter_map(|value| value.as_str())
      .collect::<Vec<_>>();
    assert!(!create_required.contains(&"project_id"));
    assert!(body["components"]["schemas"]["McpServerDto"]["properties"].get("project_id").is_none());
    assert!(body["paths"]["/mcp-servers"]["get"]["parameters"].is_null());
  });
}

#[test]
fn auth_bootstrap_owner_creates_session_and_me_accepts_cookie_auth() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;

    let bootstrap_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/bootstrap-owner")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            serde_json::json!({
              "name": "Owner",
              "icon": "brain",
              "color": "fuchsia",
              "email": "owner@example.com",
              "password": "supersecure"
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(bootstrap_response.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&bootstrap_response);

    let me_response = test_app()
      .oneshot(
        Request::builder()
          .method("GET")
          .uri("/api/v1/auth/me")
          .header(header::COOKIE, cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(me_response.status(), StatusCode::OK);
    let body = response_json(me_response).await;
    assert_eq!(body["name"], "Owner");
    assert_eq!(body["role"], "owner");
  });
}

#[test]
fn auth_bootstrap_owner_sets_secure_cookie_in_deployed_mode() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let _deployed = EnvVarGuard::set("BLPRNT_DEPLOYED", "true");

    let bootstrap_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/bootstrap-owner")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            serde_json::json!({
              "name": "Owner",
              "icon": "brain",
              "color": "fuchsia",
              "email": "owner@example.com",
              "password": "supersecure"
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    let set_cookie = bootstrap_response
      .headers()
      .get(header::SET_COOKIE)
      .and_then(|value| value.to_str().ok())
      .expect("expected session cookie header");
    assert!(set_cookie.contains("; Secure"));
    assert!(set_cookie.contains("SameSite=Lax"));
  });
}

#[test]
fn auth_status_reports_existing_owner_without_login_credentials() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let _owner_id = create_owner().await;

    let response = test_app()
      .oneshot(Request::builder().method("GET").uri("/api/v1/auth/status").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["has_owner"], true);
    assert_eq!(body["owner_login_configured"], false);
  });
}

#[test]
fn auth_bootstrap_owner_claims_existing_owner_without_login_credentials() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;

    let bootstrap_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/bootstrap-owner")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            serde_json::json!({
              "name": "Recovered Owner",
              "icon": "brain",
              "color": "fuchsia",
              "email": "owner@example.com",
              "password": "supersecure"
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(bootstrap_response.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&bootstrap_response);
    let credential = persistence::prelude::LoginCredentialRepository::find_by_employee(
      owner_id.parse::<persistence::Uuid>().unwrap().into(),
    )
    .await
    .unwrap()
    .expect("expected a credential for the existing owner");
    assert_eq!(credential.email, "owner@example.com");

    let me_response = test_app()
      .oneshot(
        Request::builder()
          .method("GET")
          .uri("/api/v1/auth/me")
          .header(header::COOKIE, cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(me_response.status(), StatusCode::OK);
    let body = response_json(me_response).await;
    assert_eq!(body["id"], owner_id);
    assert_eq!(body["name"], "Recovered Owner");
  });
}

#[test]
fn auth_bootstrap_owner_rejects_existing_owner_recovery_in_deployed_mode() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let _owner_id = create_owner().await;
    let _deployed = EnvVarGuard::set("BLPRNT_DEPLOYED", "true");

    let bootstrap_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/bootstrap-owner")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            serde_json::json!({
              "name": "Recovered Owner",
              "icon": "brain",
              "color": "fuchsia",
              "email": "owner@example.com",
              "password": "supersecure"
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(bootstrap_response.status(), StatusCode::FORBIDDEN);
  });
}

#[test]
fn auth_login_sets_session_cookie_and_logout_revokes_it() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;

    persistence::prelude::LoginCredentialRepository::create(persistence::prelude::LoginCredentialModel {
      employee_id:   owner_id.parse::<persistence::Uuid>().unwrap().into(),
      email:         "owner@example.com".to_string(),
      password_hash: {
        use argon2::Argon2;
        use argon2::PasswordHasher;
        use argon2::password_hash::SaltString;
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        Argon2::default().hash_password(b"supersecure", &salt).unwrap().to_string()
      },
      password_salt: String::new(),
      created_at:    chrono::Utc::now(),
      updated_at:    chrono::Utc::now(),
    })
    .await
    .unwrap();

    let login_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/login")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(serde_json::json!({ "email": "owner@example.com", "password": "supersecure" }).to_string()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = session_cookie_from_response(&login_response);

    let logout_response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/auth/logout")
          .header(header::COOKIE, cookie.clone())
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

    let me_response = test_app()
      .oneshot(
        Request::builder()
          .method("GET")
          .uri("/api/v1/auth/me")
          .header(header::COOKIE, cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(me_response.status(), StatusCode::BAD_REQUEST);
  });
}

#[test]
fn public_owner_onboarding_is_disabled_in_deployed_mode() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let _deployed = EnvVarGuard::set("BLPRNT_DEPLOYED", "true");

    let response = test_app()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/onboarding")
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            serde_json::json!({
              "name": "Owner",
              "icon": "brain",
              "color": "fuchsia"
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  });
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
fn skills_route_lists_user_skills_and_excludes_builtin_skills() {
  let _lock = env_lock();
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
    assert!(!payload.as_array().unwrap().iter().any(|skill| skill["name"] == "blprnt"));
    assert!(payload.as_array().unwrap().iter().any(|skill| skill["name"] == "user-skill"));
  });
}

#[test]
fn skills_route_filters_skills_already_on_requesting_employee() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let builtin_skill =
      skills::list_skills().unwrap().into_iter().find(|skill| skill.name == "blprnt").expect("builtin skill");

    EmployeeRepository::update(
      context.employee_id.parse::<persistence::Uuid>().unwrap().into(),
      EmployeePatch {
        runtime_config: Some(EmployeeRuntimeConfig {
          heartbeat_interval_sec: 3600,
          heartbeat_prompt:       String::new(),
          wake_on_demand:         true,
          timer_wakeups_enabled:  Some(true),
          dreams_enabled:         Some(false),
          max_concurrent_runs:    1,
          skill_stack:            Some(vec![EmployeeSkillRef {
            name: builtin_skill.name.clone(),
            path: builtin_skill.path.to_string_lossy().to_string(),
          }]),
          reasoning_effort:       None,
        }),
        ..Default::default()
      },
    )
    .await
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

    assert!(!payload.as_array().unwrap().iter().any(|skill| skill["name"] == "blprnt"));
  });
}

#[test]
fn skills_route_excludes_builtin_skill_names_even_from_agents_directory() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    skills::ensure_builtin_skills_installed().unwrap();
    assert!(shared::paths::agents_skills_dir().join("blprnt").join("SKILL.md").exists());

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
    assert!(!payload.as_array().unwrap().iter().any(|skill| skill["name"] == "blprnt"));
  });
}

#[test]
fn create_employee_normalizes_skill_stack_paths() {
  let _lock = env_lock();
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
              "timer_wakeups_enabled":true,
              "dreams_enabled":false,
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
    assert_eq!(payload["runtime_config"]["timer_wakeups_enabled"], true);
    assert_eq!(payload["runtime_config"]["dreams_enabled"], false);
  });
}

#[test]
fn create_employee_rejects_more_than_two_skills() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let _events = API_EVENTS.subscribe();
    let available_skills = skills::list_skills().unwrap();
    assert!(available_skills.len() >= 3, "expected at least three available skills");

    let skill_stack = available_skills
      .iter()
      .take(3)
      .map(|skill| serde_json::json!({ "name": skill.name, "path": skill.path.to_string_lossy().to_string() }))
      .collect::<Vec<_>>();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": "Platform Engineer",
            "kind": "agent",
            "role": "staff",
            "title": "Platform Engineer",
            "icon": "bot",
            "color": "#123456",
            "capabilities": [],
            "provider_config": { "provider": "mock", "slug": "platform-engineer" },
            "runtime_config": {
              "heartbeat_interval_sec": 60,
              "heartbeat_prompt": "Build platform features.",
              "wake_on_demand": true,
              "timer_wakeups_enabled": true,
              "max_concurrent_runs": 1,
              "reasoning_effort": null,
              "skill_stack": skill_stack,
            }
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unexpected response: {payload}");
    assert_eq!(payload["code"], "BAD_REQUEST");
  });
}

#[test]
fn update_employee_rejects_more_than_two_skills() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let _events = API_EVENTS.subscribe();
    let available_skills = skills::list_skills().unwrap();
    assert!(available_skills.len() >= 3, "expected at least three available skills");

    let skill_stack = available_skills
      .iter()
      .take(3)
      .map(|skill| serde_json::json!({ "name": skill.name, "path": skill.path.to_string_lossy().to_string() }))
      .collect::<Vec<_>>();

    let employee = EmployeeRepository::create(EmployeeModel {
      name: "Automation Engineer".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Automation Engineer".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}", employee.id.uuid()))
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": null,
            "title": null,
            "status": null,
            "icon": null,
            "color": null,
            "reports_to": null,
            "capabilities": null,
            "provider_config": null,
            "runtime_config": {
              "heartbeat_interval_sec": 60,
              "heartbeat_prompt": "Keep automation healthy.",
              "wake_on_demand": true,
              "timer_wakeups_enabled": true,
              "max_concurrent_runs": 1,
              "reasoning_effort": null,
              "skill_stack": skill_stack,
            }
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unexpected response: {payload}");
    assert_eq!(payload["code"], "BAD_REQUEST");
  });
}

#[test]
fn import_employee_route_creates_employee_from_repo() {
  let _lock = env_lock();
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
  let _lock = env_lock();
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
fn import_employee_route_uses_request_base_url_when_present() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let repo = TempDir::new().unwrap();
    write_employee_repo(repo.path(), "data-analyst", "staff", &["analytics-tracking"]);
    let mut events = API_EVENTS.subscribe();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees/import").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(format!(r#"{{"base_url":"{}","slug":"data-analyst"}}"#, repo.path().display())))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected import response: {payload}");
    assert_eq!(payload["name"], "data analyst");

    match events.recv().await.unwrap() {
      ApiEvent::AddEmployee { employee_id } => {
        assert_eq!(employee_id.uuid().to_string(), payload["id"].as_str().unwrap())
      }
      event => panic!("unexpected event: {:?}", event),
    }
  });
}

#[test]
fn create_employee_route_emits_employee_stream_event_for_person_employees() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let mut events = EMPLOYEE_EVENTS.subscribe();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": "People Ops",
            "kind": "person",
            "role": "staff",
            "title": "People Ops",
            "icon": "user",
            "color": "blue",
            "capabilities": [],
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected create response: {payload}");

    let event = events.recv().await.unwrap();
    assert_eq!(event.kind, EmployeeEventKind::Upsert);
    assert_eq!(event.employee_id.uuid().to_string(), payload["id"].as_str().unwrap());
  });
}

#[test]
fn memory_routes_support_project_memory_list_read_and_search_flow() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let project_memory_root =
      context._home.path().join(".blprnt").join("projects").join(&context.project_id).join("memory");
    fs::create_dir_all(project_memory_root.join("resources/runtime")).unwrap();
    fs::write(project_memory_root.join("SUMMARY.md"), "# Launch Notes\n\nShip the project memory API.").unwrap();
    fs::write(project_memory_root.join("resources/runtime/summary.md"), "# Runtime\n\nSearch should find this change.")
      .unwrap();
    let app = test_app();

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
    assert_eq!(listed["root_path"], "$PROJECT_HOME/memory");
    assert_eq!(
      listed["nodes"],
      serde_json::json!([
        {
          "type": "directory",
          "name": "resources",
          "path": "resources",
          "children": [{
            "type": "directory",
            "name": "runtime",
            "path": "resources/runtime",
            "children": [{
              "type": "file",
              "name": "summary.md",
              "path": "resources/runtime/summary.md"
            }]
          }]
        },
        { "type": "file", "name": "SUMMARY.md", "path": "SUMMARY.md" }
      ])
    );

    let read_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/projects/{}/memory/file?path=SUMMARY.md", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], "SUMMARY.md");
    assert_eq!(read["content"], "# Launch Notes\n\nShip the project memory API.");

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
    assert_eq!(search["memories"][0]["path"], "resources/runtime/summary.md");
    assert_eq!(search["memories"][0]["content"], "# Runtime\n\nSearch should find this change.");
  });
}

#[test]
fn memory_routes_reject_traversal_paths() {
  let _lock = env_lock();
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
fn project_plan_routes_return_empty_list_when_no_plan_files_exist() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{}/plans", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    assert_eq!(payload["plans"], serde_json::json!([]));
  });
}

#[test]
fn project_plan_routes_list_and_read_project_plans_with_explicit_superseded_metadata() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let project_plans_root =
      context._home.path().join(".blprnt").join("projects").join(&context.project_id).join("plans");
    fs::create_dir_all(project_plans_root.join("archive")).unwrap();

    fs::write(
      project_plans_root.join("active-plan-2026-04-08.md"),
      "---\ntitle: Active delivery plan\n---\n# Active delivery plan\n\nShip the project plans browser.",
    )
    .unwrap();
    fs::write(project_plans_root.join("archive/old-plan.txt"), "Status: superseded\nOld plain text plan.").unwrap();

    let app = test_app();

    let list_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{}/plans", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let listed = response_json(list_response).await;
    assert_eq!(listed["plans"][0]["path"], "active-plan-2026-04-08.md");
    assert_eq!(listed["plans"][0]["title"], "Active delivery plan");
    assert_eq!(listed["plans"][0]["filename"], "active-plan-2026-04-08.md");
    assert_eq!(listed["plans"][0]["is_superseded"], false);
    assert_eq!(listed["plans"][1]["path"], "archive/old-plan.txt");
    assert_eq!(listed["plans"][1]["title"], "old-plan.txt");
    assert_eq!(listed["plans"][1]["is_superseded"], true);
    assert!(listed["plans"][0]["updated_at"].as_str().unwrap().contains('T'));

    let read_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/projects/{}/plans/file?path=active-plan-2026-04-08.md", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], "active-plan-2026-04-08.md");
    assert_eq!(read["mime_type"], "text/markdown");
    assert_eq!(read["is_previewable"], true);
    assert!(read["content"].as_str().unwrap().contains("Ship the project plans browser."));
  });
}

#[test]
fn project_plan_routes_reject_invalid_and_escaped_paths() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    for path in ["", "..%2Fsecret.md", "%2Ftmp%2Fsecret.md"] {
      let response = app
        .clone()
        .oneshot(
          request_with_employee(
            Request::builder()
              .method("GET")
              .uri(format!("/api/v1/projects/{}/plans/file?path={}", context.project_id, path)),
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
    }
  });
}

#[test]
fn project_plan_routes_detect_superseded_only_from_explicit_markers() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let project_plans_root =
      context._home.path().join(".blprnt").join("projects").join(&context.project_id).join("plans");
    fs::create_dir_all(&project_plans_root).unwrap();

    fs::write(project_plans_root.join("frontmatter.md"), "---\nstatus: superseded\n---\n# Old plan").unwrap();
    fs::write(project_plans_root.join("bool-frontmatter.md"), "---\nsuperseded: true\n---\n# Older plan").unwrap();
    fs::write(project_plans_root.join("body-marker.md"), "# Plan title\n\n**Status:** Superseded").unwrap();
    fs::write(project_plans_root.join("active.md"), "# Superseded naming discussion only\n\nThis plan remains active.")
      .unwrap();

    let app = test_app();
    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/projects/{}/plans", context.project_id)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    let plans = payload["plans"].as_array().unwrap();

    let by_path = |target: &str| {
      plans.iter().find(|plan| plan["path"].as_str() == Some(target)).unwrap()["is_superseded"].as_bool().unwrap()
    };

    assert!(by_path("frontmatter.md"));
    assert!(by_path("bool-frontmatter.md"));
    assert!(by_path("body-marker.md"));
    assert!(!by_path("active.md"));
  });
}

#[test]
fn memory_routes_support_employee_memory_list_read_and_search_flow() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let employee_memory_root =
      context._home.path().join(".blprnt").join("employees").join(&context.employee_id).join("memory");
    fs::create_dir_all(&employee_memory_root).unwrap();
    fs::write(employee_memory_root.join("2026-03-31.md"), "# Runtime Notes\n\nTrack provider interruptions.").unwrap();
    fs::write(employee_memory_root.join("2026-03-30.md"), "# Runtime Notes\n\nAsk-question flow is now covered.")
      .unwrap();
    let app = test_app();

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
    assert_eq!(listed["root_path"], "$AGENT_HOME/memory");
    assert_eq!(
      listed["nodes"],
      serde_json::json!([
        {
          "type": "file",
          "name": "2026-03-31.md",
          "path": "2026-03-31.md",
        },
        {
          "type": "file",
          "name": "2026-03-30.md",
          "path": "2026-03-30.md",
        }
      ])
    );
    assert!(
      context
        ._home
        .path()
        .join(".blprnt")
        .join("employees")
        .join(&context.employee_id)
        .join("memory")
        .join("2026-03-31.md")
        .is_file()
    );

    let read_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri("/api/v1/employees/me/memory/file?path=2026-03-31.md"),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_response.status(), StatusCode::OK);
    let read = response_json(read_response).await;
    assert_eq!(read["path"], "2026-03-31.md");
    assert_eq!(read["content"], "# Runtime Notes\n\nTrack provider interruptions.");

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
    assert_eq!(search["memories"][0]["path"], "2026-03-30.md");
    assert_eq!(search["memories"][0]["content"], "# Runtime Notes\n\nAsk-question flow is now covered.");
  });
}

#[test]
fn protected_routes_accept_legacy_x_employee_id_header_alias() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let response = app
      .oneshot(
        request_with_employee_alias(
          Request::builder().method("GET").uri("/api/v1/employees/me/memory"),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  });
}

#[test]
fn employee_life_routes_list_read_and_update_employee_files() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let target = EmployeeRepository::create(EmployeeModel {
      name: "Life Target".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Life Target".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let employee_home = shared::paths::employee_home(&target.id.uuid().to_string());
    fs::create_dir_all(employee_home.join("memory").join("daily")).unwrap();
    fs::write(employee_home.join("AGENTS.md"), "# Agent Rules\n").unwrap();
    fs::write(employee_home.join("memory").join("SUMMARY.md"), "# Summary\n").unwrap();
    fs::write(employee_home.join("memory").join("daily").join("2026-04-01.md"), "# Daily\n").unwrap();

    let tree_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/employees/{}/life", target.id.uuid())),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(tree_response.status(), StatusCode::OK);
    let tree = response_json(tree_response).await;
    assert_eq!(tree["root_path"], "$AGENT_HOME");
    assert_eq!(tree["nodes"][0]["path"], "HEARTBEAT.md");
    assert_eq!(tree["nodes"][0]["editable"], true);
    assert_eq!(tree["nodes"][2]["path"], "AGENTS.md");
    assert_eq!(tree["nodes"][4]["type"], "directory");
    assert_eq!(tree["nodes"][4]["path"], "memory");

    let read_doc_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/employees/{}/life/file?path=AGENTS.md", target.id.uuid())),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_doc_response.status(), StatusCode::OK);
    let read_doc = response_json(read_doc_response).await;
    assert_eq!(read_doc["kind"], "home_doc");
    assert_eq!(read_doc["editable"], true);
    assert_eq!(read_doc["content"], "# Agent Rules\n");

    let read_memory_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/employees/{}/life/file?path=memory/daily/2026-04-01.md", target.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(read_memory_response.status(), StatusCode::OK);
    let read_memory = response_json(read_memory_response).await;
    assert_eq!(read_memory["kind"], "memory");
    assert_eq!(read_memory["editable"], false);
    assert_eq!(read_memory["content"], "# Daily\n");

    let patch_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", target.id.uuid()))
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "path": "HEARTBEAT.md",
            "content": "# Focus\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(patch_response.status(), StatusCode::OK);
    let patched = response_json(patch_response).await;
    assert_eq!(patched["editable"], true);
    assert_eq!(fs::read_to_string(employee_home.join("HEARTBEAT.md")).unwrap(), "# Focus\n");

    let readonly_patch_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", target.id.uuid()))
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "path": "memory/SUMMARY.md",
            "content": "# Rewritten\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(readonly_patch_response.status(), StatusCode::BAD_REQUEST);
  });
}

#[test]
fn employee_life_routes_enforce_hierarchy_write_permissions() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();

    let ceo = EmployeeRepository::create(EmployeeModel {
      name: "CEO".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Ceo,
      title: "CEO".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();
    let manager = EmployeeRepository::create(EmployeeModel {
      name: "Manager".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Manager,
      title: "Manager".to_string(),
      reports_to: Some(ceo.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();
    let child_manager = EmployeeRepository::create(EmployeeModel {
      name: "Child Manager".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Manager,
      title: "Child Manager".to_string(),
      reports_to: Some(manager.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();
    let report = EmployeeRepository::create(EmployeeModel {
      name: "Report".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Staff,
      title: "Report".to_string(),
      reports_to: Some(child_manager.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();
    let outsider = EmployeeRepository::create(EmployeeModel {
      name: "Outsider".to_string(),
      kind: EmployeeKind::Person,
      role: EmployeeRole::Staff,
      title: "Outsider".to_string(),
      reports_to: Some(ceo.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();

    let manager_patch_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", report.id.uuid()))
            .header("content-type", "application/json"),
          &manager.id.uuid().to_string(),
        )
        .body(Body::from(
          serde_json::json!({
            "path": "TOOLS.md",
            "content": "manager update\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(manager_patch_response.status(), StatusCode::OK);

    let forbidden_manager_patch = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", outsider.id.uuid()))
            .header("content-type", "application/json"),
          &manager.id.uuid().to_string(),
        )
        .body(Body::from(
          serde_json::json!({
            "path": "TOOLS.md",
            "content": "nope\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(forbidden_manager_patch.status(), StatusCode::FORBIDDEN);

    let staff_self_patch = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", report.id.uuid()))
            .header("content-type", "application/json"),
          &report.id.uuid().to_string(),
        )
        .body(Body::from(
          serde_json::json!({
            "path": "SOUL.md",
            "content": "self edit\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(staff_self_patch.status(), StatusCode::OK);

    let staff_other_patch = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/employees/{}/life/file", outsider.id.uuid()))
            .header("content-type", "application/json"),
          &report.id.uuid().to_string(),
        )
        .body(Body::from(
          serde_json::json!({
            "path": "SOUL.md",
            "content": "other edit\n",
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(staff_other_patch.status(), StatusCode::FORBIDDEN);

    let staff_read_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/employees/{}/life/file?path=SOUL.md", outsider.id.uuid())),
          &report.id.uuid().to_string(),
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(staff_read_response.status(), StatusCode::OK);
    let staff_read = response_json(staff_read_response).await;
    assert_eq!(staff_read["editable"], false);
  });
}

#[test]
fn memory_routes_require_existing_projects() {
  let _lock = env_lock();
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
  let _lock = env_lock();
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
        .body(Body::from(
          r#"{"name":"Runtime Hardening","description":"Harden the runtime and provider integrations.","working_directories":["/tmp/runtime","/tmp/providers"]}"#,
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    assert_eq!(created["name"], "Runtime Hardening");
    assert_eq!(created["description"], "Harden the runtime and provider integrations.");
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
    assert_eq!(fetched["description"], "Harden the runtime and provider integrations.");
    assert_eq!(fetched["working_directories"], serde_json::json!(["/tmp/runtime", "/tmp/providers"]));
  });
}

#[cfg(debug_assertions)]
#[test]
fn dev_routes_nuke_database_requires_owner_permissions() {
  let _lock = env_lock();
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
  let _lock = env_lock();
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
    assert!(
      RunRepository::list(RunFilter { employee: None, issue: None, status: None, trigger: None })
        .await
        .unwrap()
        .is_empty()
    );
  });
}

#[test]
fn issue_routes_create_respects_explicit_status() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let _events = API_EVENTS.subscribe();

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
          Request::builder().method("POST").uri("/api/v1/issues").header("content-type", "application/json"),
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
fn issue_routes_create_starts_run_for_assigned_active_issue() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let payload = serde_json::json!({
      "title": "Kick off assignee run",
      "description": "Assigned active issues should start a run on creation.",
      "status": "todo",
      "priority": "high",
      "project": context.project_id,
      "assignee": context.employee_id
    })
    .to_string();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/issues").header("content-type", "application/json"),
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
    let issue_id = created["id"].as_str().expect("created issue should include an id");

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id.uuid().to_string(), context.employee_id);
        assert!(matches!(
          trigger,
          RunTrigger::IssueAssignment { issue_id: triggered_issue_id }
            if triggered_issue_id.uuid().to_string() == issue_id
        ));
      }
      event => panic!("unexpected event: {event:?}"),
    }
  });
}

#[test]
fn issue_routes_list_issue_runs_includes_assignment_and_mention_runs_sorted_newest_first() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Timeline issue".to_string(),
      description: "Should include associated runs.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let other_issue = IssueRepository::create(IssueModel {
      title: "Other issue".to_string(),
      description: "Should not match.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Low,
      ..Default::default()
    })
    .await
    .unwrap();

    let assignment_run = RunRepository::create(RunModel::new(
      employee_id.into(),
      RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
    ))
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let comment = IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
      issue.id.clone(),
      "Ping for timeline context".to_string(),
      vec![],
      employee_id.into(),
      None,
    ))
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let mention_run = RunRepository::create(RunModel::new(
      employee_id.into(),
      RunTrigger::IssueMention { issue_id: issue.id.clone(), comment_id: comment.id.clone() },
    ))
    .await
    .unwrap();

    RunRepository::create(RunModel::new(
      employee_id.into(),
      RunTrigger::IssueAssignment { issue_id: other_issue.id.clone() },
    ))
    .await
    .unwrap();

    RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/issues/{}/runs", issue.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected issue runs response: {payload}");

    let runs = payload.as_array().expect("issue runs should be an array");
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0]["id"], mention_run.id.uuid().to_string());
    assert_eq!(runs[0]["trigger"]["issue_mention"]["issue_id"], issue.id.id().to_string());
    assert_eq!(runs[0]["trigger"]["issue_mention"]["comment_id"], comment.id.id().to_string());
    assert_eq!(runs[1]["id"], assignment_run.id.uuid().to_string());
    assert_eq!(runs[1]["trigger"]["issue_assignment"]["issue_id"], issue.id.id().to_string());

    let first_created_at = runs[0]["created_at"].as_str().unwrap();
    let second_created_at = runs[1]["created_at"].as_str().unwrap();
    assert!(
      first_created_at >= second_created_at,
      "expected newest-first ordering, got {first_created_at} then {second_created_at}"
    );
  });
}

#[test]
fn issue_routes_emit_issue_events_for_all_mutations() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let _api_events = API_EVENTS.subscribe();
    let mut issue_events = ISSUE_EVENTS.subscribe();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/issues").header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "title": "Realtime issue",
            "description": "Exercises issue stream events.",
            "status": "todo",
            "priority": "high",
            "project": context.project_id
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    let issue_id = created["id"].as_str().unwrap().to_string();

    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::Created);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let update_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/issues/{issue_id}"))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(serde_json::json!({ "title": "Realtime issue updated" }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::Updated);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let comment_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{issue_id}/comments"))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(serde_json::json!({ "comment": "Realtime note" }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(comment_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::CommentAdded);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let attachment_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{issue_id}/attachments"))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": "note.txt",
            "attachment_kind": "file",
            "attachment": "data:text/plain;base64,SGVsbG8=",
            "mime_kind": "text/plain",
            "size": 5
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(attachment_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::AttachmentAdded);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let assignee = EmployeeRepository::create(EmployeeModel {
      name: "Realtime Assignee".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Realtime Assignee".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let assign_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{issue_id}/assign"))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(serde_json::json!({ "employee_id": assignee.id.uuid().to_string() }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(assign_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::Assigned);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let unassign_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri(format!("/api/v1/issues/{issue_id}/unassign")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(unassign_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::Unassigned);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let checkout_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri(format!("/api/v1/issues/{issue_id}/checkout")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(checkout_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::CheckedOut);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);

    let release_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri(format!("/api/v1/issues/{issue_id}/release")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(release_response.status(), StatusCode::OK);
    let event = issue_events.recv().await.unwrap();
    assert_eq!(event.kind, IssueEventKind::Released);
    assert_eq!(event.issue_id.uuid().to_string(), issue_id);
  });
}

#[test]
fn issue_routes_get_attachment_returns_full_attachment_payload() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let create_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/issues").header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "title": "Attachment detail issue",
            "description": "Fetches one attachment after issue load.",
            "status": "todo",
            "priority": "medium",
            "project": context.project_id
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    let issue_id = created["id"].as_str().unwrap().to_string();

    let attachment_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{issue_id}/attachments"))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": "note.txt",
            "attachment_kind": "file",
            "attachment": "data:text/plain;base64,SGVsbG8=",
            "mime_kind": "text/plain",
            "size": 5
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(attachment_response.status(), StatusCode::OK);
    let attachment = response_json(attachment_response).await;
    let attachment_id = attachment["id"].as_str().unwrap().to_string();

    let fetch_attachment_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/issues/{issue_id}/attachments/{attachment_id}")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let status = fetch_attachment_response.status();
    let payload = response_json(fetch_attachment_response).await;

    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["id"], attachment_id);
    assert_eq!(payload["attachment"]["name"], "note.txt");
    assert_eq!(payload["attachment"]["attachment_kind"], "file");
    assert_eq!(payload["attachment"]["attachment"], "data:text/plain;base64,SGVsbG8=");
    assert_eq!(payload["attachment"]["mime_kind"], "text/plain");
    assert_eq!(payload["attachment"]["size"], 5);
  });
}

#[test]
fn issue_routes_patch_update_nullable_fields_and_record_action() {
  let _lock = env_lock();
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
fn issue_routes_create_persists_labels() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/issues").header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "title": "Label-aware create issue",
            "description": "Verify labels persist during issue creation.",
            "labels": [
              { "name": "backend", "color": "blue" },
              { "name": "api", "color": "green" }
            ],
            "status": "todo",
            "priority": "medium"
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(
      payload["labels"],
      serde_json::json!([
        { "name": "backend", "color": "blue" },
        { "name": "api", "color": "green" }
      ])
    );

    let issue_id = persistence::Uuid::parse_str(payload["id"].as_str().unwrap()).unwrap();
    let stored = IssueRepository::get(issue_id.into()).await.unwrap();
    assert_eq!(
      stored.labels,
      Some(vec![
        IssueLabel { name: "backend".to_string(), color: "blue".to_string() },
        IssueLabel { name: "api".to_string(), color: "green".to_string() },
      ])
    );
  });
}

#[test]
fn issue_routes_assign_clears_existing_checkout_before_handoff() {
  let _lock = env_lock();
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
  let _lock = env_lock();
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
  let _lock = env_lock();
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
fn issue_routes_list_issues_accepts_repeated_expected_statuses_params() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let assignee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();

    let todo_issue = IssueRepository::create(IssueModel {
      title: "Todo issue".to_string(),
      description: "Should be returned by repeated expected_statuses filter.".to_string(),
      status: IssueStatus::Todo,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let in_progress_issue = IssueRepository::create(IssueModel {
      title: "In progress issue".to_string(),
      description: "Should be returned by repeated expected_statuses filter.".to_string(),
      status: IssueStatus::InProgress,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let _done_issue = IssueRepository::create(IssueModel {
      title: "Done issue".to_string(),
      description: "Should not be returned by repeated expected_statuses filter.".to_string(),
      status: IssueStatus::Done,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!(
            "/api/v1/issues?expected_statuses=todo&expected_statuses=in_progress&assignee={}",
            context.employee_id
          )),
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
    assert_eq!(issues.len(), 2);

    let issue_ids = issues
      .iter()
      .map(|issue| issue["id"].as_str().expect("issue id").to_string())
      .collect::<std::collections::HashSet<_>>();

    assert!(issue_ids.contains(&todo_issue.id.uuid().to_string()));
    assert!(issue_ids.contains(&in_progress_issue.id.uuid().to_string()));
  });
}

#[test]
fn issue_routes_list_issues_accepts_bracketed_expected_statuses_params() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let assignee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let project_id = persistence::Uuid::parse_str(&context.project_id).unwrap();

    let blocked_issue = IssueRepository::create(IssueModel {
      title: "Blocked issue".to_string(),
      description: "Should be returned by bracketed expected_statuses filter.".to_string(),
      status: IssueStatus::Blocked,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let _done_issue = IssueRepository::create(IssueModel {
      title: "Completed issue".to_string(),
      description: "Should not be returned by bracketed expected_statuses filter.".to_string(),
      status: IssueStatus::Done,
      project: Some(project_id.into()),
      assignee: Some(assignee_id.into()),
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("GET")
            .uri(format!("/api/v1/issues?expected_statuses[]=blocked&assignee={}", context.employee_id)),
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
    assert_eq!(issues[0]["id"], blocked_issue.id.uuid().to_string());
    assert_eq!(issues[0]["status"], "blocked");
  });
}

#[test]
fn employee_routes_require_update_permissions() {
  let _lock = env_lock();
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
      "timer_wakeups_enabled": true,
      "max_concurrent_runs": 1
    }
  })
  .to_string()
}

#[test]
fn employee_routes_create_employee_writes_optional_instruction_docs() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();
    let owner_id = create_owner().await;
    let _events = API_EVENTS.subscribe();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "name": "Doc Writer",
            "kind": "agent",
            "role": "staff",
            "title": "Doc Writer",
            "icon": "bot",
            "color": "#3b82f6",
            "capabilities": ["write docs"],
            "provider_config": {
              "provider": "mock",
              "slug": "doc-writer"
            },
            "runtime_config": {
              "heartbeat_interval_sec": 1800,
              "heartbeat_prompt": "Write docs.",
              "wake_on_demand": true,
              "timer_wakeups_enabled": false,
              "max_concurrent_runs": 1
            },
            "heartbeat_md": "Check in every 30 minutes.\n",
            "soul_md": "Be practical.\n",
            "agents_md": "You are Doc Writer.\n",
            "tools_md": "Use the API.\n"
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected create employee response: {payload}");
    assert_eq!(payload["runtime_config"]["timer_wakeups_enabled"], false);

    let employee_id = payload["id"].as_str().expect("employee id");
    let employee_home = shared::paths::employee_home(employee_id);
    assert_eq!(fs::read_to_string(employee_home.join("HEARTBEAT.md")).unwrap(), "Check in every 30 minutes.\n");
    assert_eq!(fs::read_to_string(employee_home.join("SOUL.md")).unwrap(), "Be practical.\n");
    assert_eq!(fs::read_to_string(employee_home.join("AGENTS.md")).unwrap(), "You are Doc Writer.\n");
    assert_eq!(fs::read_to_string(employee_home.join("TOOLS.md")).unwrap(), "Use the API.\n");
  });
}

#[test]
fn employee_routes_create_employee_skips_unset_instruction_docs() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();
    let owner_id = create_owner().await;
    let _events = API_EVENTS.subscribe();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/employees").header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(create_agent_employee_payload("No Docs", "staff")))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected create employee response: {payload}");

    let employee_id = payload["id"].as_str().expect("employee id");
    let employee_home = shared::paths::employee_home(employee_id);
    assert!(!employee_home.join("HEARTBEAT.md").exists());
    assert!(!employee_home.join("SOUL.md").exists());
    assert!(!employee_home.join("AGENTS.md").exists());
    assert!(!employee_home.join("TOOLS.md").exists());
  });
}

#[test]
fn issue_comment_mentions_store_metadata_and_emit_one_run_per_unique_employee() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let mentioned = EmployeeRepository::create(EmployeeModel {
      name: "Mentioned Engineer".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Mentioned Engineer".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle mention-triggered work.".to_string(),
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
    .unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Mentionable issue".to_string(),
      description: "Exercise issue comment mentions.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "Please review with @Mentioned Engineer twice: @Mentioned Engineer",
            "mentions": [
              {
                "employee_id": mentioned.id.uuid().to_string(),
                "label": "Mentioned Engineer"
              },
              {
                "employee_id": mentioned.id.uuid().to_string(),
                "label": "Mentioned Engineer"
              }
            ]
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");
    assert_eq!(payload["mentions"].as_array().unwrap().len(), 2);
    assert_eq!(payload["mentions"][0]["employee_id"], mentioned.id.uuid().to_string());
    assert_eq!(payload["mentions"][0]["label"], "Mentioned Engineer");

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id, mentioned.id);
        assert!(matches!(trigger, RunTrigger::IssueMention { .. }));
        match trigger {
          RunTrigger::IssueMention { issue_id, comment_id } => {
            assert_eq!(issue_id, issue.id);
            assert_eq!(comment_id.uuid().to_string(), payload["id"].as_str().unwrap());
          }
          _ => unreachable!(),
        }
      }
      event => panic!("unexpected API event: {event:?}"),
    }

    assert!(tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await.is_err());
  });
}

#[test]
fn issue_comment_mentions_skip_employee_already_assigned_in_same_run() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let assignee = EmployeeRepository::create(EmployeeModel {
      name: "Assigned Mentioned Engineer".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Assigned Mentioned Engineer".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle assignment-triggered work.".to_string(),
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
    .unwrap();

    let actor_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let issue = IssueRepository::create(IssueModel {
      title: "Assignment mention dedupe".to_string(),
      description: "Do not double-trigger the assignee in the same action phase.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Critical,
      ..Default::default()
    })
    .await
    .unwrap();

    let run = RunRepository::create(RunModel::new(actor_id.into(), RunTrigger::Manual)).await.unwrap();
    let run_id = run.id.uuid().to_string();

    let assign_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/assign", issue.id.uuid()))
            .header("content-type", "application/json")
            .header("x-blprnt-run-id", &run_id),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "employee_id": assignee.id.uuid().to_string()
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(assign_response.status(), StatusCode::OK);

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id, assignee.id);
        assert_eq!(trigger, RunTrigger::IssueAssignment { issue_id: issue.id.clone() });
      }
      event => panic!("unexpected API event after assignment: {event:?}"),
    }

    let comment_response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json")
            .header("x-blprnt-run-id", &run_id),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "@Assigned Mentioned Engineer please take the next implementation step.",
            "mentions": [
              {
                "employee_id": assignee.id.uuid().to_string(),
                "label": "Assigned Mentioned Engineer"
              }
            ]
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = comment_response.status();
    let payload = response_json(comment_response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");
    assert_eq!(payload["mentions"].as_array().unwrap().len(), 1);

    assert!(tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await.is_err());
  });
}

#[test]
fn issue_comment_reopen_starts_assignment_run_and_dedupes_matching_mention() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let assignee = EmployeeRepository::create(EmployeeModel {
      name: "Reopened Assignee".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Reopened Assignee".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle reopened assigned work.".to_string(),
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
    .unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Reopen assigned issue".to_string(),
      description: "Reopening with a comment should wake the assignee once.".to_string(),
      status: IssueStatus::Done,
      priority: IssuePriority::High,
      assignee: Some(assignee.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "@Reopened Assignee please pick this back up.",
            "reopen_issue": true,
            "mentions": [
              {
                "employee_id": assignee.id.uuid().to_string(),
                "label": "Reopened Assignee"
              }
            ]
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id, assignee.id);
        assert_eq!(trigger, RunTrigger::IssueAssignment { issue_id: issue.id.clone() });
      }
      event => panic!("unexpected API event after reopen comment: {event:?}"),
    }

    assert!(tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await.is_err());

    let stored = IssueRepository::get(issue.id.clone()).await.unwrap();
    assert_eq!(stored.status, IssueStatus::Todo);
  });
}

#[test]
fn issue_comment_reopen_starts_assignment_run_and_other_mentions_still_trigger() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let assignee = EmployeeRepository::create(EmployeeModel {
      name: "Reopened Primary".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Reopened Primary".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle reopened assigned work.".to_string(),
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
    .unwrap();

    let mentioned = EmployeeRepository::create(EmployeeModel {
      name: "Secondary Mention".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Secondary Mention".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle mention-triggered work.".to_string(),
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
    .unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Reopen issue with another mention".to_string(),
      description: "Reopening should wake the assignee and distinct mentions.".to_string(),
      status: IssueStatus::Done,
      priority: IssuePriority::High,
      assignee: Some(assignee.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "@Reopened Primary please resume and @Secondary Mention please review.",
            "reopen_issue": true,
            "mentions": [
              {
                "employee_id": assignee.id.uuid().to_string(),
                "label": "Reopened Primary"
              },
              {
                "employee_id": mentioned.id.uuid().to_string(),
                "label": "Secondary Mention"
              }
            ]
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");

    let first = events.recv().await.unwrap();
    let second = events.recv().await.unwrap();

    let mut seen_assignment = false;
    let mut seen_mention = false;
    for event in [first, second] {
      match event {
        ApiEvent::StartRun { employee_id, trigger, .. } => match trigger {
          RunTrigger::IssueAssignment { issue_id } => {
            assert_eq!(employee_id, assignee.id);
            assert_eq!(issue_id, issue.id);
            seen_assignment = true;
          }
          RunTrigger::IssueMention { issue_id, .. } => {
            assert_eq!(employee_id, mentioned.id);
            assert_eq!(issue_id, issue.id);
            seen_mention = true;
          }
          other => panic!("unexpected trigger after reopen comment: {other:?}"),
        },
        other => panic!("unexpected API event after reopen comment: {other:?}"),
      }
    }

    assert!(seen_assignment);
    assert!(seen_mention);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await.is_err());
  });
}

#[test]
fn issue_comment_mentions_skip_self_paused_and_non_wake_employees() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let paused = EmployeeRepository::create(EmployeeModel {
      name: "Paused Mention".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Paused Mention".to_string(),
      status: EmployeeStatus::Paused,
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Paused.".to_string(),
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
    .unwrap();

    let non_wake = EmployeeRepository::create(EmployeeModel {
      name: "Non Wake Mention".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Non Wake Mention".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "No wake.".to_string(),
        wake_on_demand:         false,
        timer_wakeups_enabled:  Some(true),
        dreams_enabled:         Some(false),
        max_concurrent_runs:    1,
        skill_stack:            None,
        reasoning_effort:       None,
      }),
      ..Default::default()
    })
    .await
    .unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Skip mention runs".to_string(),
      description: "Ensure comment creation still succeeds.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "Notify @Memory Tester, @Paused Mention, and @Non Wake Mention",
            "mentions": [
              {
                "employee_id": context.employee_id,
                "label": "Memory Tester"
              },
              {
                "employee_id": paused.id.uuid().to_string(),
                "label": "Paused Mention"
              },
              {
                "employee_id": non_wake.id.uuid().to_string(),
                "label": "Non Wake Mention"
              }
            ]
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");
    assert_eq!(payload["mentions"].as_array().unwrap().len(), 3);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await.is_err());
  });
}

#[test]
fn issue_comment_mentions_infer_plain_text_mentions_and_emit_run() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let cto = EmployeeRepository::create(EmployeeModel {
      name: "CTO".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Manager,
      title: "CTO".to_string(),
      runtime_config: Some(EmployeeRuntimeConfig {
        heartbeat_interval_sec: 1800,
        heartbeat_prompt:       "Handle manager review requests.".to_string(),
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
    .unwrap();

    let issue = IssueRepository::create(IssueModel {
      title: "Infer plain text mentions".to_string(),
      description: "Raw @Name mentions from API comments should still trigger runs.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Medium,
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri(format!("/api/v1/issues/{}/comments", issue.id.uuid()))
            .header("content-type", "application/json"),
          &context.employee_id,
        )
        .body(Body::from(
          serde_json::json!({
            "comment": "@CTO\n\n## Done\n\nPlease review the completed fix.",
            "mentions": []
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected comment response: {payload}");
    assert_eq!(payload["mentions"].as_array().unwrap().len(), 1);
    assert_eq!(payload["mentions"][0]["employee_id"], cto.id.uuid().to_string());
    assert_eq!(payload["mentions"][0]["label"], "CTO");

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, trigger, .. } => {
        assert_eq!(employee_id, cto.id);
        assert!(matches!(trigger, RunTrigger::IssueMention { .. }));
      }
      event => panic!("unexpected API event: {event:?}"),
    }
  });
}

#[test]
fn issue_routes_my_work_groups_assigned_and_mentioned_with_terminal_filtering() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let project = ProjectRepository::create(ProjectModel::new(
      "My Work Project".to_string(),
      "Queue aggregation test project".to_string(),
      vec![],
    ))
    .await
    .unwrap();

    let assigned_visible = IssueRepository::create(IssueModel {
      title: "Assigned visible".to_string(),
      description: "Should appear in assigned".to_string(),
      status: IssueStatus::InProgress,
      priority: IssuePriority::High,
      project: Some(project.id.clone()),
      assignee: Some(persistence::Uuid::parse_str(&context.employee_id).unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();

    let _assigned_done = IssueRepository::create(IssueModel {
      title: "Assigned done".to_string(),
      description: "Should be excluded".to_string(),
      status: IssueStatus::Done,
      priority: IssuePriority::Medium,
      assignee: Some(persistence::Uuid::parse_str(&context.employee_id).unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();

    let mentioned_issue = IssueRepository::create(IssueModel {
      title: "Mentioned issue".to_string(),
      description: "Should appear in mentioned".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Critical,
      project: Some(project.id.clone()),
      ..Default::default()
    })
    .await
    .unwrap();

    let actor = EmployeeRepository::create(EmployeeModel {
      name: "Mention Actor".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Mention Actor".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
      mentioned_issue.id.clone(),
      "@Memory Tester please check the latest API slice before frontend wiring because this payload is now stable enough for integration.".to_string(),
      vec![persistence::prelude::IssueCommentMention {
        employee_id: persistence::Uuid::parse_str(&context.employee_id).unwrap().into(),
        label: "Memory Tester".to_string(),
      }],
      actor.id.clone(),
      None,
    ))
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/issues/my-work"), &context.employee_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected my-work response: {payload}");

    let assigned = payload["assigned"].as_array().unwrap();
    assert_eq!(assigned.len(), 1);
    assert_eq!(assigned[0]["issue_id"], assigned_visible.id.uuid().to_string());
    assert_eq!(assigned[0]["reason"], "assigned");
    assert_eq!(assigned[0]["project_id"], project.id.uuid().to_string());
    assert_eq!(assigned[0]["project_name"], "My Work Project");
    assert!(assigned[0]["comment_id"].is_null());
    assert!(assigned[0]["comment_snippet"].is_null());

    let mentioned = payload["mentioned"].as_array().unwrap();
    assert_eq!(mentioned.len(), 1);
    assert_eq!(mentioned[0]["issue_id"], mentioned_issue.id.uuid().to_string());
    assert_eq!(mentioned[0]["reason"], "mentioned");
    assert_eq!(mentioned[0]["project_name"], "My Work Project");
    assert!(mentioned[0]["comment_id"].is_string());
    assert!(mentioned[0]["comment_snippet"].as_str().unwrap().contains("@Memory Tester"));
  });
}

#[test]
fn issue_routes_my_work_deduplicates_mentions_and_orders_by_newest_comment() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let actor = EmployeeRepository::create(EmployeeModel {
      name: "Mention Actor".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Mention Actor".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let older_issue = IssueRepository::create(IssueModel {
      title: "Older mention issue".to_string(),
      description: "Has two mentions; newest should win".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Medium,
      ..Default::default()
    })
    .await
    .unwrap();

    let newer_issue = IssueRepository::create(IssueModel {
      title: "Newer mention issue".to_string(),
      description: "Newest mention overall should sort first".to_string(),
      status: IssueStatus::Blocked,
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let target_mention = persistence::prelude::IssueCommentMention {
      employee_id: persistence::Uuid::parse_str(&context.employee_id).unwrap().into(),
      label: "Memory Tester".to_string(),
    };

    let first_comment = IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
      older_issue.id.clone(),
      "@Memory Tester first mention".to_string(),
      vec![target_mention.clone()],
      actor.id.clone(),
      None,
    ))
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let latest_older_comment = IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
      older_issue.id.clone(),
      "@Memory Tester second mention should be the one returned for this issue because it is newer and should dedupe the older row.".to_string(),
      vec![target_mention.clone()],
      actor.id.clone(),
      None,
    ))
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let newest_comment = IssueRepository::add_comment(persistence::prelude::IssueCommentModel::new(
      newer_issue.id.clone(),
      "@Memory Tester newest mention overall".to_string(),
      vec![target_mention],
      actor.id.clone(),
      None,
    ))
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/issues/my-work"), &context.employee_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected my-work response: {payload}");

    let mentioned = payload["mentioned"].as_array().unwrap();
    assert_eq!(mentioned.len(), 2);
    assert_eq!(mentioned[0]["issue_id"], newer_issue.id.uuid().to_string());
    assert_eq!(mentioned[0]["comment_id"], newest_comment.id.uuid().to_string());
    assert_eq!(mentioned[1]["issue_id"], older_issue.id.uuid().to_string());
    assert_eq!(mentioned[1]["comment_id"], latest_older_comment.id.uuid().to_string());
    assert_ne!(mentioned[1]["comment_id"], first_comment.id.uuid().to_string());
    assert!(mentioned[1]["comment_snippet"].as_str().unwrap().contains("second mention should be the one returned"));
  });
}

#[test]
fn issue_routes_my_work_orders_assigned_by_most_recent_issue_activity() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let app = test_app();

    let older_issue = IssueRepository::create(IssueModel {
      title: "Assigned older".to_string(),
      description: "Older assigned issue".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::Medium,
      assignee: Some(persistence::Uuid::parse_str(&context.employee_id).unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let newer_issue = IssueRepository::create(IssueModel {
      title: "Assigned newer".to_string(),
      description: "Newer assigned issue".to_string(),
      status: IssueStatus::InProgress,
      priority: IssuePriority::High,
      assignee: Some(persistence::Uuid::parse_str(&context.employee_id).unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/issues/my-work"), &context.employee_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected my-work response: {payload}");

    let assigned = payload["assigned"].as_array().unwrap();
    assert_eq!(assigned.len(), 2);
    assert_eq!(assigned[0]["issue_id"], newer_issue.id.uuid().to_string());
    assert_eq!(assigned[1]["issue_id"], older_issue.id.uuid().to_string());
  });
}

#[test]
fn issue_routes_my_work_builds_mention_focused_snippets() {
  let snippet = build_comment_snippet_for_labels(
    "The API contract is stable now and the payload review is done, so @Memory Tester can verify the UI wiring next before merge once the final pass lands.",
    &["Memory Tester"],
  );

  assert!(snippet.contains("@Memory Tester"));
  assert!(snippet.starts_with('…') || snippet.starts_with("The API contract"));
  assert!(snippet.chars().count() <= 112);
}

#[test]
fn employee_routes_ceo_can_hire_manager_and_staff() {
  let _lock = env_lock();
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
  let _lock = env_lock();
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
  let _lock = env_lock();
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
  let _lock = env_lock();
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
  let _lock = env_lock();
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
  let _lock = env_lock();
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
fn run_routes_trigger_accepts_uuid_employee_id_payload() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let employee = EmployeeRepository::create(EmployeeModel {
      name: "Triggered Employee".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Triggered Employee".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let expected_employee_id = employee.id.clone();
    let response_task = tokio::spawn(async move {
      app
        .oneshot(
          request_with_employee(
            Request::builder().method("POST").uri("/api/v1/runs").header("content-type", "application/json"),
            &owner_id,
          )
          .body(Body::from(serde_json::json!({ "employee_id": employee.id.uuid().to_string() }).to_string()))
          .unwrap(),
        )
        .await
        .unwrap()
    });

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, run_id, trigger, rx } => {
        assert_eq!(employee_id, expected_employee_id);
        assert!(run_id.is_none());
        assert!(matches!(trigger, RunTrigger::Manual));

        let run = RunRepository::create(RunModel::new(employee_id.clone(), RunTrigger::Manual)).await.unwrap();
        let sender = rx.unwrap();
        let sender = sender.lock().await.take().unwrap();
        sender.send(Ok(Some(run))).unwrap();
      }
      event => panic!("unexpected event: {event:?}"),
    }

    let response = response_task.await.unwrap();
    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected trigger run response: {payload}");
    assert_eq!(payload["employee_id"], expected_employee_id.uuid().to_string());
  });
}

#[test]
fn run_routes_get_by_uuid_path() {
  let _lock = env_lock();
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

#[test]
fn run_routes_expose_usage_metrics_on_run_and_issue_summary_responses() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let issue = IssueRepository::create(IssueModel {
      title: "Usage issue".to_string(),
      description: "Usage metrics should serialize through the API.".to_string(),
      status: IssueStatus::Todo,
      priority: IssuePriority::High,
      ..Default::default()
    })
    .await
    .unwrap();

    let run = RunRepository::create(RunModel::new(
      employee_id.into(),
      RunTrigger::IssueAssignment { issue_id: issue.id.clone() },
    ))
    .await
    .unwrap();

    let turn = TurnRepository::create(TurnModel {
      run_id: run.id.clone(),
      steps: vec![persistence::prelude::TurnStep {
        request:      persistence::prelude::TurnStepContents {
          contents: vec![persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
            text:       "Prompt".to_string(),
            signature:  None,
            visibility: persistence::prelude::ContentsVisibility::Full,
          })],
          role:     persistence::prelude::TurnStepRole::User,
        },
        response:     persistence::prelude::TurnStepContents {
          contents: vec![persistence::prelude::TurnStepContent::Text(persistence::prelude::TurnStepText {
            text:       "Response".to_string(),
            signature:  None,
            visibility: persistence::prelude::ContentsVisibility::Full,
          })],
          role:     persistence::prelude::TurnStepRole::Assistant,
        },
        status:       persistence::prelude::TurnStepStatus::Completed,
        usage:        persistence::prelude::UsageMetrics {
          provider:                   Some(shared::agent::Provider::OpenAi),
          model:                      Some("gpt-5-test".to_string()),
          input_tokens:               Some(10),
          output_tokens:              Some(6),
          total_tokens:               Some(16),
          estimated_cost_usd:         Some(0.0016),
          has_unavailable_token_data: false,
          has_unavailable_cost_data:  false,
        },
        created_at:   chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
      }],
      ..Default::default()
    })
    .await
    .unwrap();

    let _ = turn;
    let run = RunRepository::update(run.id, persistence::prelude::RunStatus::Completed).await.unwrap();
    let run_id = run.id.uuid().to_string();

    let run_response = app
      .clone()
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri(format!("/api/v1/runs/{run_id}")), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(run_response.status(), StatusCode::OK);
    let run_payload = response_json(run_response).await;
    assert_eq!(run_payload["usage"]["provider"], "openai");
    assert_eq!(run_payload["usage"]["model"], "gpt-5-test");
    assert_eq!(run_payload["usage"]["total_tokens"], 16);
    assert_eq!(run_payload["usage"]["estimated_cost_usd"], 0.0016);
    assert_eq!(run_payload["turns"][0]["usage"]["total_tokens"], 16);
    assert_eq!(run_payload["turns"][0]["steps"][0]["usage"]["total_tokens"], 16);

    let issue_runs_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/issues/{}/runs", issue.id.uuid())),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(issue_runs_response.status(), StatusCode::OK);
    let issue_runs_payload = response_json(issue_runs_response).await;
    assert_eq!(issue_runs_payload[0]["usage"]["provider"], "openai");
    assert_eq!(issue_runs_payload[0]["usage"]["total_tokens"], 16);
  });
}

#[test]
fn run_routes_trigger_accepts_conversation_prompt_payload() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let employee = EmployeeRepository::create(EmployeeModel {
      name: "Conversation Employee".to_string(),
      kind: EmployeeKind::Agent,
      role: EmployeeRole::Staff,
      title: "Conversation Employee".to_string(),
      ..Default::default()
    })
    .await
    .unwrap();

    let expected_employee_id = employee.id.clone();
    let response_task = tokio::spawn(async move {
      app
        .oneshot(
          request_with_employee(
            Request::builder().method("POST").uri("/api/v1/runs").header("content-type", "application/json"),
            &owner_id,
          )
          .body(Body::from(
            serde_json::json!({
              "employee_id": employee.id.uuid().to_string(),
              "trigger": "conversation",
              "prompt": "Help me plan the next sprint.",
              "reasoning_effort": "high"
            })
            .to_string(),
          ))
          .unwrap(),
        )
        .await
        .unwrap()
    });

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id, run_id, trigger, rx } => {
        assert_eq!(employee_id, expected_employee_id);
        assert!(run_id.is_some());
        assert!(matches!(trigger, RunTrigger::Conversation));
        assert!(rx.is_none());
      }
      event => panic!("unexpected event: {event:?}"),
    }

    let response = response_task.await.unwrap();
    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected conversation trigger response: {payload}");
    assert_eq!(payload["employee_id"], expected_employee_id.uuid().to_string());
    assert_eq!(payload["trigger"], "conversation");
    assert_eq!(payload["turns"].as_array().unwrap().len(), 1);
    assert_eq!(payload["turns"][0]["reasoning_effort"], "high");
    assert_eq!(
      payload["turns"][0]["steps"][0]["request"]["contents"][0]["Text"]["text"],
      "Help me plan the next sprint."
    );
  });
}

#[test]
fn run_routes_append_message_emits_start_run_for_existing_run() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();
    let run = RunRepository::update(run.id, persistence::prelude::RunStatus::Completed).await.unwrap();
    let run_id = run.id.uuid().to_string();
    let request_run_id = run_id.clone();

    let response_task = tokio::spawn(async move {
      app
        .oneshot(
          request_with_employee(
            Request::builder()
              .method("POST")
              .uri(format!("/api/v1/runs/{request_run_id}/messages"))
              .header("content-type", "application/json"),
            &owner_id,
          )
          .body(Body::from(
            serde_json::json!({
              "prompt": "Continue from where you left off.",
              "reasoning_effort": "minimal"
            })
            .to_string(),
          ))
          .unwrap(),
        )
        .await
        .unwrap()
    });

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id: event_employee_id, run_id: event_run_id, trigger, rx } => {
        assert_eq!(event_employee_id.uuid().to_string(), context.employee_id);
        assert_eq!(event_run_id.unwrap().uuid().to_string(), run_id);
        assert!(matches!(trigger, RunTrigger::Manual));
        assert!(rx.is_none());
      }
      event => panic!("unexpected event: {event:?}"),
    }

    let response = response_task.await.unwrap();
    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected append message response: {payload}");
    assert_eq!(payload["id"], run_id);
    assert_eq!(payload["trigger"], "manual");
    assert_eq!(payload["turns"].as_array().unwrap().len(), 1);
    assert_eq!(payload["turns"][0]["reasoning_effort"], "minimal");
    assert_eq!(
      payload["turns"][0]["steps"][0]["request"]["contents"][0]["Text"]["text"],
      "Continue from where you left off."
    );

    let updated_run = RunRepository::get(run.id).await.unwrap();
    assert_eq!(updated_run.turns[0].reasoning_effort, Some(ReasoningEffort::Minimal));
  });
}

#[test]
fn run_routes_append_message_allows_failed_runs() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();
    let mut events = API_EVENTS.subscribe();

    let employee_id = persistence::Uuid::parse_str(&context.employee_id).unwrap();
    let run = RunRepository::create(RunModel::new(employee_id.into(), RunTrigger::Manual)).await.unwrap();
    let run = RunRepository::update(run.id, persistence::prelude::RunStatus::Failed("adapter crashed".to_string()))
      .await
      .unwrap();
    let run_id = run.id.uuid().to_string();
    let request_run_id = run_id.clone();

    let response_task = tokio::spawn(async move {
      app
        .oneshot(
          request_with_employee(
            Request::builder()
              .method("POST")
              .uri(format!("/api/v1/runs/{request_run_id}/messages"))
              .header("content-type", "application/json"),
            &owner_id,
          )
          .body(Body::from(
            serde_json::json!({
              "prompt": "Try again with a smaller change set."
            })
            .to_string(),
          ))
          .unwrap(),
        )
        .await
        .unwrap()
    });

    match events.recv().await.unwrap() {
      ApiEvent::StartRun { employee_id: event_employee_id, run_id: event_run_id, trigger, rx } => {
        assert_eq!(event_employee_id.uuid().to_string(), context.employee_id);
        assert_eq!(event_run_id.unwrap().uuid().to_string(), run_id);
        assert!(matches!(trigger, RunTrigger::Manual));
        assert!(rx.is_none());
      }
      event => panic!("unexpected event: {event:?}"),
    }

    let response = response_task.await.unwrap();
    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected append message response: {payload}");
    assert_eq!(payload["id"], run_id);
    assert_eq!(payload["status"], "Pending");
    assert_eq!(payload["turns"].as_array().unwrap().len(), 1);
    assert_eq!(
      payload["turns"][0]["steps"][0]["request"]["contents"][0]["Text"]["text"],
      "Try again with a smaller change set."
    );
  });
}

#[test]
fn scoped_openapi_routes_are_public_and_filtered() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let app = test_app();

    let response = app
      .clone()
      .oneshot(Request::builder().method("GET").uri("/api/v1/openapi.json").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = app
      .clone()
      .oneshot(Request::builder().method("GET").uri("/api/v1/issues/openapi.json").body(Body::empty()).unwrap())
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["openapi"], "3.1.0");
    assert_eq!(payload["info"]["title"], "blprnt API");
    assert_eq!(payload["servers"][0]["url"], "/api/v1");
    assert!(payload["paths"]["/issues"].is_object(), "{payload}");
    assert!(payload["paths"]["/issues/{issue_id}"].is_object(), "{payload}");
    assert!(payload["paths"]["/owner"].is_null(), "{payload}");
    assert!(payload["paths"]["/runs"].is_null(), "{payload}");
    assert!(payload["paths"]["/dev/database"].is_null(), "{payload}");
    let tags = payload["tags"].as_array().expect("openapi tags should be an array");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0]["name"], "issues");

    let response = app
      .oneshot(Request::builder().method("GET").uri("/api/v1/auth/openapi.json").body(Body::empty()).unwrap())
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;

    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert!(payload["paths"]["/auth/status"].is_object(), "{payload}");
    assert!(payload["paths"]["/auth/login"].is_object(), "{payload}");
    assert!(payload["paths"]["/owner"].is_null(), "{payload}");
    let tags = payload["tags"].as_array().expect("openapi tags should be an array");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0]["name"], "auth");
  });
}

#[test]
#[ignore = "obsolete webhook-era regression; replace with polling-only equivalent before removing ignore"]
fn telegram_link_code_can_be_claimed_via_webhook() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(Request::builder().method("POST").uri("/api/v1/integrations/telegram/config"), &owner_id)
          .header("content-type", "application/json")
          .body(Body::from(
            serde_json::json!({
              "bot_token": "bot-token",
              "webhook_secret": "hook-secret",
              "bot_username": "blprnt_bot",
              "webhook_url": "https://example.com/telegram",
              "delivery_mode": "webhook",
              "enabled": true
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected webhook response: {payload}");

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("POST").uri("/api/v1/integrations/telegram/link-codes"),
          &owner_id,
        )
        .header("content-type", "application/json")
        .body(Body::from(serde_json::json!({ "employee_id": owner_id }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let payload = response_json(response).await;
    let code = payload["code"].as_str().unwrap().to_string();

    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/integrations/telegram/webhook")
          .header("content-type", "application/json")
          .header("x-telegram-bot-api-secret-token", "hook-secret")
          .body(Body::from(
            serde_json::json!({
              "message": {
                "message_id": 7,
                "text": format!("/link {code}"),
                "chat": { "id": 123 },
                "from": { "id": 456 }
              }
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/integrations/telegram/links/{owner_id}")),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();
    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected links response: {payload}");
    assert_eq!(payload.as_array().unwrap().len(), 1);
    assert_eq!(payload[0]["telegram_chat_id"], 123);
    assert_eq!(payload[0]["telegram_user_id"], 456);
  });
}

#[test]
#[ignore = "obsolete webhook-era regression; replace with polling-only equivalent before removing ignore"]
fn telegram_webhook_rejects_invalid_secret() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(Request::builder().method("POST").uri("/api/v1/integrations/telegram/config"), &owner_id)
          .header("content-type", "application/json")
          .body(Body::from(
            serde_json::json!({
              "bot_token": "bot-token",
              "webhook_secret": "hook-secret",
              "bot_username": "blprnt_bot",
              "webhook_url": "https://example.com/telegram",
              "delivery_mode": "webhook",
              "enabled": true
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/integrations/telegram/webhook")
          .header("content-type", "application/json")
          .header("x-telegram-bot-api-secret-token", "wrong")
          .body(Body::from(
            serde_json::json!({
              "message": {
                "message_id": 1,
                "text": "/start",
                "chat": { "id": 1 },
                "from": { "id": 1 }
              }
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  });
}

#[test]
fn telegram_config_can_be_upserted_and_loaded() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "webhook_secret": "webhook-secret",
            "bot_username": "blprnt_bot",
            "webhook_url": "https://example.com/telegram/webhook",
            "delivery_mode": "webhook",
            "parse_mode": "html",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["bot_username"], "blprnt_bot");
    assert_eq!(payload["parse_mode"], "html");

    let response = app
      .oneshot(
        request_with_employee(Request::builder().method("GET").uri("/api/v1/integrations/telegram/config"), &owner_id)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["bot_username"], "blprnt_bot");
    assert_eq!(payload["enabled"], true);
  });
}

#[test]
fn telegram_polling_accepts_valid_link_flow_and_persists_link() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "parse_mode": "html",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/link-codes")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "employee_id": context.employee_id
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    let code = payload["code"].as_str().unwrap().to_string();

    crate::telegram::reset_polling_offset_for_tests();
    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 501,
        "text": format!("/link {code}"),
        "chat": { "id": 7001 },
        "from": { "id": 9001 }
      }),
    );

    crate::telegram::poll_once().await.unwrap();

    let links =
      TelegramLinkRepository::list_for_employee(context.employee_id.parse::<persistence::Uuid>().unwrap().into())
        .await
        .unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].telegram_chat_id, 7001);
    assert_eq!(links[0].telegram_user_id, 9001);

    let response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("GET").uri(format!("/api/v1/integrations/telegram/links/{}", context.employee_id)),
          &owner_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload.as_array().unwrap().len(), 1);

    let inbound = TelegramMessageCorrelationRepository::find_by_chat_message(7001, 501).await.unwrap().unwrap();
    assert_eq!(inbound.direction, TelegramMessageDirection::Inbound);
    assert_eq!(inbound.kind, TelegramCorrelationKind::LinkCode);
    assert_eq!(inbound.employee_id.unwrap().uuid().to_string(), context.employee_id);

    let sent_messages = _mock_state.sent_messages.lock().unwrap().clone();
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0]["chat_id"], 7001);
    assert!(sent_messages[0]["text"].as_str().unwrap().contains("Linked."));
  });
}

#[test]
fn telegram_config_does_not_register_webhook_with_telegram_api() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");

    let webhook_registrations = mock_state.webhook_registrations.lock().unwrap().clone();
    assert!(webhook_registrations.is_empty(), "polling-only config should not call setWebhook");
  });
}

#[test]
fn telegram_polling_reply_inherits_existing_reply_context() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    TelegramLinkRepository::upsert_link(context.employee_id.parse::<persistence::Uuid>().unwrap().into(), 9001, 7001)
      .await
      .unwrap();

    let existing = TelegramMessageCorrelationRepository::create(TelegramMessageCorrelationModel {
      telegram_chat_id:    7001,
      telegram_message_id: 88,
      direction:           TelegramMessageDirection::Outbound,
      kind:                TelegramCorrelationKind::Notification,
      issue_id:            None,
      run_id:              None,
      employee_id:         Some(context.employee_id.parse::<persistence::Uuid>().unwrap().into()),
      text_preview:        Some("Run completed".into()),
      created_at:          chrono::Utc::now(),
      updated_at:          chrono::Utc::now(),
    })
    .await
    .unwrap();

    crate::telegram::reset_polling_offset_for_tests();
    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 89,
        "text": "tell me more",
        "chat": { "id": 7001 },
        "from": { "id": 9001 },
        "reply_to_message": { "message_id": 88 }
      }),
    );

    crate::telegram::poll_once().await.unwrap();

    let inbound = TelegramMessageCorrelationRepository::find_by_chat_message(7001, 89).await.unwrap().unwrap();
    assert_eq!(inbound.direction, TelegramMessageDirection::Inbound);
    assert_eq!(inbound.kind, existing.kind);
    assert_eq!(inbound.employee_id.unwrap().uuid().to_string(), context.employee_id);
  });
}

#[test]
fn telegram_polling_supports_issue_fetch_comment_and_watch_flows() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": false
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    TelegramLinkRepository::upsert_link(context.employee_id.parse::<persistence::Uuid>().unwrap().into(), 9001, 7001)
      .await
      .unwrap();

    crate::telegram::reset_polling_offset_for_tests();

    let created = IssueRepository::create(IssueModel {
      title: "Telegram title".to_string(),
      description: "Telegram description".to_string(),
      creator: Some(context.employee_id.parse::<persistence::Uuid>().unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();
    let created_identifier = format!("{}-{}", created.identifier, created.issue_number);

    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 602,
        "text": format!("/issue {created_identifier}"),
        "chat": { "id": 7001 },
        "from": { "id": 9001 }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    let outbound_fetch = TelegramMessageCorrelationRepository::find_by_chat_message(7001, 1001).await.unwrap().unwrap();
    assert_eq!(outbound_fetch.kind, TelegramCorrelationKind::Issue);
    assert_eq!(outbound_fetch.issue_id.unwrap().uuid().to_string(), created.id.uuid().to_string());

    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 603,
        "text": format!("/comment {created_identifier} Telegram comment"),
        "chat": { "id": 7001 },
        "from": { "id": 9001 }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    let comments = IssueRepository::list_comments(created.id.clone()).await.unwrap();
    assert!(comments.iter().any(|comment| comment.comment == "Telegram comment"));

    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 604,
        "text": format!("/watch {created_identifier}"),
        "chat": { "id": 7001 },
        "from": { "id": 9001 }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    let watch = persistence::prelude::TelegramIssueWatchRepository::find(
      context.employee_id.parse::<persistence::Uuid>().unwrap().into(),
      created.id.clone(),
    )
    .await
    .unwrap();
    assert!(watch.is_some());

    let reply_target = TelegramMessageCorrelationRepository::find_by_chat_message(7001, 1001).await.unwrap().unwrap();
    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 605,
        "text": "Reply comment",
        "chat": { "id": 7001 },
        "from": { "id": 9001 },
        "reply_to_message": { "message_id": reply_target.telegram_message_id }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    let comments = IssueRepository::list_comments(created.id.clone()).await.unwrap();
    assert!(comments.iter().any(|comment| comment.comment == "Reply comment"));
  });
}

#[test]
fn telegram_polling_reports_delivery_failures_but_keeps_issue_workflow_side_effects() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "degraded-token",
            "bot_username": "blprnt_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let link_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/link-codes")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(serde_json::json!({ "employee_id": owner_id }).to_string()))
        .unwrap(),
      )
      .await
      .unwrap();
    let link_payload = response_json(link_response).await;
    let code = link_payload["code"].as_str().unwrap();

    crate::telegram::reset_polling_offset_for_tests();
    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 801,
        "text": format!("/link {code}"),
        "chat": { "id": 7801 },
        "from": { "id": 9801 }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    queue_telegram_update(
      &_mock_state,
      serde_json::json!({
        "message_id": 802,
        "text": "/issue new Delivery degraded title -- Delivery degraded description",
        "chat": { "id": 7801 },
        "from": { "id": 9801 }
      }),
    );
    crate::telegram::poll_once().await.unwrap();

    let issues = IssueRepository::list(ListIssuesParams::default()).await.unwrap();
    let created = issues.iter().find(|issue| issue.title == "Delivery degraded title");
    assert!(created.is_some(), "issue side effect should survive Telegram send failure");

    let links =
      TelegramLinkRepository::list_for_employee(owner_id.parse::<persistence::Uuid>().unwrap().into()).await.unwrap();
    assert_eq!(links.len(), 1, "link side effect should survive Telegram send failure");
  });
}

#[test]
#[ignore = "obsolete webhook-era regression; replace with polling-only equivalent before removing ignore"]
fn telegram_webhook_replies_to_unlinked_issue_commands_with_link_guidance() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": false
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/integrations/telegram/webhook")
          .header("content-type", "application/json")
          .header("x-telegram-bot-api-secret-token", "hook-secret")
          .body(Body::from(
            serde_json::json!({
              "message": {
                "message_id": 901,
                "text": "/issue ISSUE-1",
                "chat": { "id": 7901 },
                "from": { "id": 9901 }
              }
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["linked"], false);
    assert_eq!(payload["delivery_error"], Value::Null);

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0]["chat_id"], 7901);
    assert!(sent_messages[0]["text"].as_str().unwrap().contains("This chat is not linked yet"));

    let outbound = TelegramMessageCorrelationRepository::find_by_chat_message(7901, 1001).await.unwrap().unwrap();
    assert_eq!(outbound.direction, TelegramMessageDirection::Outbound);
    assert_eq!(outbound.kind, TelegramCorrelationKind::Unknown);
  });
}

#[test]
fn telegram_polling_processes_unlinked_issue_commands_with_link_guidance() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "webhook_secret": "ignored-for-polling",
            "bot_username": "blprnt_bot",
            "delivery_mode": "polling",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    crate::telegram::reset_polling_offset_for_tests();

    queue_telegram_update(
      &mock_state,
      serde_json::json!({
        "message_id": 991,
        "text": "/issue ISSUE-1",
        "chat": { "id": 7991 },
        "from": { "id": 9991 }
      }),
    );

    crate::telegram::poll_once().await.unwrap();

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0]["chat_id"], 7991);
    assert!(sent_messages[0]["text"].as_str().unwrap().contains("This chat is not linked yet"));

    let outbound = TelegramMessageCorrelationRepository::find_by_chat_message(7991, 1001).await.unwrap().unwrap();
    assert_eq!(outbound.direction, TelegramMessageDirection::Outbound);
    assert_eq!(outbound.kind, TelegramCorrelationKind::Unknown);

    crate::telegram::poll_once().await.unwrap();
    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert_eq!(sent_messages.len(), 1, "polling should not reprocess the same update after offset advances");
  });
}

#[test]
fn telegram_polling_supports_linked_issue_create_flow() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "webhook_secret": "ignored-for-polling",
            "bot_username": "blprnt_bot",
            "delivery_mode": "polling",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    TelegramLinkRepository::upsert_link(context.employee_id.parse::<persistence::Uuid>().unwrap().into(), 9001, 7001)
      .await
      .unwrap();

    crate::telegram::reset_polling_offset_for_tests();

    queue_telegram_update(
      &mock_state,
      serde_json::json!({
        "message_id": 992,
        "text": "/issue new Polled title -- Polled description",
        "chat": { "id": 7001 },
        "from": { "id": 9001 }
      }),
    );

    crate::telegram::poll_once().await.unwrap();

    let issues = IssueRepository::list(ListIssuesParams::default()).await.unwrap();
    let created = issues.iter().find(|issue| issue.title == "Polled title").unwrap();
    assert_eq!(created.description.as_str(), "Polled description");

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert_eq!(sent_messages.len(), 1);
    assert_eq!(sent_messages[0]["chat_id"], 7001);
    assert!(sent_messages[0]["text"].as_str().unwrap().contains("Created ISSUE-"));

    let outbound = TelegramMessageCorrelationRepository::find_by_chat_message(7001, 1001).await.unwrap().unwrap();
    assert_eq!(outbound.direction, TelegramMessageDirection::Outbound);
    assert_eq!(outbound.kind, TelegramCorrelationKind::Issue);
    assert_eq!(outbound.issue_id.unwrap().uuid().to_string(), created.id.uuid().to_string());
  });
}

#[test]
fn telegram_polling_skips_updates_when_config_is_webhook_mode() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": false
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let saved_config = TelegramConfigRepository::get_latest().await.unwrap().unwrap();
    assert!(!saved_config.enabled, "test setup expects disabled telegram config");

    crate::telegram::reset_polling_offset_for_tests();

    queue_telegram_update(
      &mock_state,
      serde_json::json!({
        "message_id": 993,
        "text": "/issue ISSUE-1",
        "chat": { "id": 7993 },
        "from": { "id": 9993 }
      }),
    );

    crate::telegram::poll_once().await.unwrap();

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert!(sent_messages.is_empty(), "polling worker should stay idle when telegram config is disabled");
    assert!(TelegramMessageCorrelationRepository::find_by_chat_message(7993, 993).await.unwrap().is_none());
  });
}

#[test]
fn telegram_polling_treats_get_updates_conflict_as_noop() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "conflict-token",
            "webhook_secret": "ignored-for-polling",
            "bot_username": "blprnt_bot",
            "delivery_mode": "polling",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    crate::telegram::reset_polling_offset_for_tests();

    crate::telegram::poll_once().await.unwrap();

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert!(sent_messages.is_empty(), "conflicted polling pass should not send messages or fail");
  });
}

#[test]
fn telegram_config_rejects_blank_bot_token_on_initial_create() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "   ",
            "bot_username": "blprnt_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "unexpected response {status}: {payload}");
  });
}

#[test]
fn telegram_config_blank_bot_token_preserves_existing_secret_for_polling() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let first_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "",
            "bot_username": "renamed_bot",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(second_response.status(), StatusCode::OK);

    crate::telegram::reset_polling_offset_for_tests();
    crate::telegram::poll_once().await.unwrap();

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert!(
      sent_messages.is_empty(),
      "polling should succeed against the preserved token and stay idle with no updates"
    );
  });
}

#[test]
#[ignore = "obsolete webhook-era regression; replace with polling-only equivalent before removing ignore"]
fn telegram_webhook_supports_run_start_continue_and_notifications() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "webhook_secret": "hook-secret",
            "bot_username": "blprnt_bot",
            "webhook_url": "https://example.com/telegram/webhook",
            "delivery_mode": "webhook",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    TelegramLinkRepository::upsert_link(owner_id.parse::<persistence::Uuid>().unwrap().into(), 9002, 7002)
      .await
      .unwrap();

    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/integrations/telegram/webhook")
          .header("content-type", "application/json")
          .header("x-telegram-bot-api-secret-token", "hook-secret")
          .body(Body::from(
            serde_json::json!({
              "message": {
                "message_id": 701,
                "text": "/run Investigate Telegram run workflow",
                "chat": { "id": 7002 },
                "from": { "id": 9002 }
              }
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    let payload = response_json(response).await;
    assert_eq!(payload["ok"], true, "unexpected run start webhook payload: {payload}");

    let runs = RunRepository::list(RunFilter {
      employee: Some(owner_id.parse::<persistence::Uuid>().unwrap().into()),
      issue:    None,
      status:   None,
      trigger:  Some(RunTrigger::Conversation),
    })
    .await
    .unwrap();
    assert_eq!(runs.len(), 1);
    let run = runs[0].clone();
    let run = RunRepository::update(run.id.clone(), persistence::prelude::RunStatus::Completed).await.unwrap();
    assert_eq!(run.turns.len(), 1);
    match &run.turns[0].steps[0].request.contents[0] {
      persistence::prelude::TurnStepContent::Text(text) => {
        assert_eq!(text.text, "Investigate Telegram run workflow");
      }
      other => panic!("unexpected first run request content: {other:?}"),
    }

    let run_message = TelegramMessageCorrelationRepository::find_by_chat_message(7002, 1001).await.unwrap().unwrap();
    assert_eq!(run_message.kind, TelegramCorrelationKind::Run);
    assert_eq!(run_message.run_id.unwrap().uuid().to_string(), run.id.uuid().to_string());

    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/v1/integrations/telegram/webhook")
          .header("content-type", "application/json")
          .header("x-telegram-bot-api-secret-token", "hook-secret")
          .body(Body::from(
            serde_json::json!({
              "message": {
                "message_id": 702,
                "text": "Add notification coverage too",
                "chat": { "id": 7002 },
                "from": { "id": 9002 },
                "reply_to_message": { "message_id": 1001 }
              }
            })
            .to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    let payload = response_json(response).await;
    assert_eq!(payload["ok"], true, "unexpected run continue webhook payload: {payload}");

    let continued = RunRepository::get(run.id.clone()).await.unwrap();
    assert!(matches!(continued.status, persistence::prelude::RunStatus::Pending));
    assert_eq!(continued.turns.len(), 2);
    match &continued.turns[1].steps[0].request.contents[0] {
      persistence::prelude::TurnStepContent::Text(text) => {
        assert_eq!(text.text, "Add notification coverage too");
      }
      other => panic!("unexpected continued run request content: {other:?}"),
    }

    let continue_message =
      TelegramMessageCorrelationRepository::find_by_chat_message(7002, 1002).await.unwrap().unwrap();
    assert_eq!(continue_message.kind, TelegramCorrelationKind::Run);
    assert_eq!(continue_message.run_id.unwrap().uuid().to_string(), run.id.uuid().to_string());

    let watcher_issue = IssueRepository::create(IssueModel {
      title: "Watched issue".to_string(),
      description: "desc".to_string(),
      creator: Some(context.employee_id.parse::<persistence::Uuid>().unwrap().into()),
      ..Default::default()
    })
    .await
    .unwrap();
    TelegramLinkRepository::upsert_link(context.employee_id.parse::<persistence::Uuid>().unwrap().into(), 9001, 7001)
      .await
      .unwrap();
    TelegramIssueWatchRepository::watch(
      context.employee_id.parse::<persistence::Uuid>().unwrap().into(),
      watcher_issue.id.clone(),
    )
    .await
    .unwrap();

    let watched_run = RunRepository::create(RunModel::new(
      context.employee_id.parse::<persistence::Uuid>().unwrap().into(),
      RunTrigger::IssueAssignment { issue_id: watcher_issue.id.clone() },
    ))
    .await
    .unwrap();
    TurnRepository::create(TurnModel { run_id: watched_run.id.clone(), ..Default::default() }).await.unwrap();
    let watched_run = RunRepository::update(watched_run.id, persistence::prelude::RunStatus::Completed).await.unwrap();

    crate::telegram::notify_run_terminal_status(watched_run.id.clone()).await.unwrap();

    let notification = TelegramMessageCorrelationRepository::list_outbound_for_run(watched_run.id.clone())
      .await
      .unwrap()
      .into_iter()
      .find(|record| record.telegram_chat_id == 7001 && record.kind == TelegramCorrelationKind::Notification)
      .unwrap();
    assert_eq!(notification.kind, TelegramCorrelationKind::Notification);
    assert_eq!(notification.run_id.unwrap().uuid().to_string(), watched_run.id.uuid().to_string());
    assert_eq!(notification.issue_id.unwrap().uuid().to_string(), watcher_issue.id.uuid().to_string());

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert!(sent_messages.iter().any(|message| message["parse_mode"] == Value::Null));
  });
}

#[test]
fn telegram_config_accepts_markdown_parse_mode_alias() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, _mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "parse_mode": "markdown",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();

    let status = response.status();
    let payload = response_json(response).await;
    assert_eq!(status, StatusCode::OK, "unexpected response {status}: {payload}");
    assert_eq!(payload["parse_mode"], "markdown_v2");
  });
}

#[test]
fn telegram_send_message_includes_configured_parse_mode() {
  let _lock = env_lock();
  TEST_RUNTIME.block_on(async {
    let _context = setup_context().await;
    let owner_id = create_owner().await;
    let (mock_base_url, mock_state, _server) = telegram_mock_server().await;
    let _telegram_base_url = EnvVarGuard::set("BLPRNT_TELEGRAM_API_BASE_URL", &mock_base_url);
    let app = test_app();

    let response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("POST")
            .uri("/api/v1/integrations/telegram/config")
            .header("content-type", "application/json"),
          &owner_id,
        )
        .body(Body::from(
          serde_json::json!({
            "bot_token": "bot-token",
            "bot_username": "blprnt_bot",
            "parse_mode": "html",
            "enabled": true
          })
          .to_string(),
        ))
        .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    crate::telegram::send_message(
      7002,
      "Verify parse mode delivery",
      None,
      TelegramCorrelationKind::Unknown,
      None,
      None,
      None,
    )
    .await
    .unwrap();

    let sent_messages = mock_state.sent_messages.lock().unwrap().clone();
    assert!(sent_messages.iter().any(|message| message["parse_mode"] == "html"));
  });
}
