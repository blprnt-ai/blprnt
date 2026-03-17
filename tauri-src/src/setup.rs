use anyhow::Result;
#[cfg(debug_assertions)]
use colored::Colorize;
use common::blprnt::Blprnt;
use tauri::AppHandle;
use tauri::Error;

pub fn setup(app: &AppHandle) -> Result<(), Error> {
  tracing::info!("Runtime workers: {:?}", tokio::runtime::Handle::current().metrics().num_workers());

  Blprnt::init(app);

  #[cfg(not(debug_assertions))]
  let _ = app.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
    use app_core::consts::MAIN_WINDOW;
    use tauri::Manager;
    let _ = app.get_webview_window(MAIN_WINDOW).expect("no main window").set_focus();

    tracing::info!("Single instance: {:?}", argv);
  }));

  let _ = app.plugin(tauri_plugin_window_state::Builder::default().build());
  let _ = app.plugin(tauri_plugin_process::init());
  let _ = app.plugin(tauri_plugin_updater::Builder::default().build());
  let _ = app.plugin(tauri_plugin_cors_fetch::init());
  let _ = app.plugin(tauri_plugin_http::init());
  let _ = app.plugin(tauri_plugin_store::Builder::new().build());
  let _ = app.plugin(tauri_plugin_prevent_default::debug());

  tauri::async_runtime::spawn(app_core::setup::setup_app(app.clone()));

  Ok(())
}

#[cfg(debug_assertions)]
pub fn logging(app: &AppHandle) {
  use tauri_plugin_log::Target;
  use tauri_plugin_log::TargetKind;
  use tauri_plugin_log::TimezoneStrategy;
  use tauri_plugin_log::log::LevelFilter;

  let targets = vec![Target::new(TargetKind::Stdout)];

  let _ = app.plugin(
    tauri_plugin_log::Builder::new()
      .level(LevelFilter::Warn)
      .level_for("blprnt", LevelFilter::Info)
      // crates
      .level_for("app_core", LevelFilter::Debug)
      .level_for("common", LevelFilter::Info)
      .level_for("engine_v2", LevelFilter::Info)
      .level_for("indexing", LevelFilter::Info)
      .level_for("json_repair", LevelFilter::Info)
      .level_for("oauth", LevelFilter::Info)
      .level_for("persistence", LevelFilter::Debug)
      .level_for("prompt", LevelFilter::Debug)
      .level_for("providers", LevelFilter::Trace)
      .level_for("sandbox", LevelFilter::Info)
      .level_for("session", LevelFilter::Info)
      .level_for("tauri_plugin_oauth", LevelFilter::Info)
      .level_for("tools", LevelFilter::Trace)
      .level_for("tunnel-client", LevelFilter::Trace)
      .level_for("tunnel_client", LevelFilter::Trace)
      .level_for("vault", LevelFilter::Info)
      .clear_targets()
      .timezone_strategy(TimezoneStrategy::UseLocal)
      .format(|out, message, record| {
        use tauri_plugin_log::log::Level;

        let level = match record.level() {
          // Red
          Level::Error => format!("{}", "ERROR".truecolor(208, 0, 0)),
          // Orange
          Level::Warn => format!("{}", "WARN!".truecolor(208, 85, 0)),
          // Yellow
          Level::Info => format!("{}", ">INFO".truecolor(208, 208, 0)),
          // Green
          Level::Debug => format!("{}", "DEBUG".truecolor(0, 208, 0)),
          // Blue
          Level::Trace => format!("{}", "TRACE".truecolor(0, 85, 208)),
        };

        let filename = record.file().map(|f| f.to_string().truecolor(125, 125, 125));
        let line_number = record.line().map(|l| l.to_string().truecolor(125, 125, 125));
        let timestamp = chrono::Local::now().format("%I:%M:%S%.6f").to_string().truecolor(25, 100, 25);
        let target = record.target().to_string().truecolor(125, 125, 125);

        let file = filename
          .map(|f| if let Some(line_number) = line_number { format!("{}:{}", f, line_number) } else { f.to_string() })
          .map(|f| f.truecolor(125, 125, 125));

        let middle_glpyh = "├─".to_string().truecolor(125, 125, 125);
        let last_glyph = "└─".to_string().truecolor(125, 125, 125);

        let message = message
          .to_string()
          .lines()
          .enumerate()
          .map(|(i, line)| if i == 0 { line.to_string() } else { format!("       {line}") })
          .collect::<Vec<String>>()
          .join("\n")
          .truecolor(200, 200, 200);

        if let Some(file) = file {
          out.finish(format_args!(
            "{level}  {timestamp}\n    {middle_glpyh} {target}\n    {middle_glpyh} {file}\n    {last_glyph} {message}"
          ));
        } else {
          out.finish(format_args!("{level}  {timestamp}\n    {middle_glpyh} {target}\n    {last_glyph} {message}"));
        }
      })
      .targets(targets)
      .build(),
  );
}

#[cfg(not(debug_assertions))]
pub fn logging() {
  use tracing_subscriber::prelude::*;
  tracing_subscriber::registry().with(tracing_subscriber::fmt::layer()).init();
}
