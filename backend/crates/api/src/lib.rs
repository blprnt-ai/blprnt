mod config;
mod dto;
mod mcp_oauth;
mod middleware;
mod provider_helpers;
mod routes;
mod state;
mod telegram;

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::http::HeaderValue;
use axum::http::request::Parts as RequestParts;
use colored::Colorize;
use events::ADAPTER_EVENTS;
use events::AdapterEvent;
use tower_http::cors::AllowHeaders;
use tower_http::cors::AllowMethods;
use tower_http::cors::AllowOrigin;
use tower_http::cors::CorsLayer;
use url::Url;

use crate::config::deployed_mode;

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

fn app() -> Router {
  Router::new().merge(routes::routes()).layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
  CorsLayer::new()
    .allow_credentials(true)
    .allow_headers(AllowHeaders::mirror_request())
    .allow_methods(AllowMethods::mirror_request())
    .allow_origin(AllowOrigin::predicate(is_allowed_origin))
}

fn is_allowed_origin(origin: &HeaderValue, _: &RequestParts) -> bool {
  let Ok(origin) = origin.to_str() else {
    return false;
  };

  if let Ok(configured_origins) = env::var("BLPRNT_CORS_ORIGINS") {
    if configured_origins.split(',').map(str::trim).filter(|value| !value.is_empty()).any(|value| value == origin) {
      return true;
    }
  }

  if deployed_mode() {
    return false;
  }

  let Ok(url) = Url::parse(origin) else {
    return false;
  };

  matches!(url.scheme(), "http" | "https")
    && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "0.0.0.0" | "::1"))
}

pub async fn start_server(port: u16) {
  tracing::info!("Starting Blprnt Api");

  tokio::spawn(async move {
    telegram::run_polling_loop().await;
  });

  tokio::spawn(async move {
    let mut adapter_events = ADAPTER_EVENTS.subscribe();
    loop {
      match adapter_events.recv().await {
        Ok(AdapterEvent::RunCompleted { run_id }) | Ok(AdapterEvent::RunFailed { run_id, .. }) => {
          if let Err(error) = telegram::notify_run_terminal_status(run_id).await {
            tracing::error!(?error, "failed to send telegram run notification");
          }
        }
        Ok(_) => {}
        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
          tracing::warn!(skipped, "telegram notification listener lagged behind adapter events");
        }
        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
          tokio::time::sleep(Duration::from_millis(100)).await;
          break;
        }
      }
    }
  });

  let app = app();

  let listener =
    tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await.expect("failed to bind to API port");

  axum::serve(listener, app).await.expect("failed to start server");
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;
  use std::sync::Mutex;

  use axum::Router;
  use axum::body::Body;
  use axum::http::Request;
  use axum::http::header;
  use std::env;
  use tower::ServiceExt;

  use super::cors_layer;

  static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

  struct EnvVarGuard {
    key:      &'static str,
    previous: Option<String>,
  }

  impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
      let previous = env::var(key).ok();
      unsafe { env::set_var(key, value) };
      Self { key, previous }
    }
  }

  impl Drop for EnvVarGuard {
    fn drop(&mut self) {
      match &self.previous {
        Some(value) => unsafe { env::set_var(self.key, value) },
        None => unsafe { env::remove_var(self.key) },
      }
    }
  }

  fn api_test_app() -> Router {
    Router::new().merge(crate::routes::v1::routes()).layer(cors_layer())
  }

  #[tokio::test]
  async fn cors_preflight_allows_localhost_dev_origin_with_credentials() {
    let _lock = ENV_LOCK.lock().unwrap();

    let response = api_test_app()
      .oneshot(
        Request::builder()
          .method("OPTIONS")
          .uri("/api/v1/auth/login")
          .header(header::ORIGIN, "http://localhost:5173")
          .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
          .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(
      response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).and_then(|value| value.to_str().ok()),
      Some("http://localhost:5173")
    );
    assert_eq!(
      response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
        .and_then(|value| value.to_str().ok()),
      Some("true")
    );
  }

  #[tokio::test]
  async fn cors_preflight_requires_explicit_origin_configuration_in_deployed_mode() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _deployed = EnvVarGuard::set("BLPRNT_DEPLOYED", "true");

    let response = api_test_app()
      .oneshot(
        Request::builder()
          .method("OPTIONS")
          .uri("/api/v1/auth/login")
          .header(header::ORIGIN, "https://app.example.com")
          .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
          .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert!(response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
  }

  #[tokio::test]
  async fn cors_preflight_allows_configured_origin_in_deployed_mode() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _deployed = EnvVarGuard::set("BLPRNT_DEPLOYED", "true");
    let _origins = EnvVarGuard::set("BLPRNT_CORS_ORIGINS", "https://app.example.com");

    let response = api_test_app()
      .oneshot(
        Request::builder()
          .method("OPTIONS")
          .uri("/api/v1/auth/login")
          .header(header::ORIGIN, "https://app.example.com")
          .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
          .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(
      response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).and_then(|value| value.to_str().ok()),
      Some("https://app.example.com")
    );
  }
}
