mod logging;

use std::future::Future;
use std::path::PathBuf;

use clap::ArgAction;
use clap::Parser;
use clap::Subcommand;
use employee_import::DEFAULT_EMPLOYEES_REPO_URL;
use employee_import::EmployeeLibrarySource;
use employee_import::ImportEmployeeRequest;
use persistence::prelude::DbId;

use crate::logging::init_logging;

async fn wait_for_shutdown_or_completion<Api, Adapter, Coordinator, Shutdown>(
  api: Api,
  adapter: Adapter,
  coordinator: Coordinator,
  shutdown: Shutdown,
) where
  Api: Future,
  Adapter: Future,
  Coordinator: Future,
  Shutdown: Future,
{
  tokio::select! {
    _ = async {
      tokio::join!(api, adapter, coordinator);
    } => {
      tracing::info!("Blprnt runtime exited");
    }
    _ = shutdown => {
      tracing::info!("Shutdown signal received, exiting");
    }
  }
}

#[derive(Debug, Parser)]
#[command(name = "blprnt", disable_help_flag = true)]
struct Cli {
  #[arg(long = "home_dir", short = 'd', global = true)]
  home_dir: Option<PathBuf>,
  #[arg(long, short = 'p', global = true)]
  port:     Option<u16>,
  #[arg(long = "help", action = ArgAction::Help, global = true)]
  help:     Option<bool>,
  #[command(subcommand)]
  command:  Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
  Import {
    slug:                  String,
    #[arg(long)]
    force:                 bool,
    #[arg(long)]
    skip_duplicate_skills: bool,
    #[arg(long)]
    force_skills:          bool,
  },
}

#[derive(Debug, PartialEq, Eq)]
struct RuntimeConfig {
  home_dir: Option<PathBuf>,
  api_port: u16,
}

impl RuntimeConfig {
  fn from_cli(cli: &Cli) -> Self {
    Self {
      home_dir: cli.home_dir.clone().or_else(|| std::env::var_os("BLPRNT_HOME").map(PathBuf::from)),
      api_port: cli
        .port
        .or_else(|| std::env::var("BLPRNT_API_PORT").ok().and_then(|value| value.parse::<u16>().ok()))
        .unwrap_or(api::DEFAULT_PORT),
    }
  }

  fn apply(&self, cli: &Cli) {
    if let Some(home_dir) = cli.home_dir.as_ref() {
      unsafe { std::env::set_var("BLPRNT_HOME", home_dir) };
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_logging();
  let cli = Cli::parse();
  let runtime_config = RuntimeConfig::from_cli(&cli);
  runtime_config.apply(&cli);
  bootstrap_runtime_assets()?;

  match cli.command {
    None => run_backend(runtime_config.api_port).await,
    Some(Commands::Import { slug, force, skip_duplicate_skills, force_skills }) => {
      import_employee(slug, force, skip_duplicate_skills, force_skills).await
    }
  }
}

fn bootstrap_runtime_assets() -> anyhow::Result<()> {
  skills::ensure_builtin_skills_installed()
}

async fn import_employee(
  slug: String,
  force: bool,
  skip_duplicate_skills: bool,
  force_skills: bool,
) -> anyhow::Result<()> {
  let workspace_root = std::env::current_dir()?;
  let imported = employee_import::import_employee(ImportEmployeeRequest {
    slug,
    source: employee_library_source(),
    workspace_root,
    reports_to: None,
    force,
    skip_duplicate_skills,
    force_skills,
  })
  .await?;

  println!(
    "{} {} ({})",
    match imported.action {
      employee_import::ImportEmployeeAction::Created => "Imported",
      employee_import::ImportEmployeeAction::Updated => "Updated",
    },
    imported.employee.name,
    imported.employee.id.uuid()
  );
  Ok(())
}

async fn run_backend(api_port: u16) -> anyhow::Result<()> {
  #[cfg(feature = "api")]
  let api = {
    tracing::debug!("Starting API server");
    api::start_server(api_port)
  };
  #[cfg(not(feature = "api"))]
  let api = {
    tracing::debug!("API server disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

  #[cfg(feature = "adapter")]
  let adapter = {
    tracing::debug!("Starting adapter server");
    adapters::runtime::AdapterRuntime::new().listen()
  };
  #[cfg(not(feature = "adapter"))]
  let adapter = {
    tracing::debug!("Adapter server disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

  #[cfg(feature = "coordinator")]
  let coordinator = {
    tracing::debug!("Starting coordinator");
    let coordinator = coordinator::Coordinator::new();
    coordinator.init().await?;
    coordinator.listen()
  };
  #[cfg(not(feature = "coordinator"))]
  let coordinator = {
    tracing::debug!("Coordinator disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

  #[cfg(feature = "api")]
  println!("{}", api::startup_banner());

  // #[cfg(all(feature = "api", not(debug_assertions)))]
  webbrowser::open(&format!("http://localhost:{api_port}")).expect("failed to open browser");

  wait_for_shutdown_or_completion(api, adapter, coordinator, tokio::signal::ctrl_c()).await;

  Ok(())
}

fn employee_library_source() -> EmployeeLibrarySource {
  match std::env::var("BLPRNT_EMPLOYEES_REPO") {
    Ok(value) => {
      let path = std::path::PathBuf::from(&value);
      if path.exists() { EmployeeLibrarySource::Local(path) } else { EmployeeLibrarySource::GitUrl(value) }
    }
    Err(_) => EmployeeLibrarySource::GitUrl(DEFAULT_EMPLOYEES_REPO_URL.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;
  use std::sync::Mutex;
  use std::sync::Arc;
  use std::sync::atomic::AtomicBool;
  use std::sync::atomic::Ordering;
  use std::time::Duration;

  use tempfile::TempDir;
  use tokio::sync::oneshot;

  use super::*;

  static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

  fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  struct HomeGuard {
    previous_home: Option<String>,
  }

  impl HomeGuard {
    fn set(temp_home: &TempDir) -> Self {
      let previous_home = std::env::var("HOME").ok();
      unsafe { std::env::set_var("HOME", temp_home.path()) };
      Self { previous_home }
    }
  }

  impl Drop for HomeGuard {
    fn drop(&mut self) {
      match &self.previous_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
      }
    }
  }

  #[tokio::test]
  async fn exits_when_shutdown_signal_arrives() {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let finished = Arc::new(AtomicBool::new(false));
    let finished_clone = finished.clone();

    let task = tokio::spawn(async move {
      wait_for_shutdown_or_completion(
        std::future::pending::<()>(),
        std::future::pending::<()>(),
        std::future::pending::<()>(),
        async move {
          let _ = shutdown_rx.await;
        },
      )
      .await;

      finished_clone.store(true, Ordering::SeqCst);
    });

    tokio::time::sleep(Duration::from_millis(25)).await;
    assert!(!finished.load(Ordering::SeqCst), "runtime should still be waiting before shutdown");

    shutdown_tx.send(()).expect("shutdown signal should be delivered");

    tokio::time::timeout(Duration::from_secs(1), task)
      .await
      .expect("runtime should exit after shutdown")
      .expect("shutdown task should complete");

    assert!(finished.load(Ordering::SeqCst), "runtime should stop after shutdown");
  }

  #[tokio::test]
  async fn exits_when_subsystems_finish_first() {
    let result = tokio::time::timeout(
      Duration::from_secs(1),
      wait_for_shutdown_or_completion(async {}, async {}, async {}, std::future::pending::<()>()),
    )
    .await;

    assert!(result.is_ok(), "runtime should exit when subsystems complete");
  }

  #[test]
  fn parses_import_command_with_force() {
    let cli = Cli::parse_from(["blprnt", "import", "data-analyst", "--force", "--force-skills"]);
    match cli.command {
      Some(Commands::Import { slug, force, force_skills, skip_duplicate_skills }) => {
        assert_eq!(slug, "data-analyst");
        assert!(force);
        assert!(force_skills);
        assert!(!skip_duplicate_skills);
      }
      _ => panic!("expected import command"),
    }
  }

  #[test]
  fn parses_no_args_as_backend_mode() {
    let cli = Cli::parse_from(["blprnt"]);
    assert!(cli.command.is_none());
  }

  #[test]
  fn parses_backend_flags() {
    let cli = Cli::parse_from(["blprnt", "--home_dir", "/tmp/blprnt-home", "--port", "9310"]);

    assert_eq!(cli.home_dir.as_deref(), Some(std::path::Path::new("/tmp/blprnt-home")));
    assert_eq!(cli.port, Some(9310));
    assert!(cli.command.is_none());
  }

  #[test]
  fn runtime_config_prefers_cli_over_env() {
    let _lock = test_lock();
    let _home_guard = EnvGuard::set("BLPRNT_HOME", "/tmp/from-env");
    let _port_guard = EnvGuard::set("BLPRNT_API_PORT", "9222");
    let cli = Cli::parse_from(["blprnt", "-d", "/tmp/from-cli", "-p", "9333"]);

    let config = RuntimeConfig::from_cli(&cli);

    assert_eq!(config.home_dir, Some(std::path::PathBuf::from("/tmp/from-cli")));
    assert_eq!(config.api_port, 9333);
  }

  #[test]
  fn runtime_config_uses_env_when_cli_missing() {
    let _lock = test_lock();
    let _home_guard = EnvGuard::set("BLPRNT_HOME", "/tmp/from-env");
    let _port_guard = EnvGuard::set("BLPRNT_API_PORT", "9222");
    let cli = Cli::parse_from(["blprnt"]);

    let config = RuntimeConfig::from_cli(&cli);

    assert_eq!(config.home_dir, Some(std::path::PathBuf::from("/tmp/from-env")));
    assert_eq!(config.api_port, 9222);
  }

  #[test]
  fn bootstraps_builtin_skills_into_blprnt_home() {
    let _lock = test_lock();
    let home = TempDir::new().unwrap();
    let _guard = HomeGuard::set(&home);
    let _blprnt_home_guard = EnvGuard::set("BLPRNT_HOME", home.path().to_str().expect("temp home should be utf-8"));

    bootstrap_runtime_assets().unwrap();

    assert!(shared::paths::blprnt_builtin_skills_dir().join("blprnt").join("SKILL.md").is_file());
  }

  struct EnvGuard {
    key:      &'static str,
    previous: Option<std::ffi::OsString>,
  }

  impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
      let previous = std::env::var_os(key);
      unsafe { std::env::set_var(key, value) };
      Self { key, previous }
    }
  }

  impl Drop for EnvGuard {
    fn drop(&mut self) {
      match &self.previous {
        Some(value) => unsafe { std::env::set_var(self.key, value) },
        None => unsafe { std::env::remove_var(self.key) },
      }
    }
  }
}
