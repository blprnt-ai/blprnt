mod logging;

use crate::logging::init_logging;
use clap::Parser;
use clap::Subcommand;
use employee_import::DEFAULT_EMPLOYEES_REPO_URL;
use employee_import::EmployeeLibrarySource;
use employee_import::ImportEmployeeRequest;
use persistence::prelude::DbId;

#[derive(Debug, Parser)]
#[command(name = "blprnt")]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
  Import {
    slug: String,
    #[arg(long)]
    force: bool,
  },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_logging();
  let cli = Cli::parse();

  match cli.command {
    None => run_backend().await,
    Some(Commands::Import { slug, force }) => import_employee(slug, force).await,
  }
}

async fn import_employee(slug: String, force: bool) -> anyhow::Result<()> {
  let workspace_root = std::env::current_dir()?;
  let imported = employee_import::import_employee(ImportEmployeeRequest {
    slug,
    source: employee_library_source(),
    workspace_root,
    reports_to: None,
    force,
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

  tokio::join!(api, adapter, coordinator);

  tracing::info!("Blprnt backend started");

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
  use super::*;

  #[test]
  fn parses_import_command_with_force() {
    let cli = Cli::parse_from(["blprnt", "import", "data-analyst", "--force"]);
    match cli.command {
      Some(Commands::Import { slug, force }) => {
        assert_eq!(slug, "data-analyst");
        assert!(force);
      }
      _ => panic!("expected import command"),
    }
  }

  #[test]
  fn parses_no_args_as_backend_mode() {
    let cli = Cli::parse_from(["blprnt"]);
    assert!(cli.command.is_none());
  }
}
