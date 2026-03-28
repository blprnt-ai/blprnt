mod dto;
mod middleware;
mod provider_helpers;
mod routes;
mod state;

use axum::Router;
use tower_http::cors::AllowHeaders;
use tower_http::cors::AllowMethods;
use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;

pub async fn start_server() {
  tracing::info!("Starting Blprnt Api");

  // Add CORS
  let app = Router::new().merge(routes::routes()).layer(
    CorsLayer::new()
      .allow_origin(AllowOrigin::any())
      .allow_methods(AllowMethods::any())
      .allow_headers(AllowHeaders::any()),
  );

  let listener = tokio::net::TcpListener::bind("0.0.0.0:9171").await.expect("failed to bind to port 9171");
  tracing::info!("Listening on {}", listener.local_addr().expect("failed to get local address"));
  axum::serve(listener, app).await.expect("failed to start server");
}
