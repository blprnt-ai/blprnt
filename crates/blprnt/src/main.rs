mod logging;

use coordinator::Coordinator;

use crate::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_logging();

  let coordinator = Coordinator::new();
  coordinator.init().await?;

  tokio::join!(api::start_server(), coordinator.listen());

  tracing::info!("Blprnt backend started");

  Ok(())
}
