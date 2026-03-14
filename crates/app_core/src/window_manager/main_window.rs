use tauri::AppHandle;
use tauri::LogicalSize;
use tauri::Manager;
use tauri::Size;
use tauri::WebviewUrl;
use tauri::WebviewWindow;
use tauri::WebviewWindowBuilder;

use crate::consts::MAIN_WINDOW;

pub struct MainWindow;

impl MainWindow {
  pub fn render(app: AppHandle) -> WebviewWindow {
    let window = if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
      tracing::info!("Window already exists");
      let _ = window.set_min_size(Some(Size::Logical(LogicalSize::new(1280.0, 720.0))));
      let _ = window.set_title("blprnt");

      window
    } else {
      tracing::info!("Creating new window");
      WebviewWindowBuilder::new(&app, MAIN_WINDOW, WebviewUrl::App("/index.html".into()))
        .title("blprnt")
        .resizable(true)
        .minimizable(true)
        .maximizable(true)
        .decorations(true)
        .min_inner_size(1280.0, 720.0)
        .visible(false)
        .shadow(true)
        .build()
        .expect("failed to build main window")
    };

    #[cfg(debug_assertions)]
    {
      use tauri_plugin_window_state::StateFlags;
      use tauri_plugin_window_state::WindowExt;

      let _ = window.restore_state(StateFlags::POSITION | StateFlags::SIZE);
      tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        let window = app.get_webview_window(MAIN_WINDOW).expect("no main window");
        window.open_devtools();
      });
    }

    window
  }
}
