pub mod prelude;

pub mod config;
pub mod file;
pub mod host;
pub mod memory;
pub mod plan;
pub mod primer;
pub mod question;
pub mod rg;
pub mod skill;
pub mod subagent;

pub mod test;

use std::path::PathBuf;

pub use file::*;
pub use host::*;
pub use memory::*;
pub use plan::*;
pub use primer::*;
pub use question::*;
pub use rg::*;
use serde_json::Value;
pub use skill::*;
pub use subagent::*;
use surrealdb_types::SurrealValue;

use crate::agent::ToolId;
use crate::shared::prelude::SubAgentStatus;
use crate::shared::prelude::SurrealId;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
  pub history_id:  SurrealId,
  pub tool_use_id: String,
  pub result:      ToolUseResponse,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ToolSpec {
  pub name:        Value,
  pub description: Value,
  pub params:      Value,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
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

impl Default for ToolUseResponse {
  fn default() -> Self {
    ToolUseResponse::Success(ToolUseResponseSuccess {
      success: true,
      data:    ToolUseResponseData::default(),
      message: None,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ToolUseResponseSuccess {
  pub success: bool,
  pub data:    ToolUseResponseData,
  pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ToolUseResponseError {
  pub success:         bool,
  pub tool_id:         ToolId,
  pub error:           String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subagent_id:     Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subagent_status: Option<SubAgentStatus>,
}

impl ToolUseResponseError {
  pub fn error(tool_id: ToolId, error: impl std::fmt::Display) -> ToolUseResponse {
    ToolUseResponse::Error(ToolUseResponseError {
      success: false,
      tool_id,
      error: error.to_string(),
      subagent_id: None,
      subagent_status: None,
    })
  }

  pub fn error_with_subagent_id(
    tool_id: ToolId,
    error: impl std::fmt::Display,
    subagent_id: String,
  ) -> ToolUseResponse {
    ToolUseResponse::Error(ToolUseResponseError {
      success: false,
      tool_id,
      error: error.to_string(),
      subagent_id: Some(subagent_id),
      subagent_status: None,
    })
  }

  pub fn error_with_subagent_status(
    tool_id: ToolId,
    error: impl std::fmt::Display,
    subagent_id: Option<String>,
    subagent_status: SubAgentStatus,
  ) -> ToolUseResponse {
    ToolUseResponse::Error(ToolUseResponseError {
      success: false,
      tool_id,
      error: error.to_string(),
      subagent_id,
      subagent_status: Some(subagent_status),
    })
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolUseResponseData {
  // File
  FilesRead(FilesReadPayload),
  ApplyPatch(ApplyPatchPayload),

  // Shell
  Shell(ShellPayload),

  // Terminal
  Terminal(TerminalPayload),

  // Ask Question
  AskQuestion(AskQuestionPayload),

  // Plans
  PlanCreate(PlanCreatePayload),
  PlanList(PlanListPayload),
  PlanGet(PlanGetPayload),
  PlanUpdate(PlanUpdatePayload),
  PlanDelete(PlanDeletePayload),

  // Primer
  GetPrimer(GetPrimerPayload),
  UpdatePrimer(UpdatePrimerPayload),

  // Memory
  MemoryWrite(MemoryWriteResult),
  MemorySearch(MemorySearchResult),

  // Ripgrep
  RgSearch(RgSearchPayload),

  // Subagent
  #[serde(rename = "subagent")]
  SubAgent(SubAgentPayload),

  // Get Reference
  ListSkills(ListSkillsPayload),
  ApplySkill(ApplySkillPayload),
  GetReference(GetReferencePayload),
  SkillScript(SkillScriptPayload),

  // MCP
  McpTool(McpToolPayload),

  // Unknown persisted payloads
  Unknown(UnknownToolUseResponsePayload),

  // Default
  #[default]
  Default,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct UnknownToolUseResponsePayload {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub original_type: Option<String>,
  pub raw:           Value,
  pub error:         String,
}

impl ToolUseResponseData {
  pub fn success(data: Self) -> ToolUseResponse {
    ToolUseResponse::Success(ToolUseResponseSuccess { success: true, data, message: None })
  }

  pub fn with_message(data: Self, message: String) -> ToolUseResponse {
    ToolUseResponse::Success(ToolUseResponseSuccess { success: true, data, message: Some(message) })
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
      ToolUseResponseData::Terminal(_) => ToolId::Terminal,
      ToolUseResponseData::AskQuestion(_) => ToolId::AskQuestion,
      ToolUseResponseData::PlanCreate(_) => ToolId::PlanCreate,
      ToolUseResponseData::PlanList(_) => ToolId::PlanList,
      ToolUseResponseData::PlanGet(_) => ToolId::PlanGet,
      ToolUseResponseData::PlanUpdate(_) => ToolId::PlanUpdate,
      ToolUseResponseData::PlanDelete(_) => ToolId::PlanDelete,
      ToolUseResponseData::GetPrimer(_) => ToolId::PrimerGet,
      ToolUseResponseData::UpdatePrimer(_) => ToolId::PrimerUpdate,
      ToolUseResponseData::MemoryWrite(_) => ToolId::MemoryWrite,
      ToolUseResponseData::MemorySearch(_) => ToolId::MemorySearch,
      ToolUseResponseData::RgSearch(_) => ToolId::Rg,
      ToolUseResponseData::SubAgent(_) => ToolId::SubAgent,
      ToolUseResponseData::ListSkills(_) => ToolId::ListSkills,
      ToolUseResponseData::ApplySkill(_) => ToolId::ApplySkill,
      ToolUseResponseData::GetReference(_) => ToolId::GetReference,
      ToolUseResponseData::SkillScript(_) => ToolId::SkillScript,
      ToolUseResponseData::McpTool(payload) => ToolId::Mcp(payload.name.clone()),
      ToolUseResponseData::Unknown(payload) => {
        ToolId::Unknown(payload.original_type.clone().unwrap_or_else(|| "unknown".to_string()))
      }
      ToolUseResponseData::Default => unreachable!(),
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
      ToolUseResponseData::Terminal(payload) => Self::into_surreal_value("Terminal", payload),
      ToolUseResponseData::AskQuestion(payload) => Self::into_surreal_value("AskQuestion", payload),
      ToolUseResponseData::PlanCreate(payload) => Self::into_surreal_value("PlanCreate", payload),
      ToolUseResponseData::PlanList(payload) => Self::into_surreal_value("PlanList", payload),
      ToolUseResponseData::PlanGet(payload) => Self::into_surreal_value("PlanGet", payload),
      ToolUseResponseData::PlanUpdate(payload) => Self::into_surreal_value("PlanUpdate", payload),
      ToolUseResponseData::PlanDelete(payload) => Self::into_surreal_value("PlanDelete", payload),
      ToolUseResponseData::GetPrimer(payload) => Self::into_surreal_value("GetPrimer", payload),
      ToolUseResponseData::UpdatePrimer(payload) => Self::into_surreal_value("UpdatePrimer", payload),
      ToolUseResponseData::MemoryWrite(payload) => Self::into_surreal_value("MemoryWrite", payload),
      ToolUseResponseData::MemorySearch(payload) => Self::into_surreal_value("MemorySearch", payload),
      ToolUseResponseData::RgSearch(payload) => Self::into_surreal_value("RgSearch", payload),
      ToolUseResponseData::SubAgent(payload) => Self::into_surreal_value("SubAgent", payload),
      ToolUseResponseData::ListSkills(payload) => Self::into_surreal_value("ListSkills", payload),
      ToolUseResponseData::ApplySkill(payload) => Self::into_surreal_value("ApplySkill", payload),
      ToolUseResponseData::GetReference(payload) => Self::into_surreal_value("GetReference", payload),
      ToolUseResponseData::SkillScript(payload) => Self::into_surreal_value("SkillScript", payload),
      ToolUseResponseData::McpTool(payload) => Self::into_surreal_value("McpTool", payload),
      ToolUseResponseData::Unknown(payload) => Self::into_surreal_value("Unknown", payload),
      ToolUseResponseData::Default => Self::into_surreal_value("Default", surrealdb_types::Object::new()),
    }
  }

  fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error> {
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
      "Terminal" => decode_payload!(TerminalPayload, Self::Terminal),
      "AskQuestion" => decode_payload!(AskQuestionPayload, Self::AskQuestion),
      "PlanCreate" => decode_payload!(PlanCreatePayload, Self::PlanCreate),
      "PlanList" => decode_payload!(PlanListPayload, Self::PlanList),
      "PlanGet" => decode_payload!(PlanGetPayload, Self::PlanGet),
      "PlanUpdate" => decode_payload!(PlanUpdatePayload, Self::PlanUpdate),
      "PlanDelete" => decode_payload!(PlanDeletePayload, Self::PlanDelete),
      "GetPrimer" => decode_payload!(GetPrimerPayload, Self::GetPrimer),
      "UpdatePrimer" => decode_payload!(UpdatePrimerPayload, Self::UpdatePrimer),
      "MemoryWrite" => decode_payload!(MemoryWriteResult, Self::MemoryWrite),
      "MemorySearch" => decode_payload!(MemorySearchResult, Self::MemorySearch),
      "RgSearch" => decode_payload!(RgSearchPayload, Self::RgSearch),
      "SubAgent" => decode_payload!(SubAgentPayload, Self::SubAgent),
      "ListSkills" => decode_payload!(ListSkillsPayload, Self::ListSkills),
      "ApplySkill" => decode_payload!(ApplySkillPayload, Self::ApplySkill),
      "GetReference" => decode_payload!(GetReferencePayload, Self::GetReference),
      "SkillScript" => decode_payload!(SkillScriptPayload, Self::SkillScript),
      "McpTool" => decode_payload!(McpToolPayload, Self::McpTool),
      "Unknown" => decode_payload!(UnknownToolUseResponsePayload, Self::Unknown),
      "Default" => match payload {
        surrealdb_types::Value::Object(object) if object.is_empty() => Ok(Self::Default),
        _ => Ok(Self::unknown(raw, "Failed to decode ToolUseResponseData::Default")),
      },
      _ => Ok(Self::unknown(raw, format!("Unknown ToolUseResponseData variant: {}", variant))),
    }
  }

  fn kind_of() -> surrealdb_types::Kind {
    surrealdb_types::kind!(object)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tool_use_response_data() {
    let data =
      ToolUseResponseData::success(ToolUseResponseData::FilesRead(FilesReadPayload { files: vec![], errors: vec![] }));
    println!("data: {:?}", data);

    let json = serde_json::to_string(&data).unwrap_or_default();
    println!("json: {}", json);
  }

  #[test]
  fn tool_use_response_data_surreal_value_matches_serde_shape() {
    let data = ToolUseResponseData::FilesRead(FilesReadPayload {
      files:  vec![FileReadPayload { path: "src/main.rs".into(), content: "fn main() {}".into() }],
      errors: vec![],
    });

    let json = serde_json::json!({
      "FilesRead": {
        "files": [
          {
            "path": "src/main.rs",
            "content": "fn main() {}"
          }
        ],
        "errors": []
      }
    });
    let surreal = data.into_value();
    let roundtrip_json = Value::from_value(surreal).expect("surreal value should convert back to json");

    assert_eq!(json, roundtrip_json);
  }

  #[test]
  fn tool_use_response_data_surreal_value_falls_back_to_unknown_for_unknown_variant() {
    let raw = serde_json::json!({
      "MemoryDelete": {
        "id": "abc123"
      }
    });

    let parsed = ToolUseResponseData::from_value(raw.clone().into_value()).expect("unknown rows should not fail");

    match parsed {
      ToolUseResponseData::Unknown(payload) => {
        assert_eq!(payload.original_type.as_deref(), Some("MemoryDelete"));
        assert_eq!(payload.raw, raw);
        assert!(!payload.error.is_empty());
      }
      other => panic!("expected unknown payload, got {:?}", other),
    }
  }

  #[test]
  fn tool_use_response_data_surreal_value_falls_back_to_unknown_for_invalid_known_variant_payload() {
    let raw = serde_json::json!({
      "FilesRead": {
        "files": "not-an-array"
      }
    });

    let parsed = ToolUseResponseData::from_value(raw.clone().into_value()).expect("invalid rows should not fail");

    match parsed {
      ToolUseResponseData::Unknown(payload) => {
        assert_eq!(payload.original_type.as_deref(), Some("FilesRead"));
        assert_eq!(payload.raw, raw);
        assert!(!payload.error.is_empty());
      }
      other => panic!("expected unknown payload, got {:?}", other),
    }
  }

  #[test]
  fn tool_use_response_data_surreal_value_decodes_current_plan_create_payload() {
    let raw = serde_json::json!({
      "PlanCreate": {
        "created_at": "2026-03-06T13:20:06.922857+00:00",
        "description": "Plan to add living per-project and app-wide user summary documents, prompt injection for both plus known-projects list, and optional read-only project-scoped memory search.",
        "id": "inject-project-and-user-memory-briefs-into-session-prompts_019cc34e.plan.md",
        "name": "Inject project and user memory briefs into session prompts",
        "updated_at": "2026-03-06T13:20:06.922857+00:00"
      }
    });

    let parsed = ToolUseResponseData::from_value(raw.into_value()).expect("current plan payload should decode");

    match parsed {
      ToolUseResponseData::PlanCreate(payload) => {
        assert_eq!(payload.id, "inject-project-and-user-memory-briefs-into-session-prompts_019cc34e.plan.md");
        assert_eq!(payload.name, "Inject project and user memory briefs into session prompts");
      }
      other => panic!("expected plan create payload, got {:?}", other),
    }
  }

  #[test]
  fn test_working_directories() {
    let working_directories =
      WorkingDirectories::new(vec![PathBuf::from("/home/user/projects"), PathBuf::from("/home/user/documents")]);
    println!("{}", working_directories.pretty_print());
  }
}
