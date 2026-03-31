use std::sync::Arc;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::SurrealConnection;
use qmd::Storage as _;
use tempfile::TempDir;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[tokio::test]
async fn employee_memory_service_indexes_employee_memory_directory() {
  let _guard = ENV_LOCK.lock().unwrap();
  let old_home = std::env::var("HOME").ok();
  let old_memory_base_dir = std::env::var_os("BLPRNT_MEMORY_BASE_DIR");

  let home = TempDir::new().unwrap();
  unsafe { std::env::set_var("HOME", home.path().to_string_lossy().to_string()) };
  unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", home.path()) };

  let employee: EmployeeId = persistence::Uuid::new_v4().into();
  let service = memory::EmployeeMemoryService::new(employee.clone()).await.unwrap();
  let employee_root = home.path().join(".blprnt").join("employees").join(employee.uuid().to_string());
  std::fs::create_dir_all(employee_root.join("memory")).unwrap();
  std::fs::create_dir_all(employee_root.join("life").join("projects").join("runtime")).unwrap();
  std::fs::write(employee_root.join("memory").join("2026-03-30.md"), "daily memory").unwrap();
  std::fs::write(employee_root.join("life").join("projects").join("runtime").join("summary.md"), "runtime summary")
    .unwrap();
  service.search("runtime", Some(5)).await.unwrap();

  let db = SurrealConnection::db().await;
  let storage = Arc::new(qmd::SurrealStorage::new(db));
  let collections = storage.list_collections_info().await.unwrap();
  let memory_collection = collections
    .iter()
    .find(|collection| collection.name == memory::employee_memory_collection_name(&employee))
    .unwrap();
  let life_collection =
    collections.iter().find(|collection| collection.name == memory::employee_life_collection_name(&employee)).unwrap();
  assert_eq!(memory_collection.pwd, employee_root.join("memory").to_string_lossy());
  assert_eq!(life_collection.pwd, employee_root.join("life").to_string_lossy());
  assert!(collections.iter().all(|collection| collection.pwd != employee_root.to_string_lossy()));

  let store = qmd::create_store(qmd::StoreOptions { storage, llm: None, config: None }).await.unwrap();

  let rel = qmd::handelize("2026-03-30.md").unwrap();
  let vp = qmd::build_virtual_path(&memory::employee_memory_collection_name(&employee), &rel);
  let doc = store.get(&vp, Some(&qmd::GetOptions { include_body: Some(true) })).await.unwrap().unwrap();
  assert!(doc.body.unwrap_or_default().contains("daily memory"));

  match old_home {
    Some(v) => unsafe { std::env::set_var("HOME", v) },
    None => unsafe { std::env::remove_var("HOME") },
  }
  match old_memory_base_dir {
    Some(v) => unsafe { std::env::set_var("BLPRNT_MEMORY_BASE_DIR", v) },
    None => unsafe { std::env::remove_var("BLPRNT_MEMORY_BASE_DIR") },
  }
}
