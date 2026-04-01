use std::path::Path;
use std::sync::Arc;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::SurrealConnection;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

const QMD_COLLECTION_PREFIX: &str = "memories";

pub fn employee_collection_name(employee_id: &EmployeeId) -> String {
  scoped_collection_name("employees", &employee_id.uuid().to_string())
}

pub fn employee_memory_collection_name(employee_id: &EmployeeId) -> String {
  format!("{}-memory", employee_collection_name(employee_id))
}

pub fn employee_life_collection_name(employee_id: &EmployeeId) -> String {
  format!("{}-life", employee_collection_name(employee_id))
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
