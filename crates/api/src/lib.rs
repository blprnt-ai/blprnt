mod dto;
mod middleware;
mod provider_helpers;
mod routes;
mod state;

use std::net::SocketAddr;

use axum::Router;
use colored::Colorize;
use tower_http::cors::AllowHeaders;
use tower_http::cors::AllowMethods;
use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;

pub const DEFAULT_PORT: u16 = 9171;

pub fn startup_banner() -> String {
  let banner = format!(
    r#"
########################################
██╗     ██╗                        ██╗
██║     ██║                        ██║
██████╗ ██║██████╗ ██████╗██████╗██████╗
██╔══██╗██║██╔══██╗██╔═══╝██╔══██╗ ██╔═╝
██████╔╝██║██████╔╝██║    ██║  ██║ ██║
╚═════╝ ╚═╝██╔═══╝ ╚═╝    ╚═╝  ╚═╝ ╚═╝
           ╚═╝
########################################
"#
  )
  .truecolor(15, 146, 247)
  .to_string();

  banner
}

pub async fn start_server(port: u16) {
  tracing::info!("Starting Blprnt Api");

  // Add CORS
  let app = Router::new().merge(routes::routes()).layer(
    CorsLayer::new()
      .allow_origin(AllowOrigin::any())
      .allow_methods(AllowMethods::any())
      .allow_headers(AllowHeaders::any()),
  );

  let listener =
    tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await.expect("failed to bind to API port");

  axum::serve(listener, app).await.expect("failed to start server");
}
