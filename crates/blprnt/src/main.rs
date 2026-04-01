mod logging;

use std::future::Future;

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
#[command(name = "blprnt")]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_logging();
  bootstrap_runtime_assets()?;
  let cli = Cli::parse();

  match cli.command {
    None => run_backend().await,
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

async fn run_backend() -> anyhow::Result<()> {
  #[cfg(feature = "api")]
  let api = {
    tracing::info!("Starting API server");
    api::start_server()
  };
  #[cfg(not(feature = "api"))]
  let api = {
    tracing::info!("API server disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

  #[cfg(feature = "adapter")]
  let adapter = {
    tracing::info!("Starting adapter server");
    adapters::runtime::AdapterRuntime::new().listen()
  };
  #[cfg(not(feature = "adapter"))]
  let adapter = {
    tracing::info!("Adapter server disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

  #[cfg(feature = "coordinator")]
  let coordinator = {
    tracing::info!("Starting coordinator");
    let coordinator = coordinator::Coordinator::new();
    coordinator.init().await?;
    coordinator.listen()
  };
  #[cfg(not(feature = "coordinator"))]
  let coordinator = {
    tracing::info!("Coordinator disabled");
    tokio::time::sleep(std::time::Duration::from_secs(0))
  };

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
  use tempfile::TempDir;
  use std::sync::Arc;
  use std::sync::atomic::AtomicBool;
  use std::sync::atomic::Ordering;
  use std::time::Duration;

  use tokio::sync::oneshot;

  use super::*;

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
  fn bootstraps_builtin_skills_into_blprnt_home() {
    let home = TempDir::new().unwrap();
    let _guard = HomeGuard::set(&home);

    bootstrap_runtime_assets().unwrap();

    assert!(shared::paths::blprnt_builtin_skills_dir().join("blprnt").join("SKILL.md").is_file());
  }
}
