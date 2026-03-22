mod dto;
mod middleware;
mod provider_helpers;
mod routes;
mod state;

use axum::Router;

pub async fn start_server() {
  tracing::info!("Starting Blprnt Api");

  let app = Router::new().merge(routes::routes());

  let listener = tokio::net::TcpListener::bind("0.0.0.0:9171").await.expect("failed to bind to port 9171");
  tracing::info!("Listening on {}", listener.local_addr().expect("failed to get local address"));
  axum::serve(listener, app).await.expect("failed to start server");
}
