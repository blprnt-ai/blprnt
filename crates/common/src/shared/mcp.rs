use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use rmcp::model::InitializeResult;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum McpTransportKind {
  #[default]
  Stdio,
  SseHttp,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum McpServerLifecycleState {
  #[default]
  Configured,
  Connecting,
  Connected,
  Degraded,
  Disconnected,
  Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpStdioTransportConfig {
  pub command: String,
  pub args:    Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env:     Option<BTreeMap<String, String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cwd:     Option<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpSseHttpTransportConfig {
  pub url:     String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub headers: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransportConfig {
  Stdio(McpStdioTransportConfig),
  SseHttp(McpSseHttpTransportConfig),
}

impl Default for McpTransportConfig {
  fn default() -> Self {
    Self::Stdio(McpStdioTransportConfig::default())
  }
}

#[derive(Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpAuthConfig {
  #[default]
  None,
  BearerToken {
    token: String,
  },
  ApiKey {
    header: String,
    key:    String,
  },
  Basic {
    username: String,
    password: String,
  },
  Headers {
    headers: BTreeMap<String, String>,
  },
}

impl std::fmt::Debug for McpAuthConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::None => f.debug_tuple("None").finish(),
      Self::BearerToken { .. } => f.debug_struct("BearerToken").field("token", &"[REDACTED]").finish(),
      Self::ApiKey { header, .. } => {
        f.debug_struct("ApiKey").field("header", header).field("key", &"[REDACTED]").finish()
      }
      Self::Basic { username, .. } => {
        f.debug_struct("Basic").field("username", username).field("password", &"[REDACTED]").finish()
      }
      Self::Headers { headers } => {
        let redacted_headers = headers.keys().map(|key| (key, "[REDACTED]")).collect::<BTreeMap<_, _>>();
        f.debug_struct("Headers").field("headers", &redacted_headers).finish()
      }
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpServerConfig {
  pub id:        String,
  pub name:      String,
  pub enabled:   bool,
  pub transport: McpTransportConfig,
  pub auth:      McpAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpServerCreateParams {
  pub name:      String,
  #[serde(default = "default_enabled")]
  pub enabled:   bool,
  pub transport: McpTransportConfig,
  #[serde(default)]
  pub auth:      McpAuthConfig,
}

impl Default for McpServerCreateParams {
  fn default() -> Self {
    Self {
      name:      String::default(),
      enabled:   default_enabled(),
      transport: McpTransportConfig::default(),
      auth:      McpAuthConfig::default(),
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpServerPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:      Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled:   Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub transport: Option<McpTransportConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth:      Option<McpAuthConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpServerStatus {
  pub server_id: String,
  pub state:     McpServerLifecycleState,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error:     Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct McpToolDescriptor {
  pub server_id:    String,
  pub name:         String,
  pub description:  String,
  pub input_schema: serde_json::Value,
}

#[async_trait]
pub trait McpRuntimeBridge: Send + Sync {
  async fn list_tools(&self) -> Vec<McpToolDescriptor>;
  async fn call_tool(
    &self,
    server_id: String,
    name: String,
    arguments: serde_json::Value,
  ) -> Result<serde_json::Value, String>;
  async fn get_initialize_results(&self) -> HashMap<String, InitializeResult>;
}

pub type McpRuntimeBridgeRef = Arc<dyn McpRuntimeBridge>;

impl std::fmt::Debug for dyn McpRuntimeBridge {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "McpRuntimeBridge")
  }
}

fn default_enabled() -> bool {
  true
}

const MCP_TOOL_RUNTIME_PREFIX: &str = "mcp__";

pub fn mcp_tool_runtime_name(server_id: &str, tool_name: &str) -> String {
  format!("{MCP_TOOL_RUNTIME_PREFIX}{server_id}__{tool_name}")
}

pub fn parse_mcp_tool_runtime_name(runtime_name: &str) -> Option<(String, String)> {
  let rest = runtime_name.strip_prefix(MCP_TOOL_RUNTIME_PREFIX)?;
  let mut parts = rest.splitn(2, "__");
  let server_id = parts.next()?.trim();
  let tool_name = parts.next()?.trim();
  if server_id.is_empty() || tool_name.is_empty() {
    return None;
  }
  Some((server_id.to_string(), tool_name.to_string()))
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::McpAuthConfig;
  use super::McpServerCreateParams;
  use super::mcp_tool_runtime_name;
  use super::parse_mcp_tool_runtime_name;

  #[test]
  fn auth_debug_redacts_secret_values() {
    let bearer = format!("{:?}", McpAuthConfig::BearerToken { token: "top-secret-token".to_string() });
    assert!(bearer.contains("[REDACTED]"));
    assert!(!bearer.contains("top-secret-token"));

    let api_key =
      format!("{:?}", McpAuthConfig::ApiKey { header: "x-api-key".to_string(), key: "secret-key".to_string() });
    assert!(api_key.contains("x-api-key"));
    assert!(api_key.contains("[REDACTED]"));
    assert!(!api_key.contains("secret-key"));

    let basic =
      format!("{:?}", McpAuthConfig::Basic { username: "operator".to_string(), password: "secret-pass".to_string() });
    assert!(basic.contains("operator"));
    assert!(basic.contains("[REDACTED]"));
    assert!(!basic.contains("secret-pass"));

    let mut headers = BTreeMap::new();
    headers.insert("x-custom".to_string(), "sensitive".to_string());
    let header_auth = format!("{:?}", McpAuthConfig::Headers { headers });
    assert!(header_auth.contains("x-custom"));
    assert!(header_auth.contains("[REDACTED]"));
    assert!(!header_auth.contains("sensitive"));
  }

  #[test]
  fn create_params_enabled_defaults_true_when_omitted() {
    let value = serde_json::json!({
      "name": "Server",
      "transport": {
        "type": "stdio",
        "command": "node",
        "args": []
      }
    });

    let params: McpServerCreateParams = serde_json::from_value(value).expect("params must deserialize");
    assert!(params.enabled);
  }

  #[test]
  fn create_params_default_sets_enabled_true() {
    let params = McpServerCreateParams::default();
    assert!(params.enabled);
  }

  #[test]
  fn create_params_enabled_respects_explicit_false() {
    let value = serde_json::json!({
      "name": "Server",
      "enabled": false,
      "transport": {
        "type": "stdio",
        "command": "node",
        "args": []
      }
    });

    let params: McpServerCreateParams = serde_json::from_value(value).expect("params must deserialize");
    assert!(!params.enabled);
  }

  #[test]
  fn header_auth_debug_redacts_all_header_values() {
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), "Bearer super-secret-token".to_string());
    headers.insert("x-api-key".to_string(), "super-secret-key".to_string());

    let header_auth = format!("{:?}", McpAuthConfig::Headers { headers });
    assert!(header_auth.contains("authorization"));
    assert!(header_auth.contains("x-api-key"));
    assert!(!header_auth.contains("super-secret-token"));
    assert!(!header_auth.contains("super-secret-key"));
    assert!(header_auth.contains("[REDACTED]"));
  }

  #[test]
  fn runtime_name_round_trips() {
    let runtime_name = mcp_tool_runtime_name("srv-1", "search_docs");
    assert_eq!(runtime_name, "mcp__srv-1__search_docs");
    assert_eq!(parse_mcp_tool_runtime_name(&runtime_name), Some(("srv-1".to_string(), "search_docs".to_string())));
    assert_eq!(parse_mcp_tool_runtime_name("mcp__srv-only"), None);
    assert_eq!(parse_mcp_tool_runtime_name("shell"), None);
  }
}
