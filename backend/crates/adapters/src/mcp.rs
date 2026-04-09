use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::McpServerId;
use persistence::prelude::McpServerPatch;
use persistence::prelude::McpServerRecord;
use persistence::prelude::McpServerRepository;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::model::Tool;
use rmcp::transport::ConfigureCommandExt;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::TokioChildProcess;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use shared::agent::ToolId;
use shared::tools::McpToolPayload;
use shared::tools::ToolSpec;
use shared::tools::ToolUseResponse;
use shared::tools::ToolUseResponseData;
use shared::tools::ToolUseResponseError;
use tokio::process::Command;

const MCP_TOOL_NAME_PREFIX: &str = "mcp__";
const MCP_TOOL_NAME_SEPARATOR: &str = "__";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredMcpOauthToken {
  pub access_token:      String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub refresh_token:     Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub expires_at_ms:     Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub token_type:        Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub scopes:            Vec<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub authorization_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ParsedMcpToolName {
  pub server_id: McpServerId,
  pub tool_name: String,
}

pub fn format_mcp_tool_name(server_id: &McpServerId, tool_name: &str) -> String {
  format!("{MCP_TOOL_NAME_PREFIX}{}{}{}", server_id.uuid(), MCP_TOOL_NAME_SEPARATOR, tool_name)
}

pub fn parse_mcp_tool_name(value: &str) -> Result<ParsedMcpToolName> {
  let encoded = value.strip_prefix(MCP_TOOL_NAME_PREFIX).context("mcp tool name must start with mcp__")?;
  let (server_id, tool_name) =
    encoded.split_once(MCP_TOOL_NAME_SEPARATOR).context("mcp tool name must include a server id and tool name")?;

  let server_id = Uuid::parse_str(server_id).context("invalid mcp server id in tool name")?;
  let tool_name = tool_name.trim();
  anyhow::ensure!(!tool_name.is_empty(), "mcp tool name is missing the server tool name");

  Ok(ParsedMcpToolName { server_id: server_id.into(), tool_name: tool_name.to_string() })
}

pub fn normalized_mcp_tool_success(
  server_id: &McpServerId,
  tool_name: &str,
  result: serde_json::Value,
) -> ToolUseResponse {
  ToolUseResponseData::success(ToolUseResponseData::McpTool(McpToolPayload {
    server_id: server_id.uuid().to_string(),
    name: format_mcp_tool_name(server_id, tool_name),
    result,
  }))
}

pub async fn store_mcp_server_oauth_token(server_id: &McpServerId, token: &StoredMcpOauthToken) -> Result<()> {
  vault::set_stronghold_secret(vault::Vault::Key, server_id.uuid(), &serde_json::to_string(token)?).await
}

pub async fn load_mcp_server_oauth_token(server_id: &McpServerId) -> Result<Option<StoredMcpOauthToken>> {
  let Some(raw) = vault::get_stronghold_secret(vault::Vault::Key, server_id.uuid()).await else {
    return Ok(None);
  };

  let token = serde_json::from_str(&raw).context("failed to deserialize stored MCP OAuth token")?;
  Ok(Some(token))
}

pub async fn delete_mcp_server_oauth_token(server_id: &McpServerId) -> Result<()> {
  vault::delete_stronghold_secret(vault::Vault::Key, server_id.uuid()).await
}

pub fn tool_id_to_mcp_name(tool_id: &ToolId) -> Option<ParsedMcpToolName> {
  match tool_id {
    ToolId::Mcp(name) => parse_mcp_tool_name(name).ok(),
    _ => None,
  }
}

fn mcp_tool_to_tool_spec(server_id: &McpServerId, tool: Tool) -> ToolSpec {
  ToolSpec {
    name:        serde_json::Value::String(format_mcp_tool_name(server_id, tool.name.as_ref())),
    description: serde_json::Value::String(tool.description.map(|value| value.into_owned()).unwrap_or_default()),
    params:      serde_json::to_value(tool.input_schema.as_ref())
      .unwrap_or_else(|_| serde_json::json!({ "type": "object" })),
  }
}

fn mcp_call_result_to_json(result: &rmcp::model::CallToolResult) -> serde_json::Value {
  serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({ "error": "failed to serialize MCP tool result" }))
}

fn mcp_auth_preflight_error(server: &McpServerRecord, token: Option<&StoredMcpOauthToken>) -> Option<anyhow::Error> {
  match server.auth_state {
    shared::tools::McpServerAuthState::AuthRequired => Some(anyhow::anyhow!(
      "MCP server '{}' requires OAuth authorization before it can be enabled or executed{}{}",
      server.display_name,
      server.auth_summary.as_ref().map(|summary| format!(": {summary}")).unwrap_or_default(),
      token
        .and_then(|stored| stored.authorization_url.as_ref())
        .map(|url| format!(". authorization_url={url}"))
        .unwrap_or_default()
    )),
    shared::tools::McpServerAuthState::ReconnectRequired => Some(anyhow::anyhow!(
      "MCP server '{}' requires reconnect before it can be enabled or executed{}{}",
      server.display_name,
      server.auth_summary.as_ref().map(|summary| format!(": {summary}")).unwrap_or_default(),
      token
        .and_then(|stored| stored.authorization_url.as_ref())
        .map(|url| format!(". authorization_url={url}"))
        .unwrap_or_default()
    )),
    shared::tools::McpServerAuthState::Connected
      if token.as_ref().and_then(|stored| stored.authorization_url.as_ref()).is_some() =>
    {
      None
    }
    shared::tools::McpServerAuthState::Connected | shared::tools::McpServerAuthState::NotConnected => None,
  }
}

fn stored_token_is_expired(token: &StoredMcpOauthToken) -> bool {
  token.expires_at_ms.map(|expires_at_ms| expires_at_ms <= Utc::now().timestamp_millis().max(0) as u64).unwrap_or(false)
}

fn auth_hint_suffix(token: Option<&StoredMcpOauthToken>) -> String {
  token
    .and_then(|stored| stored.authorization_url.as_ref())
    .map(|url| format!(". authorization_url={url}"))
    .unwrap_or_default()
}

fn looks_like_auth_failure(error: &anyhow::Error) -> bool {
  let rendered = format!("{error:#}").to_ascii_lowercase();
  rendered.contains("401")
    || rendered.contains("403")
    || rendered.contains("unauthorized")
    || rendered.contains("forbidden")
    || rendered.contains("authorization required")
    || rendered.contains("token refresh failed")
    || rendered.contains("access token expired")
}

async fn mark_server_reconnect_required(
  server: &McpServerRecord,
  token: Option<&StoredMcpOauthToken>,
  reason: impl Into<String>,
) -> Result<()> {
  let reason = reason.into();
  let summary = Some(match token.and_then(|stored| stored.authorization_url.as_ref()) {
    Some(url) => format!("{reason}. Reconnect required: {url}"),
    None => reason,
  });

  McpServerRepository::update(
    server.id.clone(),
    McpServerPatch {
      auth_state: Some(shared::tools::McpServerAuthState::ReconnectRequired),
      auth_summary: Some(summary),
      ..Default::default()
    },
  )
  .await?;

  Ok(())
}

pub fn server_blocks_tool_materialization(server: &McpServerRecord) -> bool {
  matches!(
    server.auth_state,
    shared::tools::McpServerAuthState::AuthRequired | shared::tools::McpServerAuthState::ReconnectRequired
  )
}

async fn with_mcp_client<T>(
  server: &McpServerRecord,
  f: impl for<'a> FnOnce(
    &'a rmcp::service::RunningService<rmcp::RoleClient, ()>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>>,
) -> Result<T> {
  let token = load_mcp_server_oauth_token(&server.id).await?;
  if let Some(stored) = token.as_ref().filter(|stored| stored_token_is_expired(stored)) {
    mark_server_reconnect_required(
      server,
      Some(stored),
      if stored.refresh_token.is_some() {
        "Stored OAuth token expired and the server must be reconnected before retrying phase-0 MCP execution"
      } else {
        "Stored OAuth token expired and no refresh token is available; reconnect is required"
      },
    )
    .await?;

    return Err(anyhow::anyhow!(
      "MCP server '{}' requires reconnect before it can be enabled or executed: stored OAuth token expired{}",
      server.display_name,
      auth_hint_suffix(Some(stored))
    ));
  }
  if let Some(error) = mcp_auth_preflight_error(server, token.as_ref()) {
    return Err(error);
  }

  let client_result = match server.transport.as_str() {
    "streamable_http" | "http" | "streamable-http" => {
      let mut config = StreamableHttpClientTransportConfig::with_uri(server.endpoint_url.clone());
      if let Some(token) = token.as_ref() {
        config = config.auth_header(token.access_token.clone());
      }
      ().serve(StreamableHttpClientTransport::from_config(config)).await
    }
    "stdio" => {
      let transport = TokioChildProcess::new(Command::new("sh").configure(|cmd| {
        cmd.arg("-lc").arg(server.endpoint_url.as_str());
      }))?;
      ().serve(transport).await
    }
    other => {
      return Err(anyhow::anyhow!("unsupported MCP transport '{}' for server '{}'", other, server.display_name));
    }
  }
  .with_context(|| format!("failed to establish MCP client session for server '{}'", server.display_name));

  let client = match client_result {
    Ok(client) => client,
    Err(error) => {
      if looks_like_auth_failure(&error) {
        mark_server_reconnect_required(
          server,
          token.as_ref(),
          "Stored OAuth credentials were rejected while opening the MCP session",
        )
        .await?;
      }
      return Err(error);
    }
  };

  let result = f(&client).await;
  let _ = client.cancel().await;
  if let Err(error) = &result
    && looks_like_auth_failure(error)
  {
    mark_server_reconnect_required(
      server,
      token.as_ref(),
      "Stored OAuth credentials were rejected during MCP tool discovery or execution",
    )
    .await?;
  }
  result
}

pub async fn load_mcp_tool_specs(server: &McpServerRecord) -> Result<Vec<ToolSpec>> {
  let server_id = server.id.uuid().to_string();
  let server_name = server.display_name.clone();
  with_mcp_client(server, |client| {
    Box::pin(async move {
      let tools = client
        .peer()
        .list_all_tools()
        .await
        .with_context(|| format!("failed to list MCP tools for server '{}'", server_name))?;
      let server_id: McpServerId = Uuid::parse_str(&server_id)?.into();
      Ok(tools.into_iter().map(|tool| mcp_tool_to_tool_spec(&server_id, tool)).collect())
    })
  })
  .await
}

pub async fn execute_mcp_tool_call(
  server: &McpServerRecord,
  tool_use_id: &str,
  tool_name: &str,
  input: serde_json::Value,
) -> ToolUseResponse {
  let server_id = server.id.uuid().to_string();
  let server_name = server.display_name.clone();
  let tool_name_owned = tool_name.to_string();
  match with_mcp_client(server, |client| {
    Box::pin(async move {
      let arguments = input
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("MCP tool '{}' arguments must be a JSON object", tool_name_owned))?;
      let result = client
        .peer()
        .call_tool(CallToolRequestParams::new(tool_name_owned.clone()).with_arguments(arguments))
        .await
        .with_context(|| format!("failed to execute MCP tool '{}' on server '{}'", tool_name_owned, server_name))?;
      let server_id: McpServerId = Uuid::parse_str(&server_id)?.into();
      Ok(normalized_mcp_tool_success(&server_id, &tool_name_owned, mcp_call_result_to_json(&result)))
    })
  })
  .await
  {
    Ok(response) => response,
    Err(error) => ToolUseResponseError::error(
      ToolId::Mcp(format_mcp_tool_name(&server.id, tool_name)),
      format!("MCP tool execution failed for tool_use_id={tool_use_id}: {error}"),
    ),
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;
  use std::time::SystemTime;
  use std::time::UNIX_EPOCH;

  use super::*;

  struct EnvGuard {
    key:      &'static str,
    previous: Option<String>,
  }

  impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
      let previous = std::env::var(key).ok();
      unsafe { std::env::set_var(key, value) };
      Self { key, previous }
    }
  }

  impl Drop for EnvGuard {
    fn drop(&mut self) {
      match &self.previous {
        Some(value) => unsafe { std::env::set_var(self.key, value) },
        None => unsafe { std::env::remove_var(self.key) },
      }
    }
  }

  fn unique_temp_dir() -> std::path::PathBuf {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("blprnt-mcp-auth-{suffix}"));
    std::fs::create_dir_all(&path).unwrap();
    path
  }

  #[test]
  fn formats_and_parses_mcp_tool_names() {
    let server_id: McpServerId = Uuid::new_v4().into();
    let formatted = format_mcp_tool_name(&server_id, "search_docs");

    assert_eq!(formatted, format!("mcp__{}__search_docs", server_id.uuid()));

    let parsed = parse_mcp_tool_name(&formatted).expect("tool name should parse");
    assert_eq!(parsed.server_id.uuid(), server_id.uuid());
    assert_eq!(parsed.tool_name, "search_docs");
  }

  #[test]
  fn normalized_mcp_tool_success_uses_runtime_name() {
    let server_id: McpServerId = Uuid::new_v4().into();
    let response = normalized_mcp_tool_success(&server_id, "fetch", serde_json::json!({ "ok": true }));

    let ToolUseResponse::Success(success) = response else {
      panic!("expected success response");
    };

    assert_eq!(success.tool_id, ToolId::Mcp(format!("mcp__{}__fetch", server_id.uuid())));
  }

  #[tokio::test]
  async fn stores_and_loads_mcp_server_oauth_tokens() {
    let _lock = crate::TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let temp = unique_temp_dir();
    let _home = EnvGuard::set("HOME", &temp);
    let _blprnt_home = EnvGuard::set("BLPRNT_HOME", &temp);

    let server_id: McpServerId = Uuid::new_v4().into();
    let token = StoredMcpOauthToken {
      access_token:      "access-token".to_string(),
      refresh_token:     Some("refresh-token".to_string()),
      expires_at_ms:     Some(12345),
      token_type:        Some("Bearer".to_string()),
      scopes:            vec!["tools:read".to_string(), "tools:execute".to_string()],
      authorization_url: Some("https://example.com/connect".to_string()),
    };

    store_mcp_server_oauth_token(&server_id, &token).await.expect("token should store");

    let loaded =
      load_mcp_server_oauth_token(&server_id).await.expect("token load should succeed").expect("token should exist");
    assert_eq!(loaded, token);

    delete_mcp_server_oauth_token(&server_id).await.expect("token delete should succeed");
    assert!(load_mcp_server_oauth_token(&server_id).await.expect("reload should succeed").is_none());
  }
}
