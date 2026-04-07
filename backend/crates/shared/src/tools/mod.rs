pub mod prelude;

pub mod config;
pub mod file;
pub mod host;
pub mod mcp;

use std::path::PathBuf;

pub use file::*;
pub use host::*;
pub use mcp::*;
use serde_json::Value;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::agent::ToolId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
  pub history_id:  Uuid,
  pub tool_use_id: String,
  pub result:      ToolUseResponse,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ToolSpec {
  pub name:        Value,
  pub description: Value,
  pub params:      Value,
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct McpToolPayload {
  pub server_id: String,
  pub name:      String,
  pub result:    Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WorkingDirectories {
  pub directories: Vec<PathBuf>,
}

impl WorkingDirectories {
  pub fn new(directories: Vec<PathBuf>) -> Self {
    Self { directories }
  }

  pub fn pretty_print(&self) -> String {
    // Get common prefix of all directories as pathbuf
    let common_prefix = self.common_ancestor();

    // Strip common prefix from all directories
    let directories = self
      .directories
      .iter()
      .map(|d| match &common_prefix {
        Some(prefix) => d.strip_prefix(prefix).unwrap_or(d),
        None => d,
      })
      .collect::<Vec<_>>();

    directories
      .iter()
      .enumerate()
      .map(|(index, d)| format!("{}: {}", index, d.display()))
      .collect::<Vec<_>>()
      .join("\n")
  }

  fn common_ancestor(&self) -> Option<PathBuf> {
    let mut paths_iter = self.directories.iter();

    let first_path = paths_iter.next()?.as_path();
    let mut common_components: Vec<_> = first_path.components().collect();

    for next_path in paths_iter {
      let next_components: Vec<_> = next_path.components().collect();

      let shared_len =
        common_components.iter().zip(next_components.iter()).take_while(|(left, right)| left == right).count();

      common_components.truncate(shared_len);

      if common_components.is_empty() {
        return None;
      }
    }

    let mut common_path = PathBuf::new();
    common_path.extend(common_components);
    Some(common_path)
  }
}

impl From<Vec<PathBuf>> for WorkingDirectories {
  fn from(directories: Vec<PathBuf>) -> Self {
    Self { directories }
  }
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolUseResponse {
  Success(ToolUseResponseSuccess),
  Error(ToolUseResponseError),
}

impl ToolUseResponse {
  pub fn is_ok(&self) -> bool {
    matches!(self, ToolUseResponse::Success(_))
  }

  pub fn is_err(&self) -> bool {
    matches!(self, ToolUseResponse::Error(_))
  }

  pub fn into_llm_payload(&self) -> Value {
    match self {
      ToolUseResponse::Success(success) => success.data.into_llm_payload(),
      ToolUseResponse::Error(error) => serde_json::to_value(error).unwrap_or_default(),
    }
  }

  pub fn get_tool_id(&self) -> ToolId {
    match self {
      ToolUseResponse::Success(success) => success.data.get_tool_id(),
      ToolUseResponse::Error(error) => error.tool_id.clone(),
    }
  }
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct ToolUseResponseSuccess {
  #[schema(example = true)]
  pub success: bool,
  pub tool_id: ToolId,
  pub data:    ToolUseResponseData,
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct ToolUseResponseError {
  #[schema(example = false)]
  pub success: bool,
  pub tool_id: ToolId,
  pub error:   String,
}

impl ToolUseResponseError {
  pub fn error(tool_id: ToolId, error: impl std::fmt::Display) -> ToolUseResponse {
    ToolUseResponse::Error(ToolUseResponseError { success: false, tool_id, error: error.to_string() })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolUseResponseData {
  // File
  FilesRead(FilesReadPayload),
  ApplyPatch(ApplyPatchPayload),

  // Shell
  Shell(ShellPayload),

  // Runtime MCP enablement
  EnableMcpServer(EnableMcpServerPayload),

  // MCP
  McpTool(McpToolPayload),

  // Unknown persisted payloads
  Unknown(UnknownToolUseResponsePayload),
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct UnknownToolUseResponsePayload {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub original_type: Option<String>,
  #[schema(value_type = Object)]
  pub raw:           Value,
  pub error:         String,
}

impl ToolUseResponseData {
  pub fn success(data: Self) -> ToolUseResponse {
    let tool_id = data.get_tool_id();
    ToolUseResponse::Success(ToolUseResponseSuccess { success: true, data, tool_id })
  }

  pub fn into_llm_payload(&self) -> Value {
    let mut value = serde_json::to_value(self).unwrap_or_default();

    if let Some(object) = value.as_object_mut() {
      object.remove("type");
    }

    value
  }

  pub fn get_tool_id(&self) -> ToolId {
    match self {
      ToolUseResponseData::FilesRead(_) => ToolId::FilesRead,
      ToolUseResponseData::ApplyPatch(_) => ToolId::ApplyPatch,
      ToolUseResponseData::Shell(_) => ToolId::Shell,
      ToolUseResponseData::EnableMcpServer(_) => ToolId::EnableMcpServer,
      ToolUseResponseData::McpTool(payload) => ToolId::Mcp(payload.name.clone()),
      ToolUseResponseData::Unknown(payload) => {
        ToolId::Unknown(payload.original_type.clone().unwrap_or_else(|| "unknown".to_string()))
      }
    }
  }

  fn unknown(raw: surrealdb_types::Value, error: impl std::fmt::Display) -> Self {
    let raw = Value::from_value(raw.clone()).unwrap_or_else(|json_error| {
      serde_json::json!({
        "surreal_value_debug": format!("{:?}", raw),
        "json_error": json_error.to_string(),
      })
    });

    let original_type = raw
      .as_object()
      .and_then(|object| (object.len() == 1).then_some(object))
      .and_then(|object| object.keys().next().cloned());

    Self::Unknown(UnknownToolUseResponsePayload { original_type, raw, error: error.to_string() })
  }

  fn into_surreal_value(name: &str, payload: impl SurrealValue) -> surrealdb_types::Value {
    let mut object = surrealdb_types::Object::new();
    object.insert(name, payload);
    surrealdb_types::Value::Object(object)
  }
}

impl SurrealValue for ToolUseResponseData {
  fn into_value(self) -> surrealdb_types::Value {
    match self {
      ToolUseResponseData::FilesRead(payload) => Self::into_surreal_value("FilesRead", payload),
      ToolUseResponseData::ApplyPatch(payload) => Self::into_surreal_value("ApplyPatch", payload),
      ToolUseResponseData::Shell(payload) => Self::into_surreal_value("Shell", payload),
      ToolUseResponseData::EnableMcpServer(payload) => Self::into_surreal_value("EnableMcpServer", payload),
      ToolUseResponseData::McpTool(payload) => Self::into_surreal_value("McpTool", payload),
      ToolUseResponseData::Unknown(payload) => Self::into_surreal_value("Unknown", payload),
    }
  }

  fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
    let raw = value.clone();
    let surrealdb_types::Value::Object(object) = value else {
      return Ok(Self::unknown(raw, "Failed to decode ToolUseResponseData, expected object"));
    };

    if object.len() != 1 {
      return Ok(Self::unknown(raw, "Failed to decode ToolUseResponseData, expected single-key object"));
    }

    let Some((variant, payload)) = object.into_iter().next() else {
      return Ok(Self::unknown(raw, "Failed to decode ToolUseResponseData, missing variant"));
    };

    macro_rules! decode_payload {
      ($payload_ty:ty, $ctor:path) => {
        match <$payload_ty>::from_value(payload) {
          Ok(decoded) => Ok($ctor(decoded)),
          Err(error) => Ok(Self::unknown(raw, error)),
        }
      };
    }

    match variant.as_str() {
      "FilesRead" => decode_payload!(FilesReadPayload, Self::FilesRead),
      "ApplyPatch" => decode_payload!(ApplyPatchPayload, Self::ApplyPatch),
      "Shell" => decode_payload!(ShellPayload, Self::Shell),
      "EnableMcpServer" => decode_payload!(EnableMcpServerPayload, Self::EnableMcpServer),
      "McpTool" => decode_payload!(McpToolPayload, Self::McpTool),
      "Unknown" => decode_payload!(UnknownToolUseResponsePayload, Self::Unknown),
      _ => Ok(Self::unknown(raw, format!("Unknown ToolUseResponseData variant: {}", variant))),
    }
  }

  fn kind_of() -> surrealdb_types::Kind {
    surrealdb_types::kind!(object)
  }
}
