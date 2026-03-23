use std::fs;
use std::path::Path;
use std::sync::Arc;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::SurrealConnection;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

use crate::service::employee_memory_root;
use crate::service::project_memory_root;

/**
 * dir structure:
 *
 * memories/
 *  /employees/<employee_id>/
 *    .learnings/
 *      ERRORS.md
 *    life/
 *      projects/<name>/
 *        summary.md
 *        items.yaml
 *      archives/
 *        summary.md
 *        items.yaml
 *      resources/<topic>/
 *        summary.md
 *        items.yaml
 *      areas/<name>/
 *        summary.md
 *        items.yaml
 *    memory/<date>.md
 *    AGENTS.md
 *    HEARTBEAT.md
 *    MEMORY.md
 *    SOUL.md
 *    TOOLS.md
 *  /projects/
 *    <project_id>/
 *      SUMMARY.md
 *      archives/
 *        summary.md
 *        items.yaml
 *      resources/<topic>/
 *        summary.md
 *        items.yaml
 *
 */

const QMD_COLLECTION_PREFIX: &str = "memories";

pub async fn init_new_project(project_id: &ProjectId) -> MemoryResult<()> {
  let project_root = project_memory_root(project_id)?;
  let collection_name = project_collection_name(project_id);

  fs::create_dir_all(&project_root)?;
  ensure_collection(&collection_name, &project_root).await?;
  sync_collection(&collection_name).await?;

  Ok(())
}

pub async fn init_new_employee(
  employee_id: &EmployeeId,
  agents: &str,
  heartbeat: &str,
  soul: &str,
  tools: &str,
) -> MemoryResult<()> {
  let employee_root = employee_memory_root(employee_id)?;
  let collection_name = employee_collection_name(employee_id);

  fs::create_dir_all(&employee_root)?;
  fs::write(employee_root.join("AGENTS.md"), agents)?;
  fs::write(employee_root.join("HEARTBEAT.md"), heartbeat)?;
  fs::write(employee_root.join("SOUL.md"), soul)?;
  fs::write(employee_root.join("TOOLS.md"), tools)?;

  ensure_collection(&collection_name, &employee_root).await?;
  sync_collection(&collection_name).await?;

  Ok(())
}

pub fn employee_collection_name(employee_id: &EmployeeId) -> String {
  scoped_collection_name("employees", &employee_id.uuid().to_string())
}

pub fn project_collection_name(project_id: &ProjectId) -> String {
  scoped_collection_name("projects", &project_id.uuid().to_string())
}

pub async fn qmd_store() -> MemoryResult<qmd::QmdStore> {
  let db = SurrealConnection::db().await;
  qmd::SurrealStorage::migrate(&db).await.map_err(|_| MemoryError::QmdCollectionInitializationFailed)?;

  let storage = qmd::SurrealStorage::new(db);
  qmd::create_store(qmd::StoreOptions { storage: Arc::new(storage), llm: None, config: None })
    .await
    .map_err(|error| MemoryError::QmdOperationFailed(error.to_string()))
}

pub async fn ensure_collection(collection_name: &str, collection_path: &Path) -> MemoryResult<()> {
  let store = qmd_store().await?;
  store
    .add_collection(
      collection_name,
      &qmd::AddCollectionOptions {
        path:    collection_path.to_string_lossy().to_string(),
        pattern: None,
        ignore:  None,
      },
    )
    .await
    .map_err(|error| MemoryError::QmdOperationFailed(error.to_string()))?;

  Ok(())
}

pub async fn sync_collection(collection_name: &str) -> MemoryResult<qmd::UpdateResult> {
  let store = qmd_store().await?;
  store
    .update(Some(&qmd::UpdateOptions { collections: Some(vec![collection_name.to_string()]), on_progress: None }))
    .await
    .map_err(|error| MemoryError::QmdOperationFailed(error.to_string()))
}

fn scoped_collection_name(scope_directory: &str, id: &str) -> String {
  format!("{QMD_COLLECTION_PREFIX}-{scope_directory}-{id}")
}

pub async fn search_collection(
  collection_name: &str,
  query: &str,
  limit: Option<usize>,
) -> MemoryResult<Vec<qmd::HybridQueryResult>> {
  let store = qmd_store().await?;
  store
    .search(&qmd::SearchOptions {
      query: Some(query.to_string()),
      collection: Some(collection_name.to_string()),
      limit,
      ..Default::default()
    })
    .await
    .map_err(|error| MemoryError::QmdOperationFailed(error.to_string()))
}
