use std::sync::Arc;

use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use tauri::State;

use crate::engine_manager::EngineManager;
use crate::engine_manager::PersonalityModelDto;

#[tauri::command]
#[specta::specta]
pub async fn personality_list(manager: State<'_, Arc<EngineManager>>) -> TauriResult<Vec<PersonalityModelDto>> {
  tracing::debug!("List Personalities");
  manager.personality_list().await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn personality_create(
  manager: State<'_, Arc<EngineManager>>,
  name: String,
  description: String,
  system_prompt: String,
) -> TauriResult<PersonalityModelDto> {
  tracing::debug!("Create Personality: {}", name);

  manager.personality_create(name, description, system_prompt).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn personality_update(
  manager: State<'_, Arc<EngineManager>>,
  id: String,
  name: String,
  description: String,
  system_prompt: String,
) -> TauriResult<PersonalityModelDto> {
  tracing::debug!("Update Personality: {}", id);

  manager.personality_update(id, name, description, system_prompt).await.into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn personality_delete(manager: State<'_, Arc<EngineManager>>, id: String) -> TauriResult<()> {
  tracing::debug!("Delete Personality: {}", id);
  manager.personality_delete(id).await.into_tauri()
}
