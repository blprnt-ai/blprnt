use std::fmt::Display;
use std::str::FromStr;

use serde_json::Value;
use surrealdb_types::SurrealValue;

use crate::errors::SerdeError;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ToolId {
  #[serde(rename = "files_read")]
  FilesRead,
  #[serde(rename = "apply_patch")]
  ApplyPatch,

  #[serde(rename = "shell")]
  Shell,

  #[serde(rename = "terminal")]
  Terminal,

  #[serde(rename = "ask_question")]
  AskQuestion,

  #[serde(rename = "plan_create")]
  PlanCreate,
  #[serde(rename = "plan_list")]
  PlanList,
  #[serde(rename = "plan_get")]
  PlanGet,
  #[serde(rename = "plan_update")]
  PlanUpdate,
  #[serde(rename = "plan_delete")]
  PlanDelete,

  #[serde(rename = "primer_get")]
  PrimerGet,
  #[serde(rename = "primer_update")]
  PrimerUpdate,

  #[serde(rename = "memory_write")]
  MemoryWrite,
  #[serde(rename = "memory_search")]
  MemorySearch,

  #[serde(rename = "rg")]
  Rg,

  #[serde(rename = "subagent")]
  SubAgent,

  #[serde(rename = "web_search")]
  WebSearch,

  #[serde(rename = "list_skills")]
  ListSkills,
  #[serde(rename = "apply_skill")]
  ApplySkill,
  #[serde(rename = "get_reference")]
  GetReference,
  #[serde(rename = "skill_script")]
  SkillScript,

  #[serde(rename = "mcp")]
  Mcp(String),

  #[serde(rename = "unknown")]
  Unknown(String),
}

impl SurrealValue for ToolId {
  fn into_value(self) -> surrealdb_types::Value {
    self.to_string().into_value()
  }

  fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb::Error> {
    let s = String::from_value(value)?;
    ToolId::try_from(s).map_err(|e| {
      surrealdb::Error::serialization(e.to_string(), Some(surrealdb_types::SerializationError::Deserialization))
    })
  }

  fn kind_of() -> surrealdb_types::Kind {
    surrealdb_types::Kind::String
  }
}

impl TryFrom<String> for ToolId {
  type Error = SerdeError;

  fn try_from(value: String) -> Result<Self, SerdeError> {
    if value.starts_with("mcp__") {
      Ok(ToolId::Mcp(value))
    }
    // Legacy serialization format, kept for backwards compatibility
    else if value.starts_with("unknown: mcp__") {
      Ok(ToolId::Mcp(value.split("unknown: mcp__").nth(1).unwrap_or("").to_string()))
    } else if value.starts_with("unknown: ") {
      Ok(ToolId::Unknown(value.split("unknown: ").nth(1).unwrap_or("").to_string()))
    }
    // New serialization format
    else if value.starts_with("unknown_mcp__") {
      Ok(ToolId::Mcp(value.split("unknown_mcp__").nth(1).unwrap_or("").to_string()))
    } else if value.starts_with("unknown_") {
      Ok(ToolId::Unknown(value.split("unknown_").nth(1).unwrap_or("").to_string()))
    }
    // Alias for apply_skill
    else if value == "apply_skill" || value == "get_skill" {
      Ok(ToolId::ApplySkill)
    }
    // Alias for rg
    else if value == "rg_search" || value == "rg" {
      Ok(ToolId::Rg)
    } else if value == "bash" || value == "shell" {
      Ok(ToolId::Shell)
    } else {
      match serde_json::from_str::<ToolId>(&value) {
        Ok(tool_id) => Ok(tool_id),
        Err(_) => match serde_plain::from_str(&value) {
          Ok(tool_id) => Ok(tool_id),
          Err(_) => Ok(ToolId::Unknown(value)),
        },
      }
    }
  }
}

impl Display for ToolId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ToolId::FilesRead => write!(f, "files_read"),
      ToolId::ApplyPatch => write!(f, "apply_patch"),
      ToolId::Shell => write!(f, "shell"),
      ToolId::Terminal => write!(f, "terminal"),
      ToolId::AskQuestion => write!(f, "ask_question"),
      ToolId::PlanCreate => write!(f, "plan_create"),
      ToolId::PlanList => write!(f, "plan_list"),
      ToolId::PlanGet => write!(f, "plan_get"),
      ToolId::PlanUpdate => write!(f, "plan_update"),
      ToolId::PlanDelete => write!(f, "plan_delete"),
      ToolId::PrimerGet => write!(f, "primer_get"),
      ToolId::PrimerUpdate => write!(f, "primer_update"),
      ToolId::MemoryWrite => write!(f, "memory_write"),
      ToolId::MemorySearch => write!(f, "memory_search"),
      ToolId::Rg => write!(f, "rg"),
      ToolId::SubAgent => write!(f, "subagent"),
      ToolId::WebSearch => write!(f, "web_search"),
      ToolId::ListSkills => write!(f, "list_skills"),
      ToolId::ApplySkill => write!(f, "apply_skill"),
      ToolId::GetReference => write!(f, "get_reference"),
      ToolId::SkillScript => write!(f, "skill_script"),
      ToolId::Mcp(name) => write!(f, "{}", name),
      ToolId::Unknown(name) => write!(f, "unknown_{}", name),
    }
  }
}

impl FromStr for ToolId {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    ToolId::try_from(s.to_string()).map_err(|e| anyhow::Error::msg(e.to_string()))
  }
}

impl TryFrom<Value> for ToolId {
  type Error = anyhow::Error;

  fn try_from(value: Value) -> anyhow::Result<Self> {
    serde_json::from_value(value).map_err(|e| SerdeError::FailedToDeserializeFromJson(e.to_string()).into())
  }
}

#[cfg(test)]
mod tests {
  use super::ToolId;

  #[test]
  fn try_from_string_parses_mcp_runtime_name_as_unknown() {
    let parsed = ToolId::try_from("mcp__server-a__tool-x".to_string()).expect("must parse");
    assert_eq!(parsed, ToolId::Mcp("mcp__server-a__tool-x".to_string()));
  }

  #[test]
  fn try_from_string_keeps_alias_behavior_for_non_mcp() {
    let shell = ToolId::try_from("bash".to_string()).expect("must parse alias");
    assert_eq!(shell, ToolId::Shell);
  }
}
