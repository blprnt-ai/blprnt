use std::sync::LazyLock;
use std::sync::Mutex;

use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use chrono::Local;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::ProjectModel;
use persistence::prelude::ProjectRepository;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

static ENV_LOCK: Mutex<()> = Mutex::new(());
static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
  tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("failed to create test runtime")
});

struct HomeGuard {
  previous_home: Option<String>,
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

struct TestContext {
  _home:       TempDir,
  _guard:      HomeGuard,
  _cwd_guard:  CwdGuard,
  employee_id: String,
  project_id:  String,
}

async fn setup_context() -> TestContext {
  let home = TempDir::new().unwrap();
  let guard = HomeGuard::set(&home);
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
    _cwd_guard:  cwd_guard,
    employee_id: employee.id.uuid().to_string(),
    project_id:  project.id.uuid().to_string(),
  }
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

#[test]
#[ignore]
fn memory_routes_support_project_memory_lifecycle() {
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

    let delete_response = app
      .clone()
      .oneshot(
        request_with_employee(
          Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/projects/{}/memory/file?path={}", context.project_id, path)),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let empty_list_response = app
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

    assert_eq!(empty_list_response.status(), StatusCode::OK);
    let empty_list = response_json(empty_list_response).await;
    assert_eq!(empty_list["nodes"], serde_json::json!([]));
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
#[ignore]
fn memory_routes_support_employee_memory_lifecycle() {
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

    let delete_response = app
      .oneshot(
        request_with_employee(
          Request::builder().method("DELETE").uri(format!("/api/v1/employees/me/memory/file?path={path}")),
          &context.employee_id,
        )
        .body(Body::empty())
        .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
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
