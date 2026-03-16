use common::blprnt::Blprnt;
use common::blprnt::BlprntEventKind;
use common::errors::TauriResult;
use tauri::AppHandle;
use tauri::Manager;

use crate::consts::LOADER_WINDOW;
use crate::consts::MAIN_WINDOW;
use crate::menu;

#[tauri::command]
#[specta::specta]
pub async fn frontend_ready(app: AppHandle) -> TauriResult<()> {
  tracing::debug!("Frontend Ready");

  let window = app.get_webview_window(MAIN_WINDOW).expect("no main window");
  let _ = window.show();

  tracing::trace!("Setting menu");
  let menu = menu::create_menu(&app).expect("failed to create menu");
  let _ = app.set_menu(menu);

  #[cfg(target_os = "windows")]
  let _ = window.hide_menu();

  if let Some(loader) = app.get_webview_window(LOADER_WINDOW) {
    tracing::trace!("Destroying loader window");
    let _ = loader.destroy();
  }

  #[cfg(not(debug_assertions))]
  let _ = window.set_focus();

  Blprnt::emit(BlprntEventKind::BackendReady, ().into());

  Ok(())
}

#[tauri::command]
#[specta::specta]
#[cfg(debug_assertions)]
pub async fn open_devtools(app: tauri::AppHandle) -> TauriResult<()> {
  tracing::debug!("Open DevTools");
  if let Some(win) = app.get_webview_window(MAIN_WINDOW) {
    win.open_devtools();
  }

  Ok(())
}

#[tauri::command]
#[specta::specta]
#[cfg(not(debug_assertions))]
pub async fn open_devtools() -> TauriResult<()> {
  Ok(())
}

#[tauri::command]
#[specta::specta]
#[cfg(debug_assertions)]
pub async fn reload_window(app: tauri::AppHandle) -> TauriResult<()> {
  tracing::debug!("Reload Window");
  if let Some(win) = app.get_webview_window(MAIN_WINDOW) {
    let _ = win.reload();
  }

  Ok(())
}

#[tauri::command]
#[specta::specta]
#[cfg(not(debug_assertions))]
pub async fn reload_window() -> TauriResult<()> {
  Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_build_hash() -> TauriResult<String> {
  let build_hash = env!("BUILD_HASH");

  Ok(build_hash.to_string())
}
