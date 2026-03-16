use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use macros::SurrealEnumValue;
use surrealdb_types::SurrealValue;

use crate::agent::AgentKind;
use crate::models::ReasoningEffort;
use crate::shared::prelude::SurrealId;

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
  specta::Type,
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

// ProviderSettings
#[derive(
  Clone,
  Default,
  Debug,
  PartialEq,
  Eq,
  Hash,
  Ord,
  PartialOrd,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  fake::Dummy,
  SurrealValue,
)]
pub struct ProviderSettings {
  #[serde(default = "default_true")]
  pub use_stream: bool,
}

fn default_true() -> bool {
  true
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

#[derive(Clone, Debug)]
pub struct PlanContext {
  pub id:      String,
  pub content: String,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct LlmModel {
  pub id:                 String,
  pub name:               String,
  pub slug:               String,
  pub context_length:     i64,
  pub supports_reasoning: bool,
  pub provider_slug:      Option<String>,
  pub enabled:            bool,
}

#[derive(Clone, Debug)]
pub struct ChatRequest {
  pub agent_kind:          AgentKind,
  pub llm_model:           LlmModel,
  pub reasoning_effort:    ReasoningEffort,
  pub personality:         String,
  pub instructions:        Option<String>,
  pub session_id:          SurrealId,
  pub working_directories: Vec<PathBuf>,
  pub agent_primer:        Option<String>,
  pub current_skills:      Vec<String>,
  pub plan_context:        Option<PlanContext>,
  pub mcp_details:         HashMap<String, String>,
  pub memory:              String,
}

#[derive(Clone, Debug)]
pub struct PromptParams {
  pub agent_kind:      AgentKind,
  pub personality:     String,
  pub workspace_roots: Vec<PathBuf>,
  pub primer:          Option<String>,
  pub current_skills:  Vec<String>,
  pub plan_context:    Option<PlanContext>,
  pub mcp_details:     HashMap<String, String>,
  pub memory:          String,
}

impl From<ChatRequest> for PromptParams {
  fn from(request: ChatRequest) -> Self {
    Self {
      agent_kind:      request.agent_kind,
      personality:     request.personality,
      workspace_roots: request.working_directories,
      primer:          request.agent_primer,
      current_skills:  request.current_skills,
      plan_context:    request.plan_context,
      mcp_details:     request.mcp_details,
      memory:          request.memory,
    }
  }
}
