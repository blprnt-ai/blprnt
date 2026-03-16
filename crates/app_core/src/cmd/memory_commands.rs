use std::fs;
use std::path::Component;
use std::path::PathBuf;

use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use common::memory::ManagedMemoryStore;
use common::memory::MemorySearchRequest;
use common::memory::MemorySearchResult;
use common::memory::MemoryWriteResult;
use common::memory::local_today;
use common::paths::BlprntPath;
use persistence::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryCreateRequest {
  pub project_id: String,
  pub content:    String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryReadRequest {
  pub project_id: String,
  pub path:       String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryReadResult {
  pub path:    String,
  pub content: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemorySearchCommandRequest {
  pub project_id: String,
  pub query:      String,
  pub limit:      Option<usize>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryUpdateRequest {
  pub project_id: String,
  pub path:       String,
  pub content:    String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryDeleteRequest {
  pub project_id: String,
  pub path:       String,
}

#[tauri::command]
#[specta::specta]
pub async fn memory_create(request: MemoryCreateRequest) -> TauriResult<MemoryWriteResult> {
  let memory_root = project_memory_root(&request.project_id)?;
  let store = ManagedMemoryStore::new(memory_root);

  store.append_entry_for_date(local_today(), &request.content).map_err(anyhow::Error::from).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn memory_read(request: MemoryReadRequest) -> TauriResult<MemoryReadResult> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  let content = fs::read_to_string(&path).map_err(anyhow::Error::from).into_tauri()?;

  Ok(MemoryReadResult { path: request.path, content })
}

#[tauri::command]
#[specta::specta]
pub async fn memory_search(request: MemorySearchCommandRequest) -> TauriResult<MemorySearchResult> {
  let project_id = SurrealId::try_from(request.project_id).map_err(anyhow::Error::from).into_tauri()?;

  common::memory::QmdMemorySearchService::new(project_id.key().to_string())
    .search(&MemorySearchRequest { query: request.query, limit: request.limit }, None)
    .await
    .map_err(anyhow::Error::from)
    .into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn memory_update(request: MemoryUpdateRequest) -> TauriResult<MemoryReadResult> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(anyhow::Error::from).into_tauri()?;
  }
  fs::write(&path, &request.content).map_err(anyhow::Error::from).into_tauri()?;

  Ok(MemoryReadResult { path: request.path, content: request.content })
}

#[tauri::command]
#[specta::specta]
pub async fn memory_delete(request: MemoryDeleteRequest) -> TauriResult<()> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  fs::remove_file(path).map_err(anyhow::Error::from).into_tauri()
}

fn project_memory_root(project_id: &str) -> TauriResult<PathBuf> {
  let project_id = SurrealId::try_from(project_id).map_err(anyhow::Error::from).into_tauri()?;
  Ok(BlprntPath::memories_root().join(project_id.key().to_string()))
}

fn resolve_memory_path(project_id: &str, relative_path: &str) -> TauriResult<PathBuf> {
  let project_id = SurrealId::try_from(project_id).map_err(anyhow::Error::from).into_tauri()?;

  let candidate = PathBuf::from(relative_path);
  if candidate.as_os_str().is_empty() {
    return Err(anyhow::anyhow!("memory path must not be empty")).into_tauri();
  }
  if candidate.is_absolute()
    || candidate
      .components()
      .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
  {
    return Err(anyhow::anyhow!("memory path must be a relative path within the project memory root")).into_tauri();
  }

  Ok(project_memory_root(&project_id.key().to_string())?.join(candidate))
}
