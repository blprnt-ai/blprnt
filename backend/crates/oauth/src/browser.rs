use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use shared::errors::OauthError;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::Duration;
use url::Url;

type SenderSlot = Arc<Mutex<Option<oneshot::Sender<HashMap<String, String>>>>>;

pub fn build_authorize_url(
  base_authorize_url: &str,
  redirect_uri: &str,
  additional_params: &[(impl AsRef<str>, impl AsRef<str>)],
) -> Result<String> {
  if base_authorize_url.contains("auth.openai.com") {
    let mut authorize_url = String::from(base_authorize_url);
    if !authorize_url.contains('?') {
      authorize_url.push('?');
    } else {
      authorize_url.push('&');
    }

    let mut map = std::collections::HashMap::new();
    for (k, v) in additional_params {
      map.insert(k.as_ref().to_string(), v.as_ref().to_string());
    }

    let ruri = redirect_uri.to_string();
    let query = [
      ("response_type", map.get("response_type").cloned()),
      ("client_id", map.get("client_id").cloned()),
      ("redirect_uri", Some(ruri)),
      ("scope", map.get("scope").cloned()),
      ("code_challenge", map.get("code_challenge").cloned()),
      ("code_challenge_method", map.get("code_challenge_method").cloned()),
      ("id_token_add_organizations", map.get("id_token_add_organizations").cloned()),
      ("codex_cli_simplified_flow", map.get("codex_cli_simplified_flow").cloned()),
      ("state", map.get("state").cloned()),
      ("originator", map.get("originator").cloned()),
    ];

    let query_params = query
      .into_iter()
      .filter_map(|(k, v)| v.map(|v| format!("{}={}", k, urlencoding::encode(&v))))
      .collect::<Vec<_>>()
      .join("&");

    return Ok(format!("{}{}", authorize_url, query_params));
  }

  let mut url = Url::parse(base_authorize_url).map_err(|e| OauthError::InvalidBaseAuthorizeUrl(e.to_string()))?;
  {
    let mut qp = url.query_pairs_mut();
    qp.append_pair("redirect_uri", redirect_uri);
    for (k, v) in additional_params {
      qp.append_pair(k.as_ref(), v.as_ref());
    }
  }

  Ok(url.into())
}

pub fn build_authorize_url_with_param(
  base_authorize_url: &str,
  redirect_param_name: &str,
  redirect_uri: &str,
  additional_params: &[(impl AsRef<str>, impl AsRef<str>)],
) -> Result<String> {
  let mut url = Url::parse(base_authorize_url).map_err(|e| OauthError::InvalidBaseAuthorizeUrl(e.to_string()))?;
  {
    let mut qp = url.query_pairs_mut();
    qp.append_pair(redirect_param_name, redirect_uri);
    for (k, v) in additional_params {
      qp.append_pair(k.as_ref(), v.as_ref());
    }
  }

  Ok(url.into())
}

pub fn open_in_browser(url: &str) -> Result<()> {
  webbrowser::open(url).map_err(|e| OauthError::FailedToOpenBrowser(e.to_string()))?;
  Ok(())
}

pub struct CallbackResult {
  pub params:       HashMap<String, String>,
  pub redirect_uri: String,
}

pub async fn run_local_browser_flow(
  base_authorize_url: &str,
  redirect_path: &str,
  additional_params: &[(impl AsRef<str>, impl AsRef<str>)],
  success_html: Option<&str>,
  timeout_secs: u64,
) -> Result<Option<CallbackResult>> {
  let redirect_path = normalize_path(redirect_path);

  let fixed_openai = base_authorize_url.contains("auth.openai.com");
  let bind_port = if fixed_openai { 1455 } else { 0 };
  let listener = match TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), bind_port)).await {
    Ok(l) => l,
    Err(e) => {
      if fixed_openai {
        tokio::net::TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
          .await
          .map_err(|e| OauthError::FailedToBindFixedPort(e.to_string()))?
      } else {
        return Err(OauthError::FailedToBindLocalCallbackListener(e.to_string()).into());
      }
    }
  };

  let addr = listener.local_addr().map_err(|e| OauthError::FailedToGetLocalAddress(e.to_string()))?;
  let redirect_uri = format!("http://localhost:{}{}", addr.port(), redirect_path);

  let (params_tx, params_rx) = oneshot::channel::<HashMap<String, String>>();
  let shared_tx = Arc::new(Mutex::new(Some(params_tx)));
  let (shutdown_server_tx, shutdown_server_rx) = oneshot::channel::<()>();

  let app = Router::new()
    .route(&redirect_path, get(callback_handler))
    .with_state(HandlerState { tx: shared_tx.clone(), success_html: success_html.map(str::to_owned) });

  let server_task = tokio::spawn(async move {
    axum::serve(listener, app).await.ok();
  });

  let auth_url = build_authorize_url(base_authorize_url, &redirect_uri, additional_params)?;

  open_in_browser(&auth_url)?;

  tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(timeout_secs)).await;
    let _ = shutdown_server_tx.send(());
  });

  tokio::select! {
    _ = shutdown_server_rx => {

      server_task.abort();
      let _ = server_task.await;

      Ok(None)
    }
    Ok(params) = params_rx => {

      server_task.abort();
      let _ = server_task.await;

      Ok(Some(CallbackResult { params, redirect_uri }))
    }
  }
}

#[derive(Clone, Debug)]
struct HandlerState {
  tx:           SenderSlot,
  success_html: Option<String>,
}

async fn callback_handler(
  State(state): State<HandlerState>,
  Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
  if let Ok(mut lock) = state.tx.lock()
    && let Some(tx) = lock.take()
  {
    let _ = tx.send(params.clone());
  }

  let body = state.success_html.unwrap_or_else(|| default_success_html().to_string());
  Html(body)
}

fn default_success_html() -> &'static str {
  r#"
<html>
<style>
  body {
    font-family: Arial, sans-serif;
    font-size: 16px;
    line-height: 1.5;
    height: 100vh;
    width: 100vw;
  }

  .container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    width: 100vw;
  }

  .content {
    display: flex;
    background-color: oklch(0.21 0.034 264);
    flex-direction: column;
    align-items: center;
    border: 1px solid #0f92f7;
    justify-content: center;
    border-radius: 10px;
    padding: 1rem;
    transition: all 0.3s ease-in-out;
    transform-origin: center;
    transform: scale(0.98);
    will-change: transform, background-color, box-shadow;
  }

  .content:hover {
    background-color: oklch(0.19 0.034 264);
    transform: scale(1);
    box-shadow: 0 0 10px oklch(0.19 0.034 264);
  }

  .bg-grid {
    --tw-bg-opacity: 0.3;
    --grid-line: 15 146 247;
    /* 0f92f7 */

    background: radial-gradient(circle at center,
        rgb(0 0 0 / 0) 0%,
        #101828 90%) center/auto no-repeat,
      linear-gradient(to right,
        rgb(var(--grid-line) / var(--tw-bg-opacity)),
        transparent 1px) calc(50% - 20px) calc(50% - 20px) / 40px 40px,
      linear-gradient(to bottom,
        rgb(var(--grid-line) / var(--tw-bg-opacity)),
        transparent 1px) calc(50% - 20px) calc(50% - 20px) / 40px 40px;
  }

  h3 {
    color: #0f92f7;
    font-size: 2rem;
    font-weight: 700;
    margin: 0;
  }

  p {
    font-size: 1rem;
    font-weight: 400;
    color: #666;
  }
</style>

<body class="bg-grid">
  <div class="container">
    <div class="content">
      <h3>Authentication complete.</h3>
      <p>You may close this window.</p>
    </div>
  </div>
</body>

</html>"#
}

fn normalize_path(p: &str) -> String {
  if p.is_empty() {
    "/".to_string()
  } else if p.starts_with('/') {
    p.to_string()
  } else {
    format!("/{}", p)
  }
}
