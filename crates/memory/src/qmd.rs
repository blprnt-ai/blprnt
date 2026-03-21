use std::path::Path;
use std::sync::Arc;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::SurrealConnection;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

use crate::store;

/**
 * dir structure:
 *
 * memories/
 *  <employee_id>/
 *    .learnings/
 *      ERRORS.md
 *    life/
 *      archives/
 *      people/<name>/
 *        summary.md
 *        items.yaml
 *      projects/<name>/
 *        summary.md
 *        items.yaml
 *      resources/<topic>/
 *        summary.md
 *        items.yaml
 *    memory/<date>.md
 *    AGENTS.md
 *    HEARTBEAT.md
 *    MEMORY.md
 *    SOUL.md
 *    TOOLS.md
 *
 */

pub async fn init_new_employee(
  employee_id: &EmployeeId,
  agents: &str,
  heartbeat: &str,
  soul: &str,
  tools: &str,
) -> MemoryResult<()> {
  let employee_id = employee_id.uuid().to_string();
  let employee_dir = Path::new(&employee_id);

  store::ensure_dir(employee_dir)?;

  store::write(&employee_dir.join("AGENTS.md"), agents)?;
  store::write(&employee_dir.join("HEARTBEAT.md"), heartbeat)?;
  store::write(&employee_dir.join("SOUL.md"), soul)?;
  store::write(&employee_dir.join("TOOLS.md"), tools)?;

  init_collection(&employee_id).await?;

  Ok(())
}

async fn init_collection(employee_id: &str) -> MemoryResult<()> {
  let collection_path = store::memories_root().join(employee_id);

  let db = SurrealConnection::db().await;
  qmd::SurrealStorage::migrate(&db).await.map_err(|_| MemoryError::QmdCollectionInitializationFailed)?;

  let storage = qmd::SurrealStorage::new(db);
  let store = qmd::create_store(qmd::StoreOptions { storage: Arc::new(storage), llm: None, config: None })
    .await
    .map_err(|_| MemoryError::QmdCollectionInitializationFailed)?;

  store
    .add_collection(
      employee_id,
      &qmd::AddCollectionOptions {
        path:    collection_path.to_string_lossy().to_string(),
        pattern: None,
        ignore:  None,
      },
    )
    .await
    .map_err(|_| MemoryError::QmdCollectionInitializationFailed)?;

  let _ = store.update(None).await.map_err(|_| MemoryError::QmdCollectionInitializationFailed)?;

  Ok(())
}
