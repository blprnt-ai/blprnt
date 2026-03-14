use std::fmt::Display;

use common::OrderedFloat;
use common::agent::prelude::*;
use common::shared::prelude::MessageRole;
use serde_json::Value;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SystemRequestBody {
  #[serde(rename = "type")]
  pub kind: String,
  pub text: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheControl {
  #[serde(rename = "type")]
  pub kind: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ttl:  Option<CacheControlTtl>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum CacheControlTtl {
  #[serde(rename = "5m")]
  FiveMinutes,
  #[serde(rename = "1h")]
  OneHour,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageRequestBody {
  // For messages and count_tokens
  pub messages: Vec<ClaudeMesssage>,
  pub model:    String,
  pub system:   Vec<SystemRequestBody>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub thinking: Option<AnthropicThinking>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools:    Option<serde_json::Value>,

  // Only for messages
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_tokens:         Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stream:             Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub output_config:      Option<AnthropicOutputConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub temperature:        Option<OrderedFloat<f32>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub context_management: Option<ContextManagement>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cache_control:      Option<CacheControl>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicThinking {
  pub budget_tokens: u32,
  #[serde(rename = "type")]
  pub kind:          String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicOutputConfig {
  pub effort: AnthropicEffort,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnthropicEffort {
  Max,
  High,
  Medium,
  Low,
}

impl From<AnthropicEffort> for AnthropicOutputConfig {
  fn from(effort: AnthropicEffort) -> Self {
    AnthropicOutputConfig { effort }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextManagement {
  pub edits: [ContextManagementEdits; 1],
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextManagementEdits {
  #[serde(rename = "type")]
  pub kind:           String,
  pub trigger:        ContextManagementTrigger,
  pub keep:           ContextManagementKeep,
  pub clear_at_least: ContextManagementClear,
  pub exclude_tools:  Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextManagementTrigger {
  #[serde(rename = "type")]
  pub kind:  String,
  pub value: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextManagementKeep {
  #[serde(rename = "type")]
  pub kind:  String,
  pub value: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContextManagementClear {
  #[serde(rename = "type")]
  pub kind:  String,
  pub value: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClaudeMesssage {
  pub role:    MessageRole,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub content: Vec<ContentPart>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPartText {
  pub text: String,
}

impl From<ContentPartText> for ContentPart {
  fn from(content_part: ContentPartText) -> Self {
    ContentPart::Text(content_part)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPartImage {
  pub source: ContentPartImageKind,
}

impl From<ContentPartImage> for ContentPart {
  fn from(content_part: ContentPartImage) -> Self {
    ContentPart::Image(content_part)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPartImageKind {
  #[serde(rename = "base64")]
  Base64 { data: String, media_type: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPartToolUse {
  pub id:    String,
  pub name:  ToolId,
  pub input: Value,
}

impl From<ContentPartToolUse> for ContentPart {
  fn from(content_part: ContentPartToolUse) -> Self {
    ContentPart::ToolUse(content_part)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPartThinking {
  pub thinking:  String,
  pub signature: String,
}

impl From<ContentPartThinking> for ContentPart {
  fn from(content_part: ContentPartThinking) -> Self {
    ContentPart::Thinking(content_part)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContentPartToolResult {
  pub tool_use_id: String,
  pub content:     String,
}

impl From<ContentPartToolResult> for ContentPart {
  fn from(content_part: ContentPartToolResult) -> Self {
    ContentPart::ToolResult(content_part)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
  Text(ContentPartText),
  Image(ContentPartImage),
  Thinking(ContentPartThinking),
  ToolUse(ContentPartToolUse),
  ToolResult(ContentPartToolResult),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatBasicResponse {
  pub content: Vec<ContentPartText>,
  pub usage:   Usage,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Usage {
  pub input_tokens:  u32,
  pub output_tokens: u32,
}

#[derive(Clone, Debug)]
pub enum Beta {
  Oauth,
  ClaudeCode,
  InterleavedThinking,
  FineGrainedToolStreaming,
  ContextManagement,
  Effort,
}

impl Display for Beta {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Beta::Oauth => write!(f, "oauth-2025-04-20"),
      Beta::ClaudeCode => write!(f, "claude-code-20250219"),
      // Beta::Oauth => write!(f, "oauth-2025-04-20"),
      Beta::InterleavedThinking => write!(f, "interleaved-thinking-2025-05-14"),
      Beta::FineGrainedToolStreaming => write!(f, "fine-grained-tool-streaming-2025-05-14"),
      Beta::ContextManagement => write!(f, "context-management-2025-06-27"),
      Beta::Effort => write!(f, "effort-2025-11-24"),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CountTokensResponse {
  pub input_tokens: u32,
}
