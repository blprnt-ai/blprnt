use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use surrealdb::types::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
#[serde(rename_all = "snake_case")]
pub enum SubAgentStatus {
  Spawned,
  Success,
  Failure,
  Timeout,
}

impl Display for SubAgentStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        SubAgentStatus::Spawned => "spawned",
        SubAgentStatus::Success => "success",
        SubAgentStatus::Failure => "failure",
        SubAgentStatus::Timeout => "timeout",
      }
    )
  }
}

impl FromStr for SubAgentStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    match s {
      "spawned" => Ok(SubAgentStatus::Spawned),
      "success" => Ok(SubAgentStatus::Success),
      "failure" => Ok(SubAgentStatus::Failure),
      "timeout" => Ok(SubAgentStatus::Timeout),
      _ => Err(anyhow::Error::msg(format!("invalid subagent status: {}", s))),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SubAgentMetadata {
  pub tokens_used: u32,
  pub duration_ms: u64,
  pub model_used:  String,
  pub parent_id:   Uuid,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
  pub name:          String,
  pub content:       String,
  pub artifact_type: ArtifactType,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
  Text,
  Json,
  Code,
  File,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SubAgentResult {
  pub status: SubAgentStatus,
  pub output: String,
}

impl SubAgentResult {
  pub fn success(output: String) -> Self {
    Self { status: SubAgentStatus::Success, output }
  }

  pub fn failure(output: String) -> Self {
    Self { status: SubAgentStatus::Failure, output }
  }

  pub fn timeout(output: String) -> Self {
    Self { status: SubAgentStatus::Timeout, output }
  }
}
