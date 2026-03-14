use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use serde_json::Value;

use crate::errors::SerdeError;

#[derive(
  Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum HistoryMessageSource {
  #[default]
  User,
  Assistant,
  Tool,
  Blprnt,
}

impl Display for HistoryMessageSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::User => "user",
        Self::Assistant => "assistant",
        Self::Tool => "tool",
        Self::Blprnt => "blprnt",
      }
    )
  }
}

impl From<HistoryMessageSource> for Value {
  fn from(source: HistoryMessageSource) -> Self {
    Value::String(source.to_string())
  }
}

impl FromStr for HistoryMessageSource {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    match s {
      "user" => Ok(Self::User),
      "assistant" => Ok(Self::Assistant),
      "tool" => Ok(Self::Tool),
      "blprnt" => Ok(Self::Blprnt),
      _ => unreachable!(),
    }
  }
}

#[derive(
  Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum PartialVisibility {
  Error,
  Warning,
  ToolResult,
  #[default]
  ToolRequest,
}

impl Display for PartialVisibility {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Error => "error",
        Self::Warning => "warning",
        Self::ToolResult => "tool_result",
        Self::ToolRequest => "tool_request",
      }
    )
  }
}

impl FromStr for PartialVisibility {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    match s {
      "error" => Ok(Self::Error),
      "warning" => Ok(Self::Warning),
      "tool_result" => Ok(Self::ToolResult),
      "tool_request" => Ok(Self::ToolRequest),
      _ => unreachable!(),
    }
  }
}

#[derive(
  Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum HistoryVisibility {
  Full,
  #[default]
  User,
  Assistant,
  Partial(PartialVisibility),
  None,
}

impl HistoryVisibility {
  pub fn for_user(&self) -> bool {
    matches!(self, Self::Full | Self::User | Self::Partial(PartialVisibility::ToolRequest))
  }

  pub fn for_assistant(&self) -> bool {
    matches!(
      self,
      Self::Full | Self::Assistant | Self::Partial(PartialVisibility::ToolRequest | PartialVisibility::ToolResult)
    )
  }

  pub fn is_tool_request(&self) -> bool {
    matches!(self, Self::Partial(PartialVisibility::ToolRequest))
  }

  pub fn is_tool_result(&self) -> bool {
    matches!(self, Self::Partial(PartialVisibility::ToolResult))
  }
}

impl Display for HistoryVisibility {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Self::Partial(partial) = self {
      return write!(f, "partial::{}", partial);
    }

    write!(
      f,
      "{}",
      match self {
        Self::Full => "full",
        Self::User => "user",
        Self::Assistant => "assistant",
        Self::None => "none",
        Self::Partial(_) => unreachable!(),
      }
    )
  }
}

impl FromStr for HistoryVisibility {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let prefix = "partial::";
    if let Some(stripped) = s.strip_prefix(prefix) {
      let partial = PartialVisibility::from_str(stripped)
        .map_err(|e| SerdeError::FailedToDeserializeFromPlain(format!("partial visibility: {}", e)))?;
      return Ok(Self::Partial(partial));
    }

    match s {
      "full" => Ok(Self::Full),
      "user" => Ok(Self::User),
      "assistant" => Ok(Self::Assistant),
      "none" => Ok(Self::None),
      _ => unreachable!(),
    }
  }
}

impl From<PartialVisibility> for HistoryVisibility {
  fn from(partial: PartialVisibility) -> Self {
    Self::Partial(partial)
  }
}
