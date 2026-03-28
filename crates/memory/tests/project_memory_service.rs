use std::sync::Arc;
use std::sync::LazyLock;

use chrono::Local;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectModel;
use persistence::prelude::ProjectRepository;
use persistence::prelude::SurrealConnection;
use qmd::Storage as _;
use tempfile::TempDir;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
  tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("failed to create test runtime")
});

async fn create_project(name: &str) -> ProjectId {
  ProjectRepository::create(ProjectModel::new(name.to_string(), vec![])).await.unwrap().id
}

fn with_temp_home(home: &TempDir) -> impl Drop {
  struct Guard(Option<String>);

  impl Drop for Guard {
    fn drop(&mut self) {
      match self.0.take() {
        Some(value) => unsafe { std::env::set_var("HOME", value) },
        None => unsafe { std::env::remove_var("HOME") },
      }
    }
  }

  let previous = std::env::var("HOME").ok();
  unsafe { std::env::set_var("HOME", home.path().to_string_lossy().to_string()) };
  Guard(previous)
}

fn with_temp_cwd(path: &std::path::Path) -> impl Drop {
  struct Guard(std::path::PathBuf);

  impl Drop for Guard {
    fn drop(&mut self) {
      std::env::set_current_dir(&self.0).unwrap();
    }
  }

  let previous = std::env::current_dir().unwrap();
  std::env::set_current_dir(path).unwrap();
  Guard(previous)
}

#[test]
fn project_memory_service_rejects_parent_traversal_paths() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let project_id = create_project("runtime-memory-paths").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();

    let error = service.read("../escape.md").await.unwrap_err();

    assert!(error.to_string().contains("invalid path"));
  });
}

#[test]
fn project_memory_service_builds_sorted_markdown_tree() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let project_id = create_project("runtime-memory-tree").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();

    service.update("archives/summary.md", "# Archives").await.unwrap();
    service.update("resources/zeta/summary.md", "# Zeta").await.unwrap();
    service.update("resources/alpha/summary.md", "# Alpha").await.unwrap();
    service.update("SUMMARY.md", "# Root").await.unwrap();

    let listed = service.list().await.unwrap();

    assert_eq!(listed.root_path, "$PROJECT_HOME");

    assert_eq!(
      listed.nodes,
      vec![
        memory::MemoryTreeNode::Directory {
          name:     "resources".to_string(),
          path:     "resources".to_string(),
          children: vec![
            memory::MemoryTreeNode::Directory {
              name:     "zeta".to_string(),
              path:     "resources/zeta".to_string(),
              children: vec![memory::MemoryTreeNode::File {
                name: "summary.md".to_string(),
                path: "resources/zeta/summary.md".to_string(),
              }],
            },
            memory::MemoryTreeNode::Directory {
              name:     "alpha".to_string(),
              path:     "resources/alpha".to_string(),
              children: vec![memory::MemoryTreeNode::File {
                name: "summary.md".to_string(),
                path: "resources/alpha/summary.md".to_string(),
              }],
            },
          ],
        },
        memory::MemoryTreeNode::Directory {
          name:     "archives".to_string(),
          path:     "archives".to_string(),
          children: vec![memory::MemoryTreeNode::File {
            name: "summary.md".to_string(),
            path: "archives/summary.md".to_string(),
          }],
        },
        memory::MemoryTreeNode::File { name: "SUMMARY.md".to_string(), path: "SUMMARY.md".to_string() },
      ]
    );
  });
}

#[test]
fn project_memory_service_bootstraps_qmd_and_keeps_search_in_sync() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let project_id = create_project("runtime-memory-qmd").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();

    let created = service.create("# Rust memory\n\nRust keeps the runtime correct.").await.unwrap();
    let search = service.search("runtime", Some(5)).await.unwrap();

    assert_eq!(created.status, memory::MemoryWriteStatus::Written);
    assert_eq!(created.path, "SUMMARY.md");
    assert_eq!(search.memories.len(), 1);
    assert!(search.memories[0].content.contains("runtime correct"));

    let db = SurrealConnection::db().await;
    let storage = Arc::new(qmd::SurrealStorage::new(db));
    let collections = storage.list_collections().await.unwrap();
    assert!(collections.iter().any(|collection| collection.name == memory::project_collection_name(&project_id)));

    service.update(&created.path, "# Updated memory\n\nStreaming output is safer to debug.").await.unwrap();

    let stale = service.search("runtime", Some(5)).await.unwrap();
    let fresh = service.search("streaming", Some(5)).await.unwrap();
    assert!(stale.memories.is_empty());
    assert_eq!(fresh.memories.len(), 1);
    assert!(fresh.memories[0].content.contains("safer to debug"));

    service.delete(&created.path).await.unwrap();

    let deleted = service.search("streaming", Some(5)).await.unwrap();
    assert!(deleted.memories.is_empty());
  });
}

#[test]
fn project_memory_service_create_at_writes_scope_relative_markdown_path() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let project_id = create_project("runtime-memory-explicit-project-path").await;
    let service = memory::ProjectMemoryService::new(project_id).await.unwrap();

    let created = service.create_at("resources/runtime/summary.md", "# Runtime").await.unwrap();
    let read = service.read(&created.path).await.unwrap();

    assert_eq!(created.status, memory::MemoryWriteStatus::Written);
    assert_eq!(created.path, "resources/runtime/summary.md");
    assert_eq!(read.content, "# Runtime");
  });
}

#[test]
fn employee_memory_service_uses_employee_scope_root() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id.clone()).await.unwrap();

    service.update("memory/2026-03-23.md", "# Runtime").await.unwrap();

    let listed = service.list().await.unwrap();
    assert_eq!(listed.root_path, "$AGENT_HOME");
    assert_eq!(
      listed.nodes,
      vec![memory::MemoryTreeNode::Directory {
        name:     "memory".to_string(),
        path:     "memory".to_string(),
        children: vec![memory::MemoryTreeNode::File {
          name: "2026-03-23.md".to_string(),
          path: "memory/2026-03-23.md".to_string(),
        }],
      }]
    );
  });
}

#[test]
fn employee_memory_service_create_at_writes_scope_relative_markdown_path() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id).await.unwrap();

    let created = service.create_at(".learnings/ERRORS.md", "# Errors").await.unwrap();
    let read = service.read(&created.path).await.unwrap();

    assert_eq!(created.status, memory::MemoryWriteStatus::Written);
    assert_eq!(created.path, ".learnings/ERRORS.md");
    assert_eq!(read.content, "# Errors");
  });
}

#[test]
fn employee_memory_service_create_uses_memory_daily_note_path() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_temp_cwd(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id).await.unwrap();

    let created = service.create("# Runtime").await.unwrap();
    let expected_date = Local::now().date_naive().format("%Y-%m-%d").to_string();

    assert_eq!(created.status, memory::MemoryWriteStatus::Written);
    assert_eq!(created.date, expected_date);
    assert_eq!(created.path, format!("memory/{expected_date}.md"));
  });
}
