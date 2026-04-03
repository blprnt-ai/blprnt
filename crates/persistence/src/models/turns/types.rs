use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use macros::SurrealEnumValue;
use serde_json::Value;
use shared::agent::Provider;
use shared::agent::ToolId;
use shared::tools::ToolUseResponse;
use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const TURNS_TABLE: &str = "turns";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TurnId(SurrealId);

impl DbId for TurnId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TurnId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TURNS_TABLE, uuid).into())
  }
}

impl From<Uuid> for TurnId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(TURNS_TABLE, uuid).into())
  }
}

impl From<RecordId> for TurnId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ContentsVisibility {
  Full,
  User,
  Assistant,
}

impl Display for ContentsVisibility {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ContentsVisibility::Full => write!(f, "full"),
      ContentsVisibility::User => write!(f, "user"),
      ContentsVisibility::Assistant => write!(f, "assistant"),
    }
  }
}

impl FromStr for ContentsVisibility {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "full" => Ok(ContentsVisibility::Full),
      "user" => Ok(ContentsVisibility::User),
      "assistant" => Ok(ContentsVisibility::Assistant),
      _ => Err(anyhow::anyhow!("Invalid turn step visibility: {}", s)),
    }
  }
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TurnStepRole {
  User,
  Assistant,
}

impl Display for TurnStepRole {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TurnStepRole::User => write!(f, "user"),
      TurnStepRole::Assistant => write!(f, "assistant"),
    }
  }
}

impl FromStr for TurnStepRole {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "user" => Ok(TurnStepRole::User),
      "assistant" => Ok(TurnStepRole::Assistant),
      _ => Err(anyhow::anyhow!("Invalid turn step role: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepText {
  pub text:       String,
  pub signature:  Option<String>,
  pub visibility: ContentsVisibility,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepImage {
  pub blob:       String,
  pub media_kind: String,
  pub visibility: ContentsVisibility,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepThinking {
  pub thinking:   String,
  pub signature:  String,
  pub visibility: ContentsVisibility,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepToolUse {
  pub tool_use_id: String,
  #[schema(value_type = String)]
  pub tool_id:     ToolId,
  #[schema(value_type = Object)]
  pub input:       Value,
  pub visibility:  ContentsVisibility,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepToolResult {
  pub tool_use_id: String,
  #[schema(value_type = String)]
  pub tool_id:     ToolId,
  pub content:     ToolUseResponse,
  pub visibility:  ContentsVisibility,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub enum TurnStepContent {
  Text(TurnStepText),
  Image64(TurnStepImage),
  Thinking(TurnStepThinking),
  ToolUse(TurnStepToolUse),
  ToolResult(TurnStepToolResult),
}

impl TurnStepContent {
  pub fn visibility(&self) -> ContentsVisibility {
    match self {
      TurnStepContent::Text(text) => text.visibility,
      TurnStepContent::Image64(image) => image.visibility,
      TurnStepContent::Thinking(thinking) => thinking.visibility,
      TurnStepContent::ToolUse(tool_use) => tool_use.visibility,
      TurnStepContent::ToolResult(tool_result) => tool_result.visibility,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStepContents {
  pub contents: Vec<TurnStepContent>,
  pub role:     TurnStepRole,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TurnStepStatus {
  InProgress,
  Completed,
  Failed,
}

impl Display for TurnStepStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TurnStepStatus::InProgress => write!(f, "in_progress"),
      TurnStepStatus::Completed => write!(f, "completed"),
      TurnStepStatus::Failed => write!(f, "failed"),
    }
  }
}

impl FromStr for TurnStepStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "in_progress" => Ok(TurnStepStatus::InProgress),
      "completed" => Ok(TurnStepStatus::Completed),
      "failed" => Ok(TurnStepStatus::Failed),
      _ => Err(anyhow::anyhow!("Invalid turn step status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UsageMetrics {
  pub provider:                    Option<Provider>,
  pub model:                       Option<String>,
  pub input_tokens:                Option<u64>,
  pub output_tokens:               Option<u64>,
  pub total_tokens:                Option<u64>,
  pub estimated_cost_usd:          Option<f64>,
  pub has_unavailable_token_data:  bool,
  pub has_unavailable_cost_data:   bool,
}

impl Default for UsageMetrics {
  fn default() -> Self {
    Self {
      provider:                   None,
      model:                      None,
      input_tokens:               None,
      output_tokens:              None,
      total_tokens:               None,
      estimated_cost_usd:         None,
      has_unavailable_token_data: false,
      has_unavailable_cost_data:  false,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnStep {
  pub request:      TurnStepContents,
  pub response:     TurnStepContents,
  pub status:       TurnStepStatus,
  #[serde(default)]
  pub usage:        UsageMetrics,
  pub created_at:   DateTime<Utc>,
  pub completed_at: Option<DateTime<Utc>>,
}
