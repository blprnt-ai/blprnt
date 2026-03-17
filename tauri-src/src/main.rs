#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use app_core::EngineManager;
use app_core::consts::MAIN_WINDOW;
use app_core::menu::DOCUMENTATION_MENU_ITEM_ID;
use app_core::menu::REPORT_BUG_MENU_ITEM_ID;
use app_core::menu::VIEW_LICENSE_MENU_ITEM_ID;
use common::blprnt::Blprnt;
use common::blprnt::BlprntEventKind;
use common::blprnt::ReportBugMenuClicked;
use common::consts::SURREAL_DB_PORT;
use common::paths::BlprntPath;
use common::paths::DATA_DIR;
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::ShellExt;
mod setup;
mod surreal_guard;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
  let _ = dotenvy::dotenv();

  let handles = Arc::new(Mutex::new(Vec::new()));

  let app = tauri::Builder::default()
    .setup({
      let handles = handles.clone();

      move |app| {
        #[cfg(not(debug_assertions))]
        setup::logging();

        #[cfg(debug_assertions)]
        setup::logging(app.handle());

        let _ = app.handle().plugin(tauri_plugin_shell::init());

        setup::setup(app.handle())?;

        let spawn_surreal = |port: u16, db_path: PathBuf, label: &str| {
          let pid_lock = BlprntPath::blprnt_home().join(DATA_DIR).join(format!("surreal.{label}.pid"));
          surreal_guard::ensure_surreal_port_is_free(port, &pid_lock, label);

          let command = match app.shell().sidecar("surreal") {
            Ok(command) => command,
            Err(err) => {
              tracing::error!("Failed to load surreal {} sidecar: {:?}", label, err);
              return;
            }
          };

          let spawn_result = command
            .arg("start")
            .arg("-b")
            .arg(format!("127.0.0.1:{}", port))
            .arg("--unauthenticated")
            .arg(format!("rocksdb:{}", db_path.to_string_lossy()))
            .spawn();

          match spawn_result {
            Ok((_, handle)) => {
              tracing::info!("Spawned surreal {}", label);
              surreal_guard::write_surreal_pid_lock(&pid_lock, handle.pid());
              handles.clone().lock().unwrap().push(handle);
            }
            Err(err) => {
              tracing::error!("Failed to spawn surreal {}: {:?}", label, err);
            }
          }
        };

        let db_path = BlprntPath::blprnt_home().join(DATA_DIR).join("surreal.v3.rocks.db");
        tracing::info!("Spawning surreal primary: {:?}", db_path);
        spawn_surreal(SURREAL_DB_PORT, db_path, "primary");

        Ok(())
      }
    })
    .plugin(tauri_plugin_clipboard::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_notification::init())
    .invoke_handler(app_core::builder().invoke_handler())
    .build(tauri::generate_context!());

  let app = match app {
    Ok(app) => app,
    Err(err) => {
      eprintln!("Failed to build Tauri app: {:?}", err);
      return;
    }
  };

  let previous_panic_hook = panic::take_hook();

  panic::set_hook({
    let handles = handles.clone();

    Box::new(move |info| {
      if let Ok(mut handles) = handles.lock() {
        while let Some(handle) = handles.pop() {
          let _ = handle.kill();
        }
      }

      let pid_lock = BlprntPath::blprnt_home().join(DATA_DIR).join("surreal.primary.pid");
      surreal_guard::clear_surreal_pid_lock(&pid_lock);

      previous_panic_hook(info);
    })
  });

  app.run({
    let handles = handles.clone();

    move |app, event| match event {
      tauri::RunEvent::MenuEvent(menu_event) => {
        if menu_event.id().as_ref() == REPORT_BUG_MENU_ITEM_ID {
          Blprnt::emit(BlprntEventKind::ReportBugMenuClicked, ReportBugMenuClicked.into());
        }
        if menu_event.id().as_ref() == DOCUMENTATION_MENU_ITEM_ID {
          let _ = app.opener().open_url("https://docs.blprnt.ai", None::<&str>);
        }
        if menu_event.id().as_ref() == VIEW_LICENSE_MENU_ITEM_ID {
          let _ = app.opener().open_url("https://blprnt.ai/terms", None::<&str>);
        }
      }
      tauri::RunEvent::WindowEvent { label, event, .. } => {
        if label == MAIN_WINDOW
          && let tauri::WindowEvent::Focused(focused) = event
        {
          let slack = app.state::<Arc<EngineManager>>().slack.clone();
          let transitioned_to_unfocused = slack.set_app_focused(focused);
          if transitioned_to_unfocused {
            tauri::async_runtime::spawn(async move {
              slack.try_deliver_all_queued_ask_questions().await;
            });
          }
        }
      }
      tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. } => {
        tracing::info!("Killing surreal processes");
        while let Some(handle) = handles.lock().unwrap().pop() {
          let _ = handle.kill();
        }
        let pid_lock = BlprntPath::blprnt_home().join(DATA_DIR).join("surreal.primary.pid");
        surreal_guard::clear_surreal_pid_lock(&pid_lock);
      }
      _ => {}
    }
  });
}
