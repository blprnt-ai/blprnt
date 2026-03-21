use std::sync::Arc;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::SurrealConnection;
use qmd::Storage as _;
use tempfile::TempDir;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[tokio::test]
async fn init_new_employee_creates_qmd_collection_and_indexes_files() {
  let _guard = ENV_LOCK.lock().unwrap();
  let old_home = std::env::var("HOME").ok();

  let home = TempDir::new().unwrap();
  unsafe { std::env::set_var("HOME", home.path().to_string_lossy().to_string()) };

  let employee: EmployeeId = persistence::Uuid::new_v4().into();

  memory::init_new_employee(&employee, "agents", "heartbeat", "soul", "tools").await.unwrap();

  let employee_id = employee.uuid().to_string();

  let db = SurrealConnection::db().await;
  let storage = Arc::new(qmd::SurrealStorage::new(db));
  let collections = storage.list_collections().await.unwrap();
  assert!(collections.iter().any(|c| c.name == employee_id));

  let store = qmd::create_store(qmd::StoreOptions { storage, llm: None, config: None }).await.unwrap();

  let rel = qmd::handelize("AGENTS.md").unwrap();
  let vp = qmd::build_virtual_path(&employee_id, &rel);
  let doc = store.get(&vp, Some(&qmd::GetOptions { include_body: Some(true) })).await.unwrap().unwrap();
  assert!(doc.body.unwrap_or_default().contains("agents"));

  match old_home {
    Some(v) => unsafe { std::env::set_var("HOME", v) },
    None => unsafe { std::env::remove_var("HOME") },
  }
}
