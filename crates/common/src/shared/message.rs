use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use serde_json::Value;
use serde_json::json;
use surrealdb_types::SurrealValue;

use crate::agent::prelude::*;
use crate::errors::ErrorEvent;
use crate::session_dispatch::prelude::SubagentDetails;
use crate::shared::prelude::*;
use crate::tools::ToolUseResponse;

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
  SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
  System,
  #[default]
  User,
  Assistant,
}

impl Display for MessageRole {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::System => "system",
        Self::User => "user",
        Self::Assistant => "assistant",
      }
    )
  }
}

impl From<MessageRole> for serde_json::Value {
  fn from(role: MessageRole) -> Self {
    serde_json::Value::String(role.to_string())
  }
}

impl FromStr for MessageRole {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    Ok(match s {
      "system" => Self::System,
      "user" => Self::User,
      "assistant" => Self::Assistant,
      _ => unreachable!(),
    })
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct SessionSummaryText {
  pub role:        MessageRole,
  pub content:     String,
  pub is_thinking: bool,
  pub tool_use:    Option<Value>,
  pub tool_result: Option<Value>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageText {
  pub text:      String,
  pub signature: Option<String>,
}

impl MessageText {
  pub fn get_text(&self) -> String {
    self.text.clone()
  }
}

impl From<MessageText> for MessageContent {
  fn from(text: MessageText) -> Self {
    Self::Text(text)
  }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageImage64 {
  pub image_64:   String,
  pub media_type: String,
}

impl FromStr for MessageImage64 {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let media_type = s
      .split_once(";")
      .map(|(media_type, _)| media_type.to_string())
      .and_then(|m| m.split_once(":").map(|(_, media_type)| media_type.to_string()))
      .unwrap_or_default();

    tracing::info!("media_type: {}", media_type);

    Ok(Self { image_64: s.to_string(), media_type })
  }
}

impl From<MessageImage64> for MessageContent {
  fn from(image_64: MessageImage64) -> Self {
    Self::Image64(image_64)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageThinking {
  pub thinking:        String,
  pub signature:       String,
  pub source_provider: Option<Provider>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageToolUse {
  pub id:               String,
  pub tool_id:          ToolId,
  pub input:            Value,
  pub subagent_details: Option<SubagentDetails>,
  pub signature:        Option<String>,
}

impl MessageToolUse {
  pub fn into_llm_payload(&self) -> Value {
    json!({
      "tool_id": self.tool_id,
      "input": self.input,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageToolResult {
  pub tool_use_id: String,
  pub content:     ToolUseResponse,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct Signal {
  pub id:      Option<SurrealId>,
  pub message: String,
  pub source:  Option<HistoryMessageSource>,
  pub error:   Option<ErrorEvent>,
}

impl Signal {
  pub fn to_info(message: String) -> MessageContent {
    MessageContent::Info(Signal { message, ..Default::default() })
  }

  pub fn to_warning(message: String) -> MessageContent {
    MessageContent::Warning(Signal { message, ..Default::default() })
  }

  pub fn to_error(message: String, source: Option<HistoryMessageSource>, error: Option<ErrorEvent>) -> MessageContent {
    MessageContent::Error(Signal { message, source, error, ..Default::default() })
  }

  pub fn with_id(id: SurrealId) -> Self {
    Self { id: Some(id), ..Default::default() }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageWebSearch {
  pub url:         String,
  pub title:       String,
  pub start_index: usize,
  pub end_index:   usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
  Text(MessageText),
  #[serde(rename = "image64")]
  Image64(MessageImage64),
  Thinking(MessageThinking),
  ToolUse(MessageToolUse),
  ToolResult(MessageToolResult),
  TokenUsage(TokenUsage),
  Info(Signal),
  Warning(Signal),
  Error(Signal),
  WebSearch(MessageWebSearch),
}

impl Default for MessageContent {
  fn default() -> Self {
    MessageContent::Text(MessageText::default())
  }
}

impl MessageContent {
  pub fn is_thinking(&self) -> bool {
    matches!(self, MessageContent::Thinking(_))
  }

  pub fn is_text(&self) -> bool {
    matches!(self, MessageContent::Text(_))
  }

  pub fn as_text(&self) -> Option<String> {
    match self {
      MessageContent::Text(text) => Some(text.text.clone()),
      _ => None,
    }
  }

  pub fn is_tool_result(&self) -> bool {
    matches!(self, MessageContent::ToolResult(_))
  }

  pub fn is_tool_use(&self) -> bool {
    matches!(self, MessageContent::ToolUse(_))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MessageDelta {
  pub item_id: String,
  pub delta:   String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReasoningDelta {
  pub item_id: String,
  pub delta:   String,
}
