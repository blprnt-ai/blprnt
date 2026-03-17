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
  ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Provider {
  #[default]
  Anthropic,
  #[serde(rename = "openai")]
  OpenAi,
  OpenRouter,
  Mock,
  ClaudeCode,
  Codex,
}

impl Provider {
  pub fn is_claude_or_codex(&self) -> bool {
    matches!(self, Self::ClaudeCode | Self::Codex)
  }
}

impl Display for Provider {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Anthropic => write!(f, "anthropic"),
      Self::ClaudeCode => write!(f, "claude_code"),
      Self::OpenAi => write!(f, "openai"),
      Self::Codex => write!(f, "codex"),
      Self::OpenRouter => write!(f, "openrouter"),
      Self::Mock => write!(f, "mock"),
    }
  }
}

impl FromStr for Provider {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let s = s.trim_matches('"');

    match s {
      "anthropic" => Ok(Self::Anthropic),
      "claude_code" => Ok(Self::ClaudeCode),
      "openai" => Ok(Self::OpenAi),
      "codex" => Ok(Self::Codex),
      "openrouter" => Ok(Self::OpenRouter),

      _ => Ok(Self::Mock),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiOauthToken {
  pub access_token:  String,
  pub refresh_token: String,
  pub expires_at_ms: u64,
  pub account_id:    Option<String>,
}

impl From<OpenAiOauthToken> for OauthToken {
  fn from(token: OpenAiOauthToken) -> Self {
    OauthToken::OpenAi(token)
  }
}

impl From<OpenAiOauthToken> for BlprntCredentials {
  fn from(token: OpenAiOauthToken) -> Self {
    BlprntCredentials::OauthToken(OauthToken::OpenAi(token))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicOauthToken {
  pub access_token:  String,
  pub refresh_token: String,
  pub expires_at_ms: u64,
}

impl From<AnthropicOauthToken> for OauthToken {
  fn from(token: AnthropicOauthToken) -> Self {
    OauthToken::Anthropic(token)
  }
}

impl From<AnthropicOauthToken> for BlprntCredentials {
  fn from(token: AnthropicOauthToken) -> Self {
    BlprntCredentials::OauthToken(OauthToken::Anthropic(token))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthToken {
  #[serde(rename = "openai")]
  OpenAi(OpenAiOauthToken),
  Anthropic(AnthropicOauthToken),
}

impl From<OauthToken> for BlprntCredentials {
  fn from(token: OauthToken) -> Self {
    match token {
      OauthToken::OpenAi(token) => BlprntCredentials::OauthToken(token.into()),
      OauthToken::Anthropic(token) => BlprntCredentials::OauthToken(token.into()),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum BlprntCredentials {
  ApiKey(String),
  OauthToken(OauthToken),
  Unknown,
}
