use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue)]
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
