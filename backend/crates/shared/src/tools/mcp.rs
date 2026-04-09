use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(
  title = "enable_mcp_server",
  description = "Enables a configured MCP server for the current run without injecting its tool definitions up front."
)]
pub struct EnableMcpServerArgs {
  #[schemars(description = "Configured MCP server record id to enable for the current run.")]
  pub server_id: String,
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum McpServerAuthState {
  NotConnected,
  AuthRequired,
  Connected,
  ReconnectRequired,
}

impl Default for McpServerAuthState {
  fn default() -> Self {
    Self::NotConnected
  }
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct EnableMcpServerPayload {
  pub server_id:    String,
  pub display_name: String,
  pub description:  String,
  pub auth_state:   McpServerAuthState,
}

impl From<EnableMcpServerPayload> for ToolUseResponseData {
  fn from(payload: EnableMcpServerPayload) -> Self {
    Self::EnableMcpServer(payload)
  }
}
