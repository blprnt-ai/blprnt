use common::bun_runtime::BunRuntimeInstallResult;
use common::bun_runtime::BunRuntimeStatus;
use common::bun_runtime::JsRuntimeHealthStatus;
use common::bun_runtime::JsRuntimeInstallResult;
use common::bun_runtime::bun_runtime_install;
use common::bun_runtime::js_runtime_install_managed;
use common::bun_runtime::load_bun_runtime_status;
use common::bun_runtime::load_js_runtime_health_status;
use common::errors::TauriError;
use common::errors::TauriResult;

#[tauri::command]
#[specta::specta]
pub async fn bun_runtime_status() -> TauriResult<BunRuntimeStatus> {
  Ok(load_bun_runtime_status())
}

#[tauri::command]
#[specta::specta]
pub async fn bun_runtime_install_user_local(overwrite: bool) -> TauriResult<BunRuntimeInstallResult> {
  bun_runtime_install(overwrite).await.map_err(|e| TauriError::new(e.to_string()))
}

#[tauri::command]
#[specta::specta]
pub async fn js_runtime_health_status() -> TauriResult<JsRuntimeHealthStatus> {
  Ok(load_js_runtime_health_status())
}

#[tauri::command]
#[specta::specta]
pub async fn js_runtime_install_managed_runtime(overwrite: bool) -> TauriResult<JsRuntimeInstallResult> {
  js_runtime_install_managed(overwrite).await.map_err(|e| TauriError::new(e.to_string()))
}
