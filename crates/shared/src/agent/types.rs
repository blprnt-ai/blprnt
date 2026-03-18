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
  schemars::JsonSchema,
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

#[derive(
  Clone,
  Copy,
  Default,
  Debug,
  PartialEq,
  Eq,
  Hash,
  Ord,
  PartialOrd,
  serde::Serialize,
  serde::Deserialize,
  schemars::JsonSchema,
  SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
  #[default]
  Anthropic,
  #[serde(rename = "openai")]
  OpenAi,
  OpenRouter,
  Mock,
  AnthropicFnf,
  #[serde(rename = "openai_fnf")]
  OpenAiFnf,
  Blprnt,
}

impl Provider {
  pub fn all() -> Vec<Provider> {
    vec![Self::Anthropic, Self::AnthropicFnf, Self::OpenAi, Self::OpenAiFnf, Self::OpenRouter, Self::Mock]
  }

  pub fn is_fnf(&self) -> bool {
    matches!(self, Self::AnthropicFnf | Self::OpenAiFnf)
  }
}

impl Display for Provider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Anthropic => write!(f, "anthropic"),
      Self::AnthropicFnf => write!(f, "anthropic_fnf"),
      Self::OpenAi => write!(f, "openai"),
      Self::OpenAiFnf => write!(f, "openai_fnf"),
      Self::OpenRouter => write!(f, "openrouter"),
      Self::Mock => write!(f, "mock"),
      Self::Blprnt => write!(f, "blprnt"),
    }
  }
}

impl FromStr for Provider {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let s = s.trim_matches('"');

    match s {
      "anthropic" => Ok(Self::Anthropic),
      "anthropic_fnf" => Ok(Self::AnthropicFnf),
      "openai" => Ok(Self::OpenAi),
      "openai_fnf" => Ok(Self::OpenAiFnf),
      "openrouter" => Ok(Self::OpenRouter),
      "blprnt" => Ok(Self::Blprnt),
      _ => Ok(Self::Mock),
    }
  }
}
