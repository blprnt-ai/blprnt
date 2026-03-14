use std::sync::Arc;

use common::api::ApiClient;
use tauri::AppHandle;
use tauri::Manager;

use crate::consts::MAIN_WINDOW;
use crate::engine_manager::EngineManager;
use crate::window_manager::main_window::MainWindow;

pub async fn setup_app(app: AppHandle) {
  tracing::info!("Setting up backend");
  ApiClient::set();

  tracing::info!("Setting up engine manager");
  let manager = Arc::new(EngineManager::new());
  app.manage(manager.clone());

  let _ = MainWindow::render(app.clone());
  tracing::info!("Main window restored");

  if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
    match window.is_focused() {
      Ok(focused) => {
        let _ = manager.slack.set_app_focused(focused);
      }
      Err(error) => tracing::warn!("Failed to seed backend app focus state: {error}"),
    }
  }

  let result = manager.init().await;
  match result {
    Ok(_) => {
      tracing::info!("Engine manager initialized");
    }
    Err(e) => {
      tracing::error!("Failed to initialize engine manager: {}", e);
    }
  }
}
