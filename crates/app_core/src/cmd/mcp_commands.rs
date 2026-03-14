use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use common::blprnt::Blprnt;
use common::errors::TauriError;
use common::errors::TauriResult;
use common::shared::prelude::McpAuthConfig;
use common::shared::prelude::McpServerConfig;
use common::shared::prelude::McpServerCreateParams;
use common::shared::prelude::McpServerLifecycleState;
use common::shared::prelude::McpServerPatch;
use common::shared::prelude::McpServerStatus;
use common::shared::prelude::McpSseHttpTransportConfig;
use common::shared::prelude::McpStdioTransportConfig;
use common::shared::prelude::McpToolDescriptor;
use common::shared::prelude::McpTransportConfig;
use tauri::State;
use tauri_plugin_store::StoreExt;
use url::Url;

use crate::engine_manager::BLPRNT_STORE;
use crate::engine_manager::EngineManager;

const MCP_CONFIG_VERSION_KEY: &str = "mcp_config_version";
const MCP_SERVERS_KEY: &str = "mcp_servers";
const MCP_LEGACY_SERVERS_KEY: &str = "mcpServers";
const MCP_CONFIG_VERSION_V1: u64 = 1;
const MCP_TEST_CONNECTION_TIMEOUT_SECS: u64 = 8;
const MCP_TEST_CONNECTION_POLL_INTERVAL_MS: u64 = 200;

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_list(_manager: State<'_, Arc<EngineManager>>) -> TauriResult<Vec<McpServerConfig>> {
  load_mcp_servers()
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_get(
  _manager: State<'_, Arc<EngineManager>>,
  _server_id: String,
) -> TauriResult<McpServerConfig> {
  let servers = load_mcp_servers()?;
  servers.into_iter().find(|server| server.id == _server_id).ok_or_else(|| TauriError::new("MCP server not found"))
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_create(
  manager: State<'_, Arc<EngineManager>>,
  params: McpServerCreateParams,
) -> TauriResult<McpServerConfig> {
  validate_server_create_params(&params)?;

  let mut servers = load_mcp_servers()?;
  let id = convert_case::ccase!(camel, params.name.trim());

  if servers.iter().any(|server| server.id == id) {
    return Err(TauriError::new("MCP server with this name already exists"));
  }

  let server = McpServerConfig {
    id:        id,
    name:      params.name.trim().to_string(),
    enabled:   params.enabled,
    transport: params.transport,
    auth:      params.auth,
  };

  validate_server_config(&server)?;
  servers.push(server.clone());
  save_mcp_servers(&servers)?;
  manager.sync_mcp_runtime_from_store().await;

  Ok(server)
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_update(
  manager: State<'_, Arc<EngineManager>>,
  server_id: String,
  patch: McpServerPatch,
) -> TauriResult<McpServerConfig> {
  validate_server_patch(&patch)?;

  let mut servers = load_mcp_servers()?;
  let server =
    servers.iter_mut().find(|server| server.id == server_id).ok_or_else(|| TauriError::new("MCP server not found"))?;

  if let Some(name) = patch.name {
    server.name = name.trim().to_string();
  }
  if let Some(enabled) = patch.enabled {
    server.enabled = enabled;
  }
  if let Some(transport) = patch.transport {
    server.transport = transport;
  }
  if let Some(auth) = patch.auth {
    server.auth = auth;
  }

  validate_server_config(server)?;
  let updated = server.clone();
  save_mcp_servers(&servers)?;
  manager.sync_mcp_runtime_from_store().await;

  Ok(updated)
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_delete(manager: State<'_, Arc<EngineManager>>, server_id: String) -> TauriResult<()> {
  let mut servers = load_mcp_servers()?;
  let original_len = servers.len();
  servers.retain(|server| server.id != server_id);
  if servers.len() == original_len {
    return Err(TauriError::new("MCP server not found"));
  }
  save_mcp_servers(&servers)?;
  manager.sync_mcp_runtime_from_store().await;
  Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_status_list(manager: State<'_, Arc<EngineManager>>) -> TauriResult<Vec<McpServerStatus>> {
  Ok(manager.mcp_statuses().await)
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_tools_list(manager: State<'_, Arc<EngineManager>>) -> TauriResult<Vec<McpToolDescriptor>> {
  Ok(manager.mcp_runtime_bridge().list_tools().await)
}

#[tauri::command]
#[specta::specta]
pub async fn mcp_server_test_connection(
  manager: State<'_, Arc<EngineManager>>,
  server_id: String,
) -> TauriResult<McpServerStatus> {
  let servers = load_mcp_servers()?;
  let server =
    servers.iter().find(|server| server.id == server_id).ok_or_else(|| TauriError::new("MCP server not found"))?;

  if !server.enabled {
    return Err(TauriError::new("MCP server is disabled. Enable it before running Test Connection."));
  }

  manager.sync_mcp_runtime_from_store().await;

  let deadline = Instant::now() + Duration::from_secs(MCP_TEST_CONNECTION_TIMEOUT_SECS);

  loop {
    let status = manager.mcp_statuses().await.into_iter().find(|status| status.server_id == server_id).unwrap_or(
      McpServerStatus { server_id: server_id.clone(), state: McpServerLifecycleState::Configured, error: None },
    );

    match status.state {
      McpServerLifecycleState::Connected => return Ok(status),
      McpServerLifecycleState::Degraded | McpServerLifecycleState::Error => {
        return Err(TauriError::new(
          status
            .error
            .unwrap_or_else(|| "MCP server connection failed. Verify transport/auth settings and retry.".to_string()),
        ));
      }
      McpServerLifecycleState::Disconnected => {
        return Err(TauriError::new(
          "MCP server disconnected during test connection. Retry after confirming the server process is reachable.",
        ));
      }
      McpServerLifecycleState::Configured | McpServerLifecycleState::Connecting => {
        if Instant::now() >= deadline {
          return Err(TauriError::new(
            "MCP server connection test timed out after 8s. Verify URL/command/auth settings and retry.",
          ));
        }
        tokio::time::sleep(Duration::from_millis(MCP_TEST_CONNECTION_POLL_INTERVAL_MS)).await;
      }
    }
  }
}

fn load_mcp_servers() -> TauriResult<Vec<McpServerConfig>> {
  let store =
    Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|_| TauriError::new("Failed to open config store"))?;
  let current_servers_value = store.get(MCP_SERVERS_KEY);
  let (version, servers_value, should_persist_migration) = resolve_store_values_for_load(
    store.get(MCP_CONFIG_VERSION_KEY).as_ref(),
    current_servers_value.clone(),
    store.get(MCP_LEGACY_SERVERS_KEY),
  )?;

  if should_persist_migration {
    if current_servers_value.is_none() {
      if let Some(servers) = servers_value.clone() {
        store.set(MCP_SERVERS_KEY, servers);
      }
    }

    store.set(MCP_CONFIG_VERSION_KEY, version);
    store.save().map_err(|_| TauriError::new("Failed to persist MCP config store"))?;
  }

  let Some(value) = servers_value else {
    return Ok(Vec::new());
  };

  let servers = serde_json::from_value::<Vec<McpServerConfig>>(value).map_err(|_| {
    TauriError::new("Invalid MCP server configuration payload. Open MCP settings, review entries, then save again.")
  })?;

  for server in &servers {
    validate_server_config(server)?;
  }

  Ok(servers)
}

fn save_mcp_servers(servers: &[McpServerConfig]) -> TauriResult<()> {
  for server in servers {
    validate_server_config(server)?;
  }

  let store =
    Blprnt::handle().clone().store(BLPRNT_STORE).map_err(|_| TauriError::new("Failed to open config store"))?;
  store.set(MCP_CONFIG_VERSION_KEY, MCP_CONFIG_VERSION_V1);
  store.set(
    MCP_SERVERS_KEY,
    serde_json::to_value(servers).map_err(|_| TauriError::new("Failed to serialize MCP server configuration"))?,
  );
  store.save().map_err(|_| TauriError::new("Failed to persist MCP config store"))?;
  Ok(())
}

fn validate_server_create_params(params: &McpServerCreateParams) -> TauriResult<()> {
  if params.name.trim().is_empty() {
    return Err(TauriError::new("MCP server name is required"));
  }
  validate_transport_config(&params.transport)?;
  validate_auth_config(&params.auth)
}

fn validate_server_patch(patch: &McpServerPatch) -> TauriResult<()> {
  if let Some(name) = &patch.name
    && name.trim().is_empty()
  {
    return Err(TauriError::new("MCP server name cannot be empty"));
  }

  if let Some(transport) = &patch.transport {
    validate_transport_config(transport)?;
  }
  if let Some(auth) = &patch.auth {
    validate_auth_config(auth)?;
  }
  Ok(())
}

fn validate_server_config(server: &McpServerConfig) -> TauriResult<()> {
  if server.id.trim().is_empty() {
    return Err(TauriError::new("MCP server id is required"));
  }
  if server.name.trim().is_empty() {
    return Err(TauriError::new("MCP server name is required"));
  }
  validate_transport_config(&server.transport)?;
  validate_auth_config(&server.auth)
}

fn validate_transport_config(transport: &McpTransportConfig) -> TauriResult<()> {
  match transport {
    McpTransportConfig::Stdio(config) => validate_stdio_transport_config(config),
    McpTransportConfig::SseHttp(config) => validate_sse_http_transport_config(config),
  }
}

fn validate_stdio_transport_config(config: &McpStdioTransportConfig) -> TauriResult<()> {
  if config.command.trim().is_empty() {
    return Err(TauriError::new("MCP stdio transport command is required"));
  }
  if let Some(cwd) = &config.cwd
    && cwd.as_os_str().is_empty()
  {
    return Err(TauriError::new("MCP stdio transport cwd cannot be empty"));
  }

  if let Some(env) = &config.env {
    for (key, value) in env {
      if key.trim().is_empty() || value.trim().is_empty() {
        return Err(TauriError::new("MCP stdio transport env entries must have non-empty key and value"));
      }
    }
  }
  Ok(())
}

fn validate_sse_http_transport_config(config: &McpSseHttpTransportConfig) -> TauriResult<()> {
  let url = config.url.trim();
  if url.is_empty() {
    return Err(TauriError::new("MCP SSE/HTTP transport URL is required"));
  }
  let parsed = Url::parse(url).map_err(|_| TauriError::new("MCP SSE/HTTP transport URL is invalid"))?;
  if parsed.scheme() != "http" && parsed.scheme() != "https" {
    return Err(TauriError::new("MCP SSE/HTTP transport URL must use http or https scheme"));
  }
  if let Some(headers) = &config.headers {
    for (key, value) in headers {
      if key.trim().is_empty() || value.trim().is_empty() {
        return Err(TauriError::new("MCP SSE/HTTP transport headers must have non-empty key and value"));
      }
    }
  }
  Ok(())
}

fn validate_auth_config(auth: &McpAuthConfig) -> TauriResult<()> {
  match auth {
    McpAuthConfig::None => Ok(()),
    McpAuthConfig::BearerToken { token } => {
      if token.trim().is_empty() {
        return Err(TauriError::new("MCP bearer token is required"));
      }
      Ok(())
    }
    McpAuthConfig::ApiKey { header, key } => {
      if header.trim().is_empty() || key.trim().is_empty() {
        return Err(TauriError::new("MCP API key auth requires non-empty header and key"));
      }
      Ok(())
    }
    McpAuthConfig::Basic { username, password } => {
      if username.trim().is_empty() || password.trim().is_empty() {
        return Err(TauriError::new("MCP basic auth requires non-empty username and password"));
      }
      Ok(())
    }
    McpAuthConfig::Headers { headers } => {
      if headers.is_empty() {
        return Err(TauriError::new("MCP header auth requires at least one header"));
      }
      for (key, value) in headers {
        if key.trim().is_empty() || value.trim().is_empty() {
          return Err(TauriError::new("MCP header auth requires non-empty header names and values"));
        }
      }
      Ok(())
    }
  }
}

fn resolve_store_values_for_load(
  version_value: Option<&serde_json::Value>,
  current_servers_value: Option<serde_json::Value>,
  legacy_servers_value: Option<serde_json::Value>,
) -> TauriResult<(u64, Option<serde_json::Value>, bool)> {
  let version = match version_value {
    Some(value) => {
      value.as_u64().ok_or_else(|| TauriError::new("Invalid MCP config version: expected numeric value"))?
    }
    None => 0,
  };

  if version > MCP_CONFIG_VERSION_V1 {
    return Err(TauriError::new("Unsupported MCP config version"));
  }

  if version < MCP_CONFIG_VERSION_V1 {
    let servers = if current_servers_value.is_some() { current_servers_value } else { legacy_servers_value };
    return Ok((MCP_CONFIG_VERSION_V1, servers, true));
  }

  Ok((version, current_servers_value, false))
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use std::path::PathBuf;

  use common::shared::prelude::McpAuthConfig;
  use common::shared::prelude::McpServerConfig;
  use common::shared::prelude::McpServerCreateParams;
  use common::shared::prelude::McpServerPatch;
  use common::shared::prelude::McpSseHttpTransportConfig;
  use common::shared::prelude::McpStdioTransportConfig;
  use common::shared::prelude::McpTransportConfig;

  use super::resolve_store_values_for_load;
  use super::validate_server_config;
  use super::validate_server_create_params;
  use super::validate_server_patch;

  fn valid_stdio_transport() -> McpTransportConfig {
    McpTransportConfig::Stdio(McpStdioTransportConfig {
      command: "node".to_string(),
      args:    vec![],
      env:     None,
      cwd:     Some(PathBuf::from(".")),
    })
  }

  #[test]
  fn migration_promotes_legacy_servers_when_current_servers_missing() {
    let legacy = serde_json::json!([{ "id": "s1", "name": "Legacy", "enabled": true }]);
    let (version, servers, should_persist) = resolve_store_values_for_load(None, None, Some(legacy.clone())).unwrap();

    assert_eq!(version, 1);
    assert_eq!(servers, Some(legacy));
    assert!(should_persist);
  }

  #[test]
  fn migration_with_no_version_or_servers_still_sets_v1() {
    let (version, servers, should_persist) = resolve_store_values_for_load(None, None, None).unwrap();

    assert_eq!(version, 1);
    assert_eq!(servers, None);
    assert!(should_persist);
  }

  #[test]
  fn migration_keeps_current_servers_when_present() {
    let current = serde_json::json!([{ "id": "s2", "name": "Current", "enabled": true }]);
    let legacy = serde_json::json!([{ "id": "s1", "name": "Legacy", "enabled": true }]);
    let (version, servers, should_persist) =
      resolve_store_values_for_load(None, Some(current.clone()), Some(legacy)).unwrap();

    assert_eq!(version, 1);
    assert_eq!(servers, Some(current));
    assert!(should_persist);
  }

  #[test]
  fn version_one_does_not_trigger_migration() {
    let (version, servers, should_persist) =
      resolve_store_values_for_load(Some(&serde_json::json!(1)), Some(serde_json::json!([{"id":"s1"}])), None).unwrap();

    assert_eq!(version, 1);
    assert!(!should_persist);
    assert!(servers.is_some());
  }

  #[test]
  fn invalid_version_value_is_rejected() {
    let error = resolve_store_values_for_load(Some(&serde_json::json!("one")), None, None).unwrap_err();
    assert!(error.to_string().contains("Invalid MCP config version"));
  }

  #[test]
  fn malformed_numeric_version_is_rejected() {
    let error = resolve_store_values_for_load(Some(&serde_json::json!(1.25)), None, None).unwrap_err();
    assert!(error.to_string().contains("Invalid MCP config version"));
  }

  #[test]
  fn future_version_is_rejected() {
    let error = resolve_store_values_for_load(Some(&serde_json::json!(99)), None, None).unwrap_err();
    assert!(error.to_string().contains("Unsupported MCP config version"));
  }

  #[test]
  fn version_one_ignores_legacy_servers() {
    let legacy = serde_json::json!([{ "id": "legacy", "name": "Legacy", "enabled": true }]);
    let (version, servers, should_persist) =
      resolve_store_values_for_load(Some(&serde_json::json!(1)), None, Some(legacy)).unwrap();

    assert_eq!(version, 1);
    assert_eq!(servers, None);
    assert!(!should_persist);
  }

  #[test]
  fn create_params_validation_rejects_empty_name_and_bad_transport() {
    let params = McpServerCreateParams {
      name:      "   ".to_string(),
      enabled:   true,
      transport: McpTransportConfig::Stdio(McpStdioTransportConfig {
        command: " ".to_string(),
        args:    vec![],
        env:     None,
        cwd:     None,
      }),
      auth:      McpAuthConfig::None,
    };

    let error = validate_server_create_params(&params).unwrap_err();
    assert!(error.to_string().contains("MCP server name is required"));
  }

  #[test]
  fn create_params_validation_rejects_empty_stdio_command() {
    let params = McpServerCreateParams {
      name:      "Server".to_string(),
      enabled:   true,
      transport: McpTransportConfig::Stdio(McpStdioTransportConfig {
        command: " ".to_string(),
        args:    vec![],
        env:     None,
        cwd:     Some(PathBuf::from(".")),
      }),
      auth:      McpAuthConfig::None,
    };

    let error = validate_server_create_params(&params).unwrap_err();
    assert!(error.to_string().contains("MCP stdio transport command is required"));
  }

  #[test]
  fn create_params_validation_rejects_empty_stdio_cwd() {
    let params = McpServerCreateParams {
      name:      "Server".to_string(),
      enabled:   true,
      transport: McpTransportConfig::Stdio(McpStdioTransportConfig {
        command: "node".to_string(),
        args:    vec![],
        env:     None,
        cwd:     Some(PathBuf::new()),
      }),
      auth:      McpAuthConfig::None,
    };

    let error = validate_server_create_params(&params).unwrap_err();
    assert!(error.to_string().contains("MCP stdio transport cwd cannot be empty"));
  }

  #[test]
  fn create_params_validation_rejects_stdio_env_blank_entries() {
    let mut env = BTreeMap::new();
    env.insert("TOKEN".to_string(), " ".to_string());
    let params = McpServerCreateParams {
      name:      "Server".to_string(),
      enabled:   true,
      transport: McpTransportConfig::Stdio(McpStdioTransportConfig {
        command: "node".to_string(),
        args:    vec![],
        env:     Some(env),
        cwd:     Some(PathBuf::from(".")),
      }),
      auth:      McpAuthConfig::None,
    };

    let error = validate_server_create_params(&params).unwrap_err();
    assert!(error.to_string().contains("MCP stdio transport env entries must have non-empty key and value"));
  }

  #[test]
  fn patch_validation_rejects_invalid_sse_http_url() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: Some(McpTransportConfig::SseHttp(McpSseHttpTransportConfig {
        url:     "ftp://invalid".to_string(),
        headers: None,
      })),
      auth:      None,
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("must use http or https scheme"));
  }

  #[test]
  fn patch_validation_rejects_malformed_sse_http_url() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: Some(McpTransportConfig::SseHttp(McpSseHttpTransportConfig {
        url:     "http://".to_string(),
        headers: None,
      })),
      auth:      None,
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP SSE/HTTP transport URL is invalid"));
  }

  #[test]
  fn patch_validation_rejects_sse_http_headers_with_blank_values() {
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), " ".to_string());
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: Some(McpTransportConfig::SseHttp(McpSseHttpTransportConfig {
        url:     "https://example.com/sse".to_string(),
        headers: Some(headers),
      })),
      auth:      None,
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP SSE/HTTP transport headers must have non-empty key and value"));
  }

  #[test]
  fn patch_validation_rejects_empty_bearer_token() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: None,
      auth:      Some(McpAuthConfig::BearerToken { token: " ".to_string() }),
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP bearer token is required"));
  }

  #[test]
  fn patch_validation_rejects_api_key_missing_parts() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: None,
      auth:      Some(McpAuthConfig::ApiKey { header: "x-api-key".to_string(), key: " ".to_string() }),
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP API key auth requires non-empty header and key"));
  }

  #[test]
  fn patch_validation_rejects_basic_auth_missing_parts() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: None,
      auth:      Some(McpAuthConfig::Basic { username: "user".to_string(), password: " ".to_string() }),
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP basic auth requires non-empty username and password"));
  }

  #[test]
  fn patch_validation_rejects_header_auth_blank_entries() {
    let mut headers = BTreeMap::new();
    headers.insert(" ".to_string(), "secret".to_string());
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: None,
      auth:      Some(McpAuthConfig::Headers { headers }),
    };

    let error = validate_server_patch(&patch).unwrap_err();
    assert!(error.to_string().contains("MCP header auth requires non-empty header names and values"));
  }

  #[test]
  fn patch_validation_accepts_valid_transport_and_no_auth() {
    let patch = McpServerPatch {
      name:      None,
      enabled:   None,
      transport: Some(McpTransportConfig::SseHttp(McpSseHttpTransportConfig {
        url:     "https://example.com/mcp".to_string(),
        headers: None,
      })),
      auth:      Some(McpAuthConfig::None),
    };

    assert!(validate_server_patch(&patch).is_ok());
  }

  #[test]
  fn server_config_validation_rejects_invalid_auth_payload() {
    let server = McpServerConfig {
      id:        "server-1".to_string(),
      name:      "Server".to_string(),
      enabled:   true,
      transport: McpTransportConfig::Stdio(McpStdioTransportConfig {
        command: "node".to_string(),
        args:    vec![],
        env:     None,
        cwd:     Some(PathBuf::from(".")),
      }),
      auth:      McpAuthConfig::Headers { headers: Default::default() },
    };

    let error = validate_server_config(&server).unwrap_err();
    assert!(error.to_string().contains("requires at least one header"));
  }

  #[test]
  fn server_config_validation_accepts_valid_auth_and_transport() {
    let server = McpServerConfig {
      id:        "server-1".to_string(),
      name:      "Server".to_string(),
      enabled:   true,
      transport: valid_stdio_transport(),
      auth:      McpAuthConfig::ApiKey { header: "x-api-key".to_string(), key: "super-secret".to_string() },
    };

    assert!(validate_server_config(&server).is_ok());
  }
}
