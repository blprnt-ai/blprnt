use std::collections::HashMap;
use std::future::Future;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use common::errors::TauriError;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use persistence::prelude::ProjectRecord;
use persistence::prelude::ProjectRepositoryV2;
use persistence::prelude::SurrealId;
use surrealdb::types::Uuid;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use url::Url;

use crate::preview::detect::DetectedDevServer;
use crate::preview::detect::DetectedFramework;
use crate::preview::detect::DetectedLanguage;
use crate::preview::detect::DevCommand;
use crate::preview::detect::detect_dev_server;
use crate::preview::instrumentation::PreviewInstrumentationConfig;
use crate::preview::process::ProcessManager;
use crate::preview::process::SpawnedProcess;
use crate::preview::proxy::ProxyConfig;
use crate::preview::proxy::ProxyState;
use crate::preview::proxy::default_allowed_hosts;
use crate::preview::proxy::proxy_router;
use crate::preview::static_server::static_app;
use crate::preview::types::PreviewDetectedServer;
use crate::preview::types::PreviewMode;
use crate::preview::types::PreviewServerAction;
use crate::preview::types::PreviewSession;
use crate::preview::types::PreviewSessionStatus;
use crate::preview::types::PreviewStartParams;
use crate::preview::types::PreviewStatusResponse;

const DEFAULT_PROXY_BIND: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

type HealthCheckFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
type HealthCheckFn = Arc<dyn Fn(String) -> HealthCheckFuture + Send + Sync>;
type PathExistsFn = Arc<dyn Fn(&Path) -> bool + Send + Sync>;

#[derive(Clone)]
pub struct PreviewManager {
  inner:        Arc<Mutex<PreviewManagerState>>,
  processes:    Arc<Mutex<ProcessManager>>,
  health_check: HealthCheckFn,
  path_exists:  PathExistsFn,
}

impl Default for PreviewManager {
  fn default() -> Self {
    Self::new()
  }
}

impl PreviewManager {
  pub fn new() -> Self {
    Self {
      inner:        Arc::new(Mutex::new(PreviewManagerState::new())),
      processes:    Arc::new(Mutex::new(ProcessManager::new())),
      health_check: Arc::new(|url| Box::pin(async move { ensure_server_reachable(&url).await })),
      path_exists:  Arc::new(Path::exists),
    }
  }

  pub fn with_process_manager(processes: Arc<Mutex<ProcessManager>>, health_check: HealthCheckFn) -> Self {
    Self {
      inner: Arc::new(Mutex::new(PreviewManagerState::new())),
      processes,
      health_check,
      path_exists: Arc::new(Path::exists),
    }
  }

  pub fn with_dependencies(
    processes: Arc<Mutex<ProcessManager>>,
    health_check: HealthCheckFn,
    path_exists: PathExistsFn,
  ) -> Self {
    Self { inner: Arc::new(Mutex::new(PreviewManagerState::new())), processes, health_check, path_exists }
  }

  pub async fn start_preview(&self, params: PreviewStartParams) -> Result<PreviewSession> {
    let project_id: SurrealId = params.project_id.clone().try_into()?;
    let project = ProjectRepositoryV2::get(project_id).await?;

    let base_partition_id = Uuid::new_v4().to_string();
    let session_id = Uuid::new_v4().to_string();
    let created_at = Utc::now();

    let allowed_hosts = params.allowed_hosts.clone().unwrap_or_else(default_allowed_hosts);
    let instrumentation_enabled = params.instrumentation_enabled;

    let existing = {
      let mut state = self.inner.lock().await;
      state.last_status.remove(&params.project_id);
      state.sessions.remove(&params.project_id)
    };

    if let Some(existing) = existing {
      let mut processes = self.processes.lock().await;
      existing.shutdown(&mut processes);
    }

    let preview_state = match params.mode {
      PreviewMode::Dev => {
        let resolved = resolve_dev_server_url(
          &project,
          params.dev_server_url.as_deref(),
          &session_id,
          self.processes.clone(),
          self.health_check.clone(),
        )
        .await?;
        let (server_action, was_auto_started) = if resolved.process_key.is_some() {
          (Some(PreviewServerAction::Started), Some(true))
        } else {
          (Some(PreviewServerAction::Attached), Some(false))
        };
        let instrumentation = build_instrumentation_config(instrumentation_enabled, &session_id, &params.project_id);
        let (listen_addr, server) =
          start_dev_proxy(resolved.url.clone(), allowed_hosts, params.proxy_port, instrumentation.clone()).await?;
        PreviewRuntimeState {
          mode: PreviewMode::Dev,
          proxy_address: Some(listen_addr),
          static_address: None,
          instrumentation_enabled: instrumentation.enabled,
          status: PreviewSessionStatus::Ready,
          last_error: None,
          join_handle: Some(server),
          shutdown: None,
          detected_server: resolved.detected,
          process_key: resolved.process_key,
          server_action,
          was_auto_started,
        }
      }
      PreviewMode::Static => {
        let static_path = resolve_static_path(&project, params.static_path.as_deref(), &self.path_exists)?;
        let instrumentation = build_instrumentation_config(instrumentation_enabled, &session_id, &params.project_id);
        let (listen_addr, server, shutdown) =
          start_static_server(static_path.clone(), params.static_port, instrumentation.clone()).await?;
        PreviewRuntimeState {
          mode:                    PreviewMode::Static,
          proxy_address:           None,
          static_address:          Some(listen_addr),
          instrumentation_enabled: instrumentation.enabled,
          status:                  PreviewSessionStatus::Ready,
          last_error:              None,
          join_handle:             Some(server),
          shutdown:                Some(shutdown),
          detected_server:         None,
          process_key:             None,
          server_action:           None,
          was_auto_started:        None,
        }
      }
    };

    let url = build_session_url(&preview_state)?;

    let mut state = self.inner.lock().await;

    let session = PreviewSession {
      id: session_id,
      project_id: params.project_id.clone(),
      mode: preview_state.mode.clone(),
      status: preview_state.status.clone(),
      partition_id: base_partition_id,
      url,
      created_at,
    };

    state
      .sessions
      .insert(params.project_id.clone(), PreviewSessionEntry { session: session.clone(), runtime: preview_state });

    Ok(session)
  }

  pub async fn stop_preview(&self, project_id: String) -> Result<()> {
    let mut state = self.inner.lock().await;
    if let Some(entry) = state.sessions.remove(&project_id) {
      let stopped_status = build_stopped_status(&entry);
      state.last_status.insert(project_id.clone(), stopped_status);
      let mut processes = self.processes.lock().await;
      entry.shutdown(&mut processes);
    }
    Ok(())
  }

  pub async fn reload_preview(&self, project_id: String) -> Result<PreviewSession> {
    let mut state = self.inner.lock().await;
    let entry = state.sessions.get_mut(&project_id).ok_or_else(|| anyhow::anyhow!("Preview session not found"))?;

    let new_partition = Uuid::new_v4().to_string();
    entry.session.partition_id = new_partition;

    entry.session.created_at = Utc::now();
    entry.session.status = PreviewSessionStatus::Ready;

    if entry.runtime.instrumentation_enabled {
      // TODO: hook frontend instrumentation reload if enabled
    }

    Ok(entry.session.clone())
  }

  pub async fn status(&self, project_id: String) -> Result<PreviewStatusResponse> {
    let state = self.inner.lock().await;
    let Some(entry) = state.sessions.get(&project_id) else {
      if let Some(previous) = state.last_status.get(&project_id) {
        return Ok(previous.clone());
      }
      return Ok(PreviewStatusResponse {
        status:           PreviewSessionStatus::Stopped,
        last_error:       None,
        server_action:    None,
        detected:         None,
        url:              None,
        was_auto_started: None,
      });
    };

    Ok(build_status_from_entry(entry))
  }

  pub async fn get_session(&self, project_id: String) -> Result<Option<PreviewSession>> {
    let state = self.inner.lock().await;
    Ok(state.sessions.get(&project_id).map(|entry| entry.session.clone()))
  }
}

struct PreviewManagerState {
  sessions:    HashMap<String, PreviewSessionEntry>,
  last_status: HashMap<String, PreviewStatusResponse>,
}

impl PreviewManagerState {
  fn new() -> Self {
    Self { sessions: HashMap::new(), last_status: HashMap::new() }
  }
}

struct PreviewSessionEntry {
  session: PreviewSession,
  runtime: PreviewRuntimeState,
}

impl PreviewSessionEntry {
  fn shutdown(self, processes: &mut ProcessManager) {
    if let Some(shutdown) = self.runtime.shutdown {
      let _ = shutdown.send(());
    }
    if let Some(handle) = self.runtime.join_handle {
      handle.abort();
    }
    if let Some(process_key) = self.runtime.process_key {
      processes.stop(&process_key);
    }
  }
}

struct PreviewRuntimeState {
  mode:                    PreviewMode,
  proxy_address:           Option<SocketAddr>,
  static_address:          Option<SocketAddr>,
  instrumentation_enabled: bool,
  status:                  PreviewSessionStatus,
  last_error:              Option<TauriError>,
  join_handle:             Option<JoinHandle<()>>,
  shutdown:                Option<oneshot::Sender<()>>,
  detected_server:         Option<DetectedDevServer>,
  process_key:             Option<String>,
  server_action:           Option<PreviewServerAction>,
  was_auto_started:        Option<bool>,
}

async fn start_dev_proxy(
  target_url: String,
  allowed_hosts: Vec<String>,
  proxy_port: Option<u16>,
  instrumentation: PreviewInstrumentationConfig,
) -> Result<(SocketAddr, JoinHandle<()>)> {
  let listener = TcpListener::bind(SocketAddr::new(DEFAULT_PROXY_BIND, proxy_port.unwrap_or(0))).await?;
  let listen_addr = listener.local_addr()?;

  let client: Client<hyper_util::client::legacy::connect::HttpConnector, axum::body::Body> =
    Client::builder(TokioExecutor::new()).build_http();

  let state = ProxyState { config: ProxyConfig { target_url, allowed_hosts, instrumentation }, client };
  let app = proxy_router(state);

  let server = tokio::spawn(async move {
    let _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await;
  });

  Ok((listen_addr, server))
}

async fn start_static_server(
  static_path: PathBuf,
  port: Option<u16>,
  instrumentation: PreviewInstrumentationConfig,
) -> Result<(SocketAddr, JoinHandle<()>, oneshot::Sender<()>)> {
  let bind_port = port.unwrap_or(0);
  let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), bind_port)).await?;
  let listen_addr = listener.local_addr()?;

  let app = static_app(static_path.to_string_lossy().as_ref(), instrumentation)
    .into_make_service_with_connect_info::<SocketAddr>();
  let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

  let server = tokio::spawn(async move {
    let _ = axum::serve(listener, app)
      .with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
      })
      .await;
  });

  Ok((listen_addr, server, shutdown_tx))
}

fn build_session_url(state: &PreviewRuntimeState) -> Result<String> {
  let address =
    state.proxy_address.or(state.static_address).ok_or_else(|| anyhow::anyhow!("Preview server not running"))?;

  Ok(format!("http://{}:{}", address.ip(), address.port()))
}

fn build_status_from_entry(entry: &PreviewSessionEntry) -> PreviewStatusResponse {
  PreviewStatusResponse {
    status:           entry.runtime.status.clone(),
    last_error:       entry.runtime.last_error.clone(),
    server_action:    entry.runtime.server_action.clone(),
    detected:         entry.runtime.detected_server.as_ref().map(map_detected_server),
    url:              Some(entry.session.url.clone()),
    was_auto_started: entry.runtime.was_auto_started,
  }
}

fn build_stopped_status(entry: &PreviewSessionEntry) -> PreviewStatusResponse {
  PreviewStatusResponse {
    status:           PreviewSessionStatus::Stopped,
    last_error:       entry.runtime.last_error.clone(),
    server_action:    entry.runtime.server_action.clone(),
    detected:         entry.runtime.detected_server.as_ref().map(map_detected_server),
    url:              Some(entry.session.url.clone()),
    was_auto_started: entry.runtime.was_auto_started,
  }
}

fn map_detected_server(detected: &DetectedDevServer) -> PreviewDetectedServer {
  PreviewDetectedServer {
    language:         Some(map_language(&detected.language)),
    framework:        detected.framework.as_ref().map(map_framework),
    suggested_port:   detected.port,
    detected_command: detected.command.as_ref().map(format_command),
  }
}

fn map_language(language: &DetectedLanguage) -> String {
  match language {
    DetectedLanguage::JavaScript => "JavaScript".to_string(),
    DetectedLanguage::Python => "Python".to_string(),
  }
}

fn map_framework(framework: &DetectedFramework) -> String {
  match framework {
    DetectedFramework::Js(value) => match value {
      crate::preview::detect::JsFramework::Vite => "Vite".to_string(),
      crate::preview::detect::JsFramework::Next => "Next".to_string(),
      crate::preview::detect::JsFramework::Nuxt => "Nuxt".to_string(),
      crate::preview::detect::JsFramework::React => "React".to_string(),
      crate::preview::detect::JsFramework::Vue => "Vue".to_string(),
      crate::preview::detect::JsFramework::Angular => "Angular".to_string(),
      crate::preview::detect::JsFramework::Svelte => "Svelte".to_string(),
    },
    DetectedFramework::Python(value) => match value {
      crate::preview::detect::PythonFramework::Django => "Django".to_string(),
      crate::preview::detect::PythonFramework::Flask => "Flask".to_string(),
      crate::preview::detect::PythonFramework::FastApi => "FastAPI".to_string(),
    },
  }
}

fn format_command(command: &DevCommand) -> String {
  if command.args.is_empty() {
    return command.program.clone();
  }
  format!("{} {}", command.program, command.args.join(" "))
}

struct ResolvedDevServer {
  url:         String,
  detected:    Option<DetectedDevServer>,
  process_key: Option<String>,
}

async fn resolve_dev_server_url(
  project: &ProjectRecord,
  override_url: Option<&str>,
  session_id: &str,
  processes: Arc<Mutex<ProcessManager>>,
  health_check: HealthCheckFn,
) -> Result<ResolvedDevServer> {
  if let Some(override_url) = override_url {
    let url = normalize_url(override_url)?;
    (health_check)(url.clone()).await?;
    return Ok(ResolvedDevServer { url, detected: None, process_key: None });
  }

  let working_dir =
    project.working_directories().0.first().cloned().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
  let detected = detect_dev_server(&working_dir);

  if let Some(detected) = detected {
    let port = detected.port.unwrap_or(default_port_for_language(&detected.language));
    let url = format!("http://localhost:{}", port);
    let normalized = normalize_url(&url)?;

    if (health_check)(normalized.clone()).await.is_ok() {
      return Ok(ResolvedDevServer { url: normalized, detected: Some(detected), process_key: None });
    }

    if let Some(command) = detected.command.clone() {
      let process_key = format!("preview:{}:{}", project.id.key(), session_id);
      let spawned = {
        let mut processes = processes.lock().await;
        processes.spawn(process_key.clone(), working_dir.clone(), command)?
      };
      let resolved_url = match wait_for_ready(normalized, spawned, health_check.clone()).await {
        Ok(url) => url,
        Err(err) => {
          let mut processes = processes.lock().await;
          processes.stop(&process_key);
          return Err(err);
        }
      };
      return Ok(ResolvedDevServer {
        url:         resolved_url,
        detected:    Some(detected),
        process_key: Some(process_key),
      });
    }

    return Err(anyhow::anyhow!("Detected dev server is not reachable"));
  }

  let fallback = "http://localhost:5173".to_string();
  let normalized = normalize_url(&fallback)?;
  if (health_check)(normalized.clone()).await.is_ok() {
    return Ok(ResolvedDevServer { url: normalized, detected: None, process_key: None });
  }

  Err(anyhow::anyhow!("No dev server detected and default port is unreachable"))
}

async fn ensure_server_reachable(url: &str) -> Result<()> {
  let client: Client<hyper_util::client::legacy::connect::HttpConnector, axum::body::Body> =
    Client::builder(TokioExecutor::new()).build_http();
  let uri = url.parse::<hyper::Uri>()?;

  let deadline = Instant::now() + Duration::from_secs(10);

  loop {
    let response = client.get(uri.clone()).await;
    if let Ok(response) = response
      && !response.status().is_server_error()
    {
      return Ok(());
    }

    if Instant::now() >= deadline {
      break;
    }

    sleep(Duration::from_millis(250)).await;
  }

  Err(anyhow::anyhow!("Preview server not reachable"))
}

async fn wait_for_ready(
  initial_url: String,
  mut spawned: SpawnedProcess,
  health_check: HealthCheckFn,
) -> Result<String> {
  let deadline = Instant::now() + Duration::from_secs(15);
  let mut candidate = initial_url;

  loop {
    if (health_check)(candidate.clone()).await.is_ok() {
      return Ok(candidate);
    }

    let now = Instant::now();
    if now >= deadline {
      break;
    }

    let remaining = deadline.saturating_duration_since(now);
    if let Ok(Some(url)) =
      tokio::time::timeout(remaining.min(Duration::from_millis(500)), spawned.url_receiver.recv()).await
    {
      candidate = url;
      continue;
    }

    sleep(Duration::from_millis(200)).await;
  }

  Err(anyhow::anyhow!("Preview server failed to start"))
}

fn default_port_for_language(language: &crate::preview::detect::DetectedLanguage) -> u16 {
  match language {
    crate::preview::detect::DetectedLanguage::JavaScript => 5173,
    crate::preview::detect::DetectedLanguage::Python => 8000,
  }
}

fn resolve_static_path(
  project: &ProjectRecord,
  override_path: Option<&str>,
  path_exists: &PathExistsFn,
) -> Result<PathBuf> {
  if let Some(override_path) = override_path {
    return Ok(PathBuf::from(override_path));
  }

  let working_dir =
    project.working_directories().0.first().cloned().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
  let candidates = [working_dir.join("dist"), working_dir.join("build"), working_dir.join("out")];

  for path in candidates {
    if (path_exists)(&path) {
      return Ok(path);
    }
  }

  Ok(working_dir.join("dist"))
}

fn normalize_url(value: &str) -> Result<String> {
  let value = if value.starts_with("http://") || value.starts_with("https://") {
    value.to_string()
  } else {
    format!("http://{}", value)
  };
  let mut url = Url::parse(&value).map_err(|err| anyhow::anyhow!("Invalid URL: {err}"))?;
  url.set_fragment(None);
  url.set_query(None);
  Ok(url.to_string().trim_end_matches('/').to_string())
}

fn build_instrumentation_config(enabled: bool, session_id: &str, project_id: &str) -> PreviewInstrumentationConfig {
  if enabled {
    PreviewInstrumentationConfig::enabled(session_id.to_string(), project_id.to_string())
  } else {
    PreviewInstrumentationConfig {
      enabled:    false,
      session_id: session_id.to_string(),
      project_id: project_id.to_string(),
    }
  }
}
