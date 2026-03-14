use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;

#[derive(
  Clone,
  Copy,
  Default,
  Debug,
  PartialEq,
  Eq,
  Hash,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
  SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum AgentKind {
  #[default]
  #[schemars(skip)]
  Crew,
  Planner,
  Executor,
  Verifier,
  Researcher,
  Designer,
}

impl Display for AgentKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AgentKind::Crew => write!(f, "crew"),
      AgentKind::Planner => write!(f, "planner"),
      AgentKind::Executor => write!(f, "executor"),
      AgentKind::Verifier => write!(f, "verifier"),
      AgentKind::Researcher => write!(f, "researcher"),
      AgentKind::Designer => write!(f, "designer"),
    }
  }
}

impl FromStr for AgentKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    match s {
      "crew" => Ok(AgentKind::Crew),
      "planner" => Ok(AgentKind::Planner),
      "executor" => Ok(AgentKind::Executor),
      "verifier" => Ok(AgentKind::Verifier),
      "researcher" => Ok(AgentKind::Researcher),
      "designer" => Ok(AgentKind::Designer),
      _ => Err(anyhow::Error::msg(format!("invalid agent kind: {}", s))),
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
  #[default]
  Solo,
  Crew,
}
