mod logging;

use crate::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_logging();

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
