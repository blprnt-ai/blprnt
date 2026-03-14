#![allow(unused)]
use std::net::SocketAddr;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::connect_info::ConnectInfo;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::Result;
use axum::routing::any;
use common::errors::TauriError;
use futures_util::SinkExt;
use futures_util::StreamExt;
use http_body_util::BodyExt;
use hyper::Uri;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tower_http::services::ServeDir;
use url::Url;

use crate::preview::instrumentation::PREVIEW_INSTRUMENTATION_EVENT_PATH;
use crate::preview::instrumentation::PREVIEW_INSTRUMENTATION_SCRIPT_PATH;
use crate::preview::instrumentation::PreviewInstrumentationConfig;
use crate::preview::instrumentation::PreviewInstrumentationEventRequest;
use crate::preview::instrumentation::emit_instrumentation_event;
use crate::preview::instrumentation::inject_instrumentation;
use crate::preview::instrumentation::instrumentation_script;
use crate::preview::instrumentation::is_html_response;

const DEFAULT_ALLOWED_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];

#[derive(Clone, Debug)]
pub struct ProxyConfig {
  pub target_url:      String,
  pub allowed_hosts:   Vec<String>,
  pub instrumentation: PreviewInstrumentationConfig,
}

#[derive(Clone)]
pub struct ProxyState {
  pub config: ProxyConfig,
  pub client: Client<hyper_util::client::legacy::connect::HttpConnector, Body>,
}

pub fn default_allowed_hosts() -> Vec<String> {
  DEFAULT_ALLOWED_HOSTS.iter().map(|host| host.to_string()).collect()
}

pub fn proxy_router(state: ProxyState) -> Router {
  Router::new().route("/", any(proxy_handler)).route("/{*path}", any(proxy_handler)).with_state(state)
}

pub fn instrumentation_router(config: PreviewInstrumentationConfig) -> Router {
  Router::new()
    .route(PREVIEW_INSTRUMENTATION_SCRIPT_PATH, any(instrumentation_script_handler))
    .route(PREVIEW_INSTRUMENTATION_EVENT_PATH, any(instrumentation_event_handler))
    .with_state(config)
}

pub async fn proxy_handler(
  ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
  axum::extract::State(state): axum::extract::State<ProxyState>,
  req: Request,
) -> Result<Response, TauriError> {
  if !is_allowed_host(&remote_addr, &state.config.allowed_hosts) {
    return Ok((StatusCode::FORBIDDEN, "forbidden").into_response());
  }

  if state.config.instrumentation.enabled {
    if req.uri().path() == PREVIEW_INSTRUMENTATION_SCRIPT_PATH {
      return instrumentation_script_handler(axum::extract::State(state.config.instrumentation.clone())).await;
    }

    if req.uri().path() == PREVIEW_INSTRUMENTATION_EVENT_PATH {
      return instrumentation_event_handler(
        ConnectInfo(remote_addr),
        axum::extract::State(state.config.instrumentation.clone()),
        req,
      )
      .await;
    }
  }

  if is_upgrade_request(&req) {
    return handle_websocket_proxy(req, &state).await;
  }

  let target_uri = build_target_uri(&state.config.target_url, &req)?;
  let (mut parts, body) = req.into_parts();
  parts.uri = target_uri;
  if let Some(authority) = parts.uri.authority() {
    let host = HeaderValue::from_str(authority.as_str())
      .map_err(|err| TauriError::new(format!("Invalid host header: {}", err)))?;
    parts.headers.insert(header::HOST, host);
  }
  let mut outbound = Request::from_parts(parts, Body::empty());

  let body_bytes = axum::body::to_bytes(body, usize::MAX)
    .await
    .map_err(|err| TauriError::new(format!("Failed to read proxy body: {}", err)))?;
  *outbound.body_mut() = Body::from(body_bytes);

  let response =
    state.client.request(outbound).await.map_err(|err| TauriError::new(format!("Proxy request failed: {}", err)))?;

  let (mut parts, body) = response.into_parts();

  if state.config.instrumentation.enabled && is_html_response(parts.headers.get(header::CONTENT_TYPE)) {
    let body_bytes = body
      .collect()
      .await
      .map_err(|err| TauriError::new(format!("Failed to read proxy response body: {}", err)))?
      .to_bytes();
    let body_str =
      String::from_utf8(body_bytes.to_vec()).map_err(|err| TauriError::new(format!("Invalid UTF-8 body: {}", err)))?;
    let injected = inject_instrumentation(&body_str);
    parts.headers.remove(header::CONTENT_LENGTH);
    let response = Response::from_parts(parts, Body::from(injected));
    return Ok(response);
  }

  let stream = body.into_data_stream().map(|chunk| chunk.map_err(std::io::Error::other));
  let proxied_body = Body::from_stream(stream);
  let response = Response::from_parts(parts, proxied_body);
  Ok(response)
}

fn is_allowed_host(remote_addr: &SocketAddr, allowlist: &[String]) -> bool {
  if remote_addr.ip().is_loopback() {
    return true;
  }

  allowlist.iter().any(|allowed| allowed == "*" || allowed == &remote_addr.ip().to_string())
}

fn is_upgrade_request(req: &Request) -> bool {
  let has_upgrade = req
    .headers()
    .get(header::UPGRADE)
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_ascii_lowercase())
    .map(|value| value.contains("websocket"))
    .unwrap_or(false);

  let has_connection = req
    .headers()
    .get(header::CONNECTION)
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_ascii_lowercase())
    .map(|value| value.contains("upgrade"))
    .unwrap_or(false);

  has_upgrade && has_connection
}

fn build_target_uri(target_url: &str, req: &Request) -> Result<Uri, TauriError> {
  let mut base = target_url.to_string();
  if base.ends_with('/') {
    base.pop();
  }

  let path = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
  let full = format!("{}{}", base, path);
  full.parse::<Uri>().map_err(|err| TauriError::new(format!("Invalid proxy target URI: {}", err)))
}

async fn handle_websocket_proxy(req: Request, state: &ProxyState) -> Result<Response, TauriError> {
  let ws_request = build_ws_request(&req, &state.config.target_url)?;
  let response = tokio_tungstenite::tungstenite::handshake::server::create_response(&ws_request)
    .map_err(|err| TauriError::new(format!("WS handshake failed: {}", err)))?;

  let on_upgrade = hyper::upgrade::on(req);

  tokio::spawn(async move {
    let upgraded = match on_upgrade.await {
      Ok(upgraded) => upgraded,
      Err(err) => {
        tracing::warn!("WS upgrade failed: {}", err);
        return;
      }
    };

    if let Err(err) = proxy_websocket_connection(upgraded, ws_request).await {
      tracing::warn!("WS proxy error: {}", err);
    }
  });

  let (parts, _) = response.into_parts();
  Ok(Response::from_parts(parts, Body::empty()))
}

fn build_ws_request(
  req: &Request,
  target_url: &str,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>, TauriError> {
  let target_uri = build_target_uri(target_url, req)?;
  let target_url =
    Url::parse(&target_uri.to_string()).map_err(|err| TauriError::new(format!("Invalid WS target URL: {}", err)))?;
  let ws_url = match target_url.scheme() {
    "https" => target_url.as_str().replace("https://", "wss://"),
    "http" => target_url.as_str().replace("http://", "ws://"),
    _ => target_url.as_str().to_string(),
  };
  let mut ws_request =
    ws_url.into_client_request().map_err(|err| TauriError::new(format!("Failed to build WS request: {}", err)))?;

  for (key, value) in req.headers().iter() {
    ws_request.headers_mut().insert(key, value.clone());
  }

  if let Some(authority) = target_uri.authority()
    && let Ok(host) = HeaderValue::from_str(authority.as_str())
  {
    ws_request.headers_mut().insert(header::HOST, host);
  }

  Ok(ws_request)
}

async fn proxy_websocket_connection(
  upgraded: hyper::upgrade::Upgraded,
  ws_request: tokio_tungstenite::tungstenite::http::Request<()>,
) -> Result<(), TauriError> {
  let server_socket = tokio_tungstenite::WebSocketStream::from_raw_socket(
    TokioIo::new(upgraded),
    tokio_tungstenite::tungstenite::protocol::Role::Server,
    None,
  )
  .await;
  let (client_socket, _) = tokio_tungstenite::connect_async(ws_request)
    .await
    .map_err(|err| TauriError::new(format!("Failed to connect to WS target: {}", err)))?;

  let (mut server_tx, mut server_rx) = server_socket.split();
  let (mut client_tx, mut client_rx) = client_socket.split();

  let server_to_client = async {
    while let Some(msg) = server_rx.next().await {
      let msg = msg.map_err(|err| TauriError::new(format!("WS read error: {}", err)))?;
      client_tx.send(msg).await.map_err(|err| TauriError::new(format!("WS send error: {}", err)))?;
    }
    let _ = client_tx.close().await;
    Ok::<_, TauriError>(())
  };

  let client_to_server = async {
    while let Some(msg) = client_rx.next().await {
      let msg = msg.map_err(|err| TauriError::new(format!("WS read error: {}", err)))?;
      server_tx.send(msg).await.map_err(|err| TauriError::new(format!("WS send error: {}", err)))?;
    }
    let _ = server_tx.close().await;
    Ok::<_, TauriError>(())
  };

  tokio::try_join!(server_to_client, client_to_server)?;

  Ok(())
}

pub fn static_router(static_path: &str) -> Router {
  let serve_dir = ServeDir::new(static_path).append_index_html_on_directories(true);
  let fallback_file = format!("{}/index.html", static_path);
  let serve_dir = serve_dir.fallback(tower_http::services::ServeFile::new(fallback_file));
  Router::new().fallback_service(serve_dir)
}

async fn instrumentation_script_handler(
  axum::extract::State(config): axum::extract::State<PreviewInstrumentationConfig>,
) -> Result<Response, TauriError> {
  if !config.enabled {
    return Ok((StatusCode::NOT_FOUND, "not found").into_response());
  }

  let body = instrumentation_script();
  let mut response = Response::new(Body::from(body));
  *response.status_mut() = StatusCode::OK;
  response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/javascript"));
  Ok(response)
}

async fn instrumentation_event_handler(
  ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
  axum::extract::State(config): axum::extract::State<PreviewInstrumentationConfig>,
  req: Request,
) -> Result<Response, TauriError> {
  if !config.enabled {
    return Ok((StatusCode::NOT_FOUND, "not found").into_response());
  }

  if !is_allowed_host(&remote_addr, &["127.0.0.1".to_string(), "localhost".to_string(), "::1".to_string()]) {
    return Ok((StatusCode::FORBIDDEN, "forbidden").into_response());
  }

  let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
    .await
    .map_err(|err| TauriError::new(format!("Failed to read instrumentation event body: {}", err)))?;
  let event = serde_json::from_slice::<PreviewInstrumentationEventRequest>(&bytes)
    .map_err(|err| TauriError::new(err.to_string()))?;

  emit_instrumentation_event(&config, event);

  Ok((StatusCode::OK, "ok").into_response())
}
