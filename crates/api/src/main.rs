mod middleware;
mod routes;
mod state;

use axum::Router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();
  tracing::info!("Starting Blprnt Api");

  let app = Router::new().merge(routes::routes());

  let listener = tokio::net::TcpListener::bind("0.0.0.0:9171").await?;
  tracing::info!("Listening on {}", listener.local_addr()?);
  axum::serve(listener, app).await?;

  Ok(())
}
