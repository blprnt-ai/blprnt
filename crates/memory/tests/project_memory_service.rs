use std::sync::Arc;
use std::sync::LazyLock;

use persistence::prelude::DbId;
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

fn with_memory_base_dir(path: &std::path::Path) -> impl Drop {
  struct Guard(Option<std::ffi::OsString>);

  impl Drop for Guard {
    fn drop(&mut self) {
      match self.0.take() {
        Some(value) => unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", value) },
        None => unsafe { std::env::remove_var("BLPRNT_MEMORY_BASE_DIR") },
      }
    }
  }

  let previous = std::env::var_os("BLPRNT_MEMORY_BASE_DIR");
  unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", path) };
  Guard(previous)
}

#[test]
fn project_memory_service_rejects_parent_traversal_paths() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_memory_base_dir(home.path());
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
    let _cwd_guard = with_memory_base_dir(home.path());
    let project_id = create_project("runtime-memory-tree").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("projects").join(project_id.uuid().to_string()).join("memory");

    std::fs::create_dir_all(root.join("archives")).unwrap();
    std::fs::create_dir_all(root.join("resources/zeta")).unwrap();
    std::fs::create_dir_all(root.join("resources/alpha")).unwrap();
    std::fs::write(root.join("archives/summary.md"), "# Archives").unwrap();
    std::fs::write(root.join("resources/zeta/summary.md"), "# Zeta").unwrap();
    std::fs::write(root.join("resources/alpha/summary.md"), "# Alpha").unwrap();
    std::fs::write(root.join("SUMMARY.md"), "# Root").unwrap();

    let listed = service.list().await.unwrap();

    assert_eq!(listed.root_path, "$PROJECT_HOME/memory");

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
    let _cwd_guard = with_memory_base_dir(home.path());
    let project_id = create_project("runtime-memory-qmd").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("projects").join(project_id.uuid().to_string()).join("memory");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("SUMMARY.md"), "# Rust memory\n\nRust keeps the runtime correct.").unwrap();
    let search = service.search("runtime", Some(5)).await.unwrap();

    assert_eq!(search.memories.len(), 1);
    assert!(search.memories[0].content.contains("runtime correct"));

    let db = SurrealConnection::db().await;
    let storage = Arc::new(qmd::SurrealStorage::new(db));
    let collections = storage.list_collections_info().await.unwrap();
    let collection =
      collections.iter().find(|collection| collection.name == memory::project_collection_name(&project_id)).unwrap();
    assert_eq!(
      collection.pwd,
      home.path().join(".blprnt").join("projects").join(project_id.uuid().to_string()).join("memory").to_string_lossy()
    );

    std::fs::write(root.join("SUMMARY.md"), "# Updated memory\n\nStreaming output is safer to debug.").unwrap();

    let stale = service.search("runtime", Some(5)).await.unwrap();
    let fresh = service.search("streaming", Some(5)).await.unwrap();
    assert!(stale.memories.is_empty());
    assert_eq!(fresh.memories.len(), 1);
    assert!(fresh.memories[0].content.contains("safer to debug"));

    std::fs::remove_file(root.join("SUMMARY.md")).unwrap();

    let deleted = service.search("streaming", Some(5)).await.unwrap();
    assert!(deleted.memories.is_empty());
  });
}

#[test]
fn project_memory_service_reads_existing_scope_relative_markdown_path() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_memory_base_dir(home.path());
    let project_id = create_project("runtime-memory-explicit-project-path").await;
    let service = memory::ProjectMemoryService::new(project_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("projects").join(project_id.uuid().to_string()).join("memory");
    std::fs::create_dir_all(root.join("resources/runtime")).unwrap();
    std::fs::write(root.join("resources/runtime/summary.md"), "# Runtime").unwrap();

    let read = service.read("resources/runtime/summary.md").await.unwrap();

    assert_eq!(read.path, "resources/runtime/summary.md");
    assert_eq!(read.content, "# Runtime");
  });
}

#[test]
fn employee_memory_service_uses_employee_scope_root() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_memory_base_dir(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("employees").join(employee_id.uuid().to_string()).join("memory");
    std::fs::create_dir_all(&root).unwrap();

    std::fs::write(root.join("2026-03-23.md"), "# Runtime").unwrap();

    let listed = service.list().await.unwrap();
    assert_eq!(listed.root_path, "$AGENT_HOME/memory");
    assert_eq!(
      listed.nodes,
      vec![memory::MemoryTreeNode::File { name: "2026-03-23.md".to_string(), path: "2026-03-23.md".to_string() }]
    );

    let expected_root = home.path().join(".blprnt").join("employees").join(employee_id.uuid().to_string());
    assert!(expected_root.join("memory").join("2026-03-23.md").is_file());
  });
}

#[test]
fn employee_memory_service_reads_existing_scope_relative_markdown_path() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_memory_base_dir(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("employees").join(employee_id.uuid().to_string()).join("memory");
    std::fs::create_dir_all(root.join("areas/runtime")).unwrap();
    std::fs::write(root.join("areas/runtime/summary.md"), "# Runtime").unwrap();

    let read = service.read("areas/runtime/summary.md").await.unwrap();

    assert_eq!(read.path, "areas/runtime/summary.md");
    assert_eq!(read.content, "# Runtime");
  });
}

#[test]
fn employee_memory_service_searches_existing_daily_notes() {
  let _guard = ENV_LOCK.lock().unwrap();
  TEST_RUNTIME.block_on(async {
    let home = TempDir::new().unwrap();
    let _home_guard = with_temp_home(&home);
    let _cwd_guard = with_memory_base_dir(home.path());
    let employee_id: persistence::prelude::EmployeeId = persistence::Uuid::new_v4().into();
    let service = memory::EmployeeMemoryService::new(employee_id.clone()).await.unwrap();
    let root = home.path().join(".blprnt").join("employees").join(employee_id.uuid().to_string()).join("memory");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("2026-03-31.md"), "# Runtime\n\nDaily note memory").unwrap();

    let search = service.search("daily note", Some(5)).await.unwrap();

    assert_eq!(search.memories.len(), 1);
    assert!(search.memories[0].content.contains("Daily note memory"));
  });
}
