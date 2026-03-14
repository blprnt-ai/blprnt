use std::borrow::Cow;
use std::collections::HashMap;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use common::blprnt::Blprnt;
use common::blprnt::BlprntEventKind;
use common::shared::prelude::McpAuthConfig;
use common::shared::prelude::McpRuntimeBridge;
use common::shared::prelude::McpServerConfig;
use common::shared::prelude::McpServerLifecycleState;
use common::shared::prelude::McpServerStatus;
use common::shared::prelude::McpSseHttpTransportConfig;
use common::shared::prelude::McpStdioTransportConfig;
use common::shared::prelude::McpToolDescriptor;
use common::shared::prelude::McpTransportConfig;
use http::HeaderName as HttpHeaderName;
use http::HeaderValue as HttpHeaderValue;
use rmcp::Peer;
use rmcp::RoleClient;
use rmcp::model::CallToolRequestParams;
use rmcp::model::ClientCapabilities;
use rmcp::model::ClientInfo;
use rmcp::model::Implementation;
use rmcp::model::InitializeResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ProtocolVersion;
use rmcp::transport::ConfigureCommandExt;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::TokioChildProcess;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

const MCP_RECONNECT_MAX_RETRIES: u32 = 5;
const MCP_RECONNECT_MAX_BACKOFF_SECS: u64 = 30;
const MCP_HTTP_HEALTHCHECK_INTERVAL_SECS: u64 = 30;
const MCP_STDIO_READY_TIMEOUT_MS: u64 = 10_000;
const MCP_HTTP_CONNECT_TIMEOUT_SECS: u64 = 20;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn resolve_stdio_command(command: &str, env: Option<&std::collections::BTreeMap<String, String>>) -> String {
  #[cfg(windows)]
  {
    resolve_windows_stdio_command(command, env)
  }

  #[cfg(not(windows))]
  {
    let _ = env;
    command.to_string()
  }
}

#[cfg(windows)]
fn resolve_windows_stdio_command(command: &str, env: Option<&std::collections::BTreeMap<String, String>>) -> String {
  let command = command.trim();
  if command.is_empty() {
    return command.to_string();
  }

  let path = Path::new(command);

  // If the user already provided an extension (e.g. npx.cmd), don't second guess.
  if path.extension().is_some() {
    return command.to_string();
  }

  // If the user provided a path-like value (absolute/relative), prefer a sibling .cmd/.bat when present.
  // This avoids Node's installer shims like "...\\nodejs\\npx" (bash script) when "...\\nodejs\\npx.cmd" exists.
  let is_path_like = command.contains('\\') || command.contains('/') || command.contains(':');
  if is_path_like {
    let cmd_variant = path.with_extension("cmd");
    if cmd_variant.is_file() {
      return cmd_variant.to_string_lossy().into_owned();
    }
    let bat_variant = path.with_extension("bat");
    if bat_variant.is_file() {
      return bat_variant.to_string_lossy().into_owned();
    }
    return command.to_string();
  }

  let effective_path = env
    .and_then(|m| m.iter().find(|(k, _)| k.as_str().eq_ignore_ascii_case("path")).map(|(_, v)| v.as_str()))
    .and_then(|v| if v.trim().is_empty() { None } else { Some(v) })
    .map(std::ffi::OsString::from)
    .or_else(|| std::env::var_os("PATH"));

  let Some(effective_path) = effective_path else {
    return command.to_string();
  };

  // Prefer real executables first; fall back to cmd/bat shims. Avoid selecting extensionless files,
  // which can be non-executable scripts on Windows (e.g. Node's "npx" bash shim).
  for dir in std::env::split_paths(&effective_path) {
    for ext in ["exe", "cmd", "bat", "com"] {
      let candidate = dir.join(format!("{command}.{ext}"));
      if candidate.is_file() {
        return candidate.to_string_lossy().into_owned();
      }
    }
  }

  command.to_string()
}

fn format_runtime_error(error: &anyhow::Error, attempts: u32) -> String {
  let next_retry = attempts < MCP_RECONNECT_MAX_RETRIES;
  if next_retry {
    format!("Connection issue: {}. Retrying ({}/{})...", error, attempts, MCP_RECONNECT_MAX_RETRIES)
  } else {
    format!("Connection failed after {} attempts: {}. Verify transport/auth settings, then reconnect.", attempts, error)
  }
}

#[derive(Clone)]
struct RmcpClientHandle {
  peer: Peer<RoleClient>,
}

struct WorkerHandle {
  shutdown:   watch::Sender<bool>,
  generation: u64,
}

pub struct McpRuntimeManager {
  statuses:    RwLock<HashMap<String, McpServerStatus>>,
  workers:     Mutex<HashMap<String, WorkerHandle>>,
  clients:     RwLock<HashMap<String, Arc<RmcpClientHandle>>>,
  tools_cache: RwLock<HashMap<String, Vec<McpToolDescriptor>>>,
}

impl Default for McpRuntimeManager {
  fn default() -> Self {
    Self::new()
  }
}

impl McpRuntimeManager {
  pub fn new() -> Self {
    Self {
      statuses:    RwLock::new(HashMap::new()),
      workers:     Mutex::new(HashMap::new()),
      clients:     RwLock::new(HashMap::new()),
      tools_cache: RwLock::new(HashMap::new()),
    }
  }

  pub async fn statuses(&self) -> Vec<McpServerStatus> {
    let statuses = self.statuses.read().await;
    let mut list = statuses.values().cloned().collect::<Vec<_>>();
    list.sort_by(|left, right| left.server_id.cmp(&right.server_id));
    list
  }

  pub async fn sync_servers(self: &Arc<Self>, servers: Vec<McpServerConfig>) {
    let target_ids = servers.iter().map(|server| server.id.clone()).collect::<Vec<_>>();

    {
      let mut workers = self.workers.lock().await;
      let removed_ids =
        workers.keys().filter(|server_id| !target_ids.iter().any(|id| id == *server_id)).cloned().collect::<Vec<_>>();
      for server_id in removed_ids {
        if let Some(worker) = workers.remove(&server_id) {
          let _ = worker.shutdown.send(true);
        }
      }
    }

    {
      let mut statuses = self.statuses.write().await;
      statuses.retain(|server_id, _| target_ids.iter().any(|id| id == server_id));
    }

    {
      let mut clients = self.clients.write().await;
      clients.retain(|server_id, _| target_ids.iter().any(|id| id == server_id));
    }

    {
      let mut tools_cache = self.tools_cache.write().await;
      tools_cache.retain(|server_id, _| target_ids.iter().any(|id| id == server_id));
    }

    for server in servers {
      if !server.enabled {
        self.stop_server(&server.id).await;
        self.set_status(&server.id, McpServerLifecycleState::Configured, None).await;
        continue;
      }

      self.start_server(server).await;
    }
  }

  async fn stop_server(&self, server_id: &str) {
    let mut workers = self.workers.lock().await;
    if let Some(worker) = workers.remove(server_id) {
      let _ = worker.shutdown.send(true);
    }
    self.clients.write().await.remove(server_id);
    self.tools_cache.write().await.remove(server_id);
  }

  pub async fn list_tools(&self) -> Vec<McpToolDescriptor> {
    let cache = self.tools_cache.read().await;
    let mut tools = cache.values().flat_map(|list| list.iter().cloned()).collect::<Vec<_>>();
    tools.sort_by(|a, b| (&a.server_id, &a.name).cmp(&(&b.server_id, &b.name)));
    tools
  }

  pub async fn call_tool(
    &self,
    server_id: String,
    name: String,
    arguments: serde_json::Value,
  ) -> Result<serde_json::Value, String> {
    let client = {
      let clients = self.clients.read().await;
      clients.get(&server_id).cloned()
    }
    .ok_or_else(|| format!("MCP server {server_id} is not connected"))?;

    let arguments = match arguments {
      serde_json::Value::Null => None,
      serde_json::Value::Object(map) => Some(map),
      _ => return Err("MCP tool arguments must be a JSON object".to_string()),
    };

    let result = client
      .peer
      .call_tool(CallToolRequestParams { meta: None, name: Cow::Owned(name), arguments, task: None })
      .await
      .map_err(|error| error.to_string())?;

    serde_json::to_value(result).map_err(|error| error.to_string())
  }

  async fn start_server(self: &Arc<Self>, server: McpServerConfig) {
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    let generation = {
      let mut workers = self.workers.lock().await;
      let next_generation = workers
        .remove(&server.id)
        .map(|worker| {
          let _ = worker.shutdown.send(true);
          worker.generation.saturating_add(1)
        })
        .unwrap_or(1);
      workers.insert(server.id.clone(), WorkerHandle { shutdown: shutdown_tx, generation: next_generation });
      next_generation
    };

    self.set_status(&server.id, McpServerLifecycleState::Configured, None).await;

    let runtime = self.clone();
    tokio::spawn(async move {
      let mut attempts = 0_u32;
      let mut discovered_for_generation = false;

      loop {
        if *shutdown_rx.borrow() {
          runtime.set_status_for_worker(&server.id, generation, McpServerLifecycleState::Disconnected, None).await;
          break;
        }

        runtime.set_status_for_worker(&server.id, generation, McpServerLifecycleState::Connecting, None).await;

        let connect_result = connect_rmcp_client(&server).await;
        match connect_result {
          Ok(ConnectedClient { ct, service, peer }) => {
            // Always install the current client handle so tool calls work after reconnects.
            runtime.clients.write().await.insert(server.id.clone(), Arc::new(RmcpClientHandle { peer: peer.clone() }));

            if !discovered_for_generation {
              match discover_tools(&server.id, &peer).await {
                Ok(tools) => {
                  runtime.tools_cache.write().await.insert(server.id.clone(), tools);
                  discovered_for_generation = true;
                }
                Err(error) => {
                  runtime.clients.write().await.remove(&server.id);
                  attempts += 1;
                  if attempts > MCP_RECONNECT_MAX_RETRIES {
                    runtime
                      .set_status_for_worker(
                        &server.id,
                        generation,
                        McpServerLifecycleState::Error,
                        Some(format!("Connected but tool discovery failed: {error}")),
                      )
                      .await;
                    break;
                  }
                  runtime
                    .set_status_for_worker(
                      &server.id,
                      generation,
                      McpServerLifecycleState::Degraded,
                      Some(format!(
                        "Connected but tool discovery failed: {error}. Retrying ({}/{})...",
                        attempts, MCP_RECONNECT_MAX_RETRIES
                      )),
                    )
                    .await;
                  cancel_and_wait(ct, service).await;
                  backoff_or_shutdown(&runtime, &server.id, generation, attempts, &mut shutdown_rx).await;
                  continue;
                }
              }
            }

            attempts = 0;
            runtime.set_status_for_worker(&server.id, generation, McpServerLifecycleState::Connected, None).await;

            // Keep the service alive until shutdown or disconnect.
            let is_http = matches!(server.transport, McpTransportConfig::SseHttp(_));
            match wait_for_disconnect_or_shutdown(is_http, &peer, ct.clone(), service, &mut shutdown_rx).await {
              WaitResult::Shutdown => {
                runtime.clients.write().await.remove(&server.id);
                runtime
                  .set_status_for_worker(&server.id, generation, McpServerLifecycleState::Disconnected, None)
                  .await;
                break;
              }
              WaitResult::Disconnected(error) => {
                runtime.clients.write().await.remove(&server.id);
                attempts += 1;
                if attempts > MCP_RECONNECT_MAX_RETRIES {
                  runtime
                    .set_status_for_worker(
                      &server.id,
                      generation,
                      McpServerLifecycleState::Error,
                      Some(format_runtime_error(&error, attempts)),
                    )
                    .await;
                  break;
                }
                runtime
                  .set_status_for_worker(
                    &server.id,
                    generation,
                    McpServerLifecycleState::Degraded,
                    Some(format_runtime_error(&error, attempts)),
                  )
                  .await;
              }
            }
          }
          Err(error) => {
            runtime.clients.write().await.remove(&server.id);
            attempts += 1;
            if attempts > MCP_RECONNECT_MAX_RETRIES {
              runtime
                .set_status_for_worker(
                  &server.id,
                  generation,
                  McpServerLifecycleState::Error,
                  Some(format_runtime_error(&error, attempts)),
                )
                .await;
              break;
            }
            runtime
              .set_status_for_worker(
                &server.id,
                generation,
                McpServerLifecycleState::Degraded,
                Some(format_runtime_error(&error, attempts)),
              )
              .await;
          }
        }

        backoff_or_shutdown(&runtime, &server.id, generation, attempts, &mut shutdown_rx).await;
      }
    });
  }

  async fn set_status_for_worker(
    &self,
    server_id: &str,
    generation: u64,
    state: McpServerLifecycleState,
    error: Option<String>,
  ) {
    let is_current_generation = {
      let workers = self.workers.lock().await;
      workers.get(server_id).map(|worker| worker.generation == generation).unwrap_or(false)
    };

    if !is_current_generation {
      return;
    }

    self.set_status(server_id, state, error).await;
  }

  async fn set_status(&self, server_id: &str, state: McpServerLifecycleState, error: Option<String>) {
    let status = McpServerStatus { server_id: server_id.to_string(), state, error };

    let should_emit = {
      let mut statuses = self.statuses.write().await;
      match statuses.get(server_id) {
        Some(existing) if existing == &status => false,
        _ => {
          statuses.insert(server_id.to_string(), status.clone());
          true
        }
      }
    };

    if should_emit {
      let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        Blprnt::emit(BlprntEventKind::McpServerStatus, status.into());
      }));
    }
  }
}

#[async_trait]
impl McpRuntimeBridge for McpRuntimeManager {
  async fn list_tools(&self) -> Vec<McpToolDescriptor> {
    McpRuntimeManager::list_tools(self).await
  }

  async fn call_tool(
    &self,
    server_id: String,
    name: String,
    arguments: serde_json::Value,
  ) -> Result<serde_json::Value, String> {
    McpRuntimeManager::call_tool(self, server_id, name, arguments).await
  }

  async fn get_initialize_results(&self) -> HashMap<String, InitializeResult> {
    let mut results = HashMap::new();
    for (server_id, client) in self.clients.read().await.iter() {
      if let Some(peer_info) = client.peer.peer_info() {
        results.insert(server_id.clone(), peer_info.clone());
      }
    }
    results
  }
}

struct ConnectedClient {
  ct:      CancellationToken,
  service: rmcp::service::RunningService<RoleClient, ClientInfo>,
  peer:    Peer<RoleClient>,
}

async fn connect_rmcp_client(server: &McpServerConfig) -> anyhow::Result<ConnectedClient> {
  let client_info = ClientInfo {
    meta:             None,
    protocol_version: ProtocolVersion::V_2024_11_05,
    capabilities:     ClientCapabilities::default(),
    client_info:      Implementation {
      name:        "blprnt".to_string(),
      title:       None,
      version:     "1.0.0".to_string(),
      description: None,
      icons:       None,
      website_url: None,
    },
  };

  match &server.transport {
    McpTransportConfig::Stdio(McpStdioTransportConfig { command, args, env, cwd }) => {
      let resolved_command = resolve_stdio_command(command, env.as_ref());
      if resolved_command != *command {
        tracing::debug!("Resolved stdio command '{}' -> '{}'", command, resolved_command);
      }

      let mut cmd = tokio::process::Command::new(&resolved_command);
      cmd.args(args);
      cmd.stdin(Stdio::piped());
      cmd.stdout(Stdio::piped());
      cmd.stderr(Stdio::piped());
      if let Some(working_dir) = cwd {
        cmd.current_dir(working_dir);
      }
      if let Some(environment) = env {
        cmd.envs(environment.iter());
      }
      #[cfg(windows)]
      cmd.creation_flags(CREATE_NO_WINDOW);

      let transport = TokioChildProcess::new(cmd.configure(|_cmd| {})).map_err(|error| {
        anyhow::anyhow!("stdio spawn failed for command '{}' (resolved to '{}'): {error}", command, resolved_command)
      })?;

      let ct = CancellationToken::new();
      let service = tokio::time::timeout(
        Duration::from_millis(MCP_STDIO_READY_TIMEOUT_MS),
        rmcp::service::serve_client_with_ct(client_info, transport, ct.clone()),
      )
      .await
      .map_err(|_| anyhow::anyhow!("stdio initialize timeout"))?
      .map_err(|error| anyhow::anyhow!("stdio initialize failed: {error}"))?;

      let peer = service.peer().clone();
      Ok(ConnectedClient { ct, service, peer })
    }
    McpTransportConfig::SseHttp(McpSseHttpTransportConfig { url, headers }) => {
      let (auth_header, custom_headers) = build_http_auth_and_headers(&server.auth, headers.as_ref())?;

      let mut config =
        rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig::with_uri(url.clone());
      config.allow_stateless = true;
      config.channel_buffer_capacity = 16;
      config.custom_headers = custom_headers;
      if let Some(token) = auth_header {
        config.auth_header = Some(token);
      }

      let transport = StreamableHttpClientTransport::<reqwest::Client>::from_config(config);

      let ct = CancellationToken::new();
      let service = tokio::time::timeout(
        Duration::from_secs(MCP_HTTP_CONNECT_TIMEOUT_SECS),
        rmcp::service::serve_client_with_ct(client_info, transport, ct.clone()),
      )
      .await
      .map_err(|_| anyhow::anyhow!("HTTP initialize timeout"))?
      .map_err(|error| anyhow::anyhow!("HTTP initialize failed: {error}"))?;

      let peer = service.peer().clone();
      Ok(ConnectedClient { ct, service, peer })
    }
  }
}

fn build_http_auth_and_headers(
  auth: &McpAuthConfig,
  extra_headers: Option<&std::collections::BTreeMap<String, String>>,
) -> anyhow::Result<(Option<String>, HashMap<HttpHeaderName, HttpHeaderValue>)> {
  let mut headers = HashMap::<HttpHeaderName, HttpHeaderValue>::new();

  if let Some(raw_headers) = extra_headers {
    for (key, value) in raw_headers {
      // Filter reserved headers that rmcp's reqwest transport refuses.
      if key.eq_ignore_ascii_case("accept")
        || key.eq_ignore_ascii_case("mcp-session-id")
        || key.eq_ignore_ascii_case("mcp-protocol-version")
        || key.eq_ignore_ascii_case("last-event-id")
      {
        continue;
      }

      let name = HttpHeaderName::from_bytes(key.as_bytes()).map_err(|_| anyhow::anyhow!("invalid header name"))?;
      let value = HttpHeaderValue::from_str(value).map_err(|_| anyhow::anyhow!("invalid header value"))?;
      headers.insert(name, value);
    }
  }

  let mut bearer_for_transport: Option<String> = None;

  match auth {
    McpAuthConfig::None => {}
    McpAuthConfig::BearerToken { token } => {
      bearer_for_transport = Some(token.clone());
    }
    McpAuthConfig::ApiKey { header, key } => {
      let name =
        HttpHeaderName::from_bytes(header.as_bytes()).map_err(|_| anyhow::anyhow!("invalid api key header"))?;
      let value = HttpHeaderValue::from_str(key).map_err(|_| anyhow::anyhow!("invalid api key value"))?;
      headers.insert(name, value);
    }
    McpAuthConfig::Basic { username, password } => {
      let encoded =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, format!("{username}:{password}"));
      let value = HttpHeaderValue::from_str(&format!("Basic {encoded}"))
        .map_err(|_| anyhow::anyhow!("invalid basic auth payload"))?;
      headers.insert(HttpHeaderName::from_static("authorization"), value);
    }
    McpAuthConfig::Headers { headers: auth_headers } => {
      for (key, value) in auth_headers {
        if key.eq_ignore_ascii_case("accept")
          || key.eq_ignore_ascii_case("mcp-session-id")
          || key.eq_ignore_ascii_case("mcp-protocol-version")
          || key.eq_ignore_ascii_case("last-event-id")
        {
          continue;
        }
        let name =
          HttpHeaderName::from_bytes(key.as_bytes()).map_err(|_| anyhow::anyhow!("invalid auth header name"))?;
        let value = HttpHeaderValue::from_str(value).map_err(|_| anyhow::anyhow!("invalid auth header value"))?;
        headers.insert(name, value);
      }
    }
  }

  Ok((bearer_for_transport, headers))
}

async fn discover_tools(server_id: &str, peer: &Peer<RoleClient>) -> anyhow::Result<Vec<McpToolDescriptor>> {
  let tools = peer.list_all_tools().await.map_err(|error| anyhow::anyhow!("{error}"))?;
  let mut descriptors = Vec::with_capacity(tools.len());
  for tool in tools {
    let description = tool.description.as_ref().map(|d| d.to_string()).unwrap_or_default();
    let input_schema = serde_json::Value::Object(tool.input_schema.as_ref().clone());
    descriptors.push(McpToolDescriptor {
      server_id: server_id.to_string(),
      name: tool.name.to_string(),
      description,
      input_schema,
    });
  }

  Ok(descriptors)
}

enum WaitResult {
  Shutdown,
  Disconnected(anyhow::Error),
}

async fn wait_for_disconnect_or_shutdown(
  is_http: bool,
  peer: &Peer<RoleClient>,
  ct: CancellationToken,
  service: rmcp::service::RunningService<RoleClient, ClientInfo>,
  shutdown: &mut watch::Receiver<bool>,
) -> WaitResult {
  let mut interval = tokio::time::interval(Duration::from_secs(MCP_HTTP_HEALTHCHECK_INTERVAL_SECS));

  let wait_fut = service.waiting();
  tokio::pin!(wait_fut);

  loop {
    tokio::select! {
      _ = shutdown.changed() => {
        ct.cancel();
        let _ = (&mut wait_fut).await;
        return WaitResult::Shutdown;
      }
      res = &mut wait_fut => {
        let err = anyhow::anyhow!("MCP connection closed: {:?}", res);
        return WaitResult::Disconnected(err);
      }
      _ = interval.tick(), if is_http => {
        // Lightweight healthcheck: attempt first page of tools/list.
        let check = peer.list_tools(Some(PaginatedRequestParams { meta: None, cursor: None })).await;
        if let Err(error) = check {
          ct.cancel();
          let _ = (&mut wait_fut).await;
          return WaitResult::Disconnected(anyhow::anyhow!("HTTP healthcheck failed: {error}"));
        }
      }
    }
  }
}

async fn cancel_and_wait(ct: CancellationToken, service: rmcp::service::RunningService<RoleClient, ClientInfo>) {
  let wait_fut = service.waiting();
  tokio::pin!(wait_fut);
  ct.cancel();
  let _ = (&mut wait_fut).await;
}

async fn backoff_or_shutdown(
  runtime: &Arc<McpRuntimeManager>,
  server_id: &str,
  generation: u64,
  attempts: u32,
  shutdown_rx: &mut watch::Receiver<bool>,
) {
  let backoff_secs = (1_u64 << attempts.min(5)).min(MCP_RECONNECT_MAX_BACKOFF_SECS);
  tokio::select! {
    _ = shutdown_rx.changed() => {
      runtime.set_status_for_worker(server_id, generation, McpServerLifecycleState::Disconnected, None).await;
    }
    _ = tokio::time::sleep(Duration::from_secs(backoff_secs)) => {}
  }
}
