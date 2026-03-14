use std::sync::Arc;

use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use tauri::State;

use crate::engine_manager::EngineManager;
use crate::preview::PreviewSession;
use crate::preview::PreviewStartParams;
use crate::preview::PreviewStatusResponse;

#[tauri::command]
#[specta::specta]
pub async fn preview_start(
  manager: State<'_, Arc<EngineManager>>,
  params: PreviewStartParams,
) -> TauriResult<PreviewSession> {
  manager.preview_start(params).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn preview_stop(manager: State<'_, Arc<EngineManager>>, project_id: String) -> TauriResult<()> {
  manager.preview_stop(project_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn preview_reload(manager: State<'_, Arc<EngineManager>>, project_id: String) -> TauriResult<PreviewSession> {
  manager.preview_reload(project_id).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn preview_status(
  manager: State<'_, Arc<EngineManager>>,
  project_id: String,
) -> TauriResult<PreviewStatusResponse> {
  manager.preview_status(project_id).await.into_tauri()
}
