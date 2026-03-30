mod logging;

use crate::logging::init_logging;
use std::future::Future;

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

  wait_for_shutdown_or_completion(api, adapter, coordinator, tokio::signal::ctrl_c()).await;

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;
  use std::sync::atomic::AtomicBool;
  use std::sync::atomic::Ordering;
  use std::time::Duration;

  use tokio::sync::oneshot;

  use super::wait_for_shutdown_or_completion;

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
}
