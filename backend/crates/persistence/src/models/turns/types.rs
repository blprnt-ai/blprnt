use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use macros::SurrealEnumValue;
use serde_json::Value as JsonValue;
use shared::agent::Provider;
use shared::agent::ToolId;
use shared::tools::ToolUseResponse;
use surrealdb_types::ConversionError;
use surrealdb_types::Kind;
use surrealdb_types::Object as SurrealObject;
use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use surrealdb_types::Value as SurrealDbValue;
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
  pub input:       JsonValue,
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UsageMetrics {
  pub provider:                   Option<Provider>,
  pub model:                      Option<String>,
  pub input_tokens:               Option<u64>,
  pub output_tokens:              Option<u64>,
  pub total_tokens:               Option<u64>,
  pub estimated_cost_usd:         Option<f64>,
  pub has_unavailable_token_data: bool,
  pub has_unavailable_cost_data:  bool,
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

impl SurrealValue for UsageMetrics {
  fn kind_of() -> Kind {
    surrealdb_types::kind!(object | none | null)
  }

  fn is_value(value: &SurrealDbValue) -> bool {
    matches!(value, SurrealDbValue::Object(_) | SurrealDbValue::None | SurrealDbValue::Null)
  }

  fn into_value(self) -> SurrealDbValue {
    SurrealDbValue::Object(surrealdb_types::object! {
      provider: self.provider,
      model: self.model,
      input_tokens: self.input_tokens,
      output_tokens: self.output_tokens,
      total_tokens: self.total_tokens,
      estimated_cost_usd: self.estimated_cost_usd,
      has_unavailable_token_data: self.has_unavailable_token_data,
      has_unavailable_cost_data: self.has_unavailable_cost_data,
    })
  }

  fn from_value(value: SurrealDbValue) -> std::result::Result<Self, surrealdb_types::Error> {
    match value {
      SurrealDbValue::None | SurrealDbValue::Null => Ok(Self::default()),
      SurrealDbValue::Object(mut object) => Ok(Self {
        provider:                   optional_usage_field(&mut object, "provider")?,
        model:                      optional_usage_field(&mut object, "model")?,
        input_tokens:               optional_usage_field(&mut object, "input_tokens")?,
        output_tokens:              optional_usage_field(&mut object, "output_tokens")?,
        total_tokens:               optional_usage_field(&mut object, "total_tokens")?,
        estimated_cost_usd:         optional_usage_field(&mut object, "estimated_cost_usd")?,
        has_unavailable_token_data: boolean_usage_field(&mut object, "has_unavailable_token_data")?,
        has_unavailable_cost_data:  boolean_usage_field(&mut object, "has_unavailable_cost_data")?,
      }),
      other => Err(ConversionError::from_value(Self::kind_of(), &other).into()),
    }
  }
}

fn optional_usage_field<T: SurrealValue>(
  object: &mut SurrealObject,
  key: &str,
) -> std::result::Result<Option<T>, surrealdb_types::Error> {
  match object.remove(key).unwrap_or(SurrealDbValue::None) {
    SurrealDbValue::None | SurrealDbValue::Null => Ok(None),
    value => T::from_value(value).map(Some),
  }
}

fn boolean_usage_field(object: &mut SurrealObject, key: &str) -> std::result::Result<bool, surrealdb_types::Error> {
  match object.remove(key).unwrap_or(SurrealDbValue::None) {
    SurrealDbValue::None | SurrealDbValue::Null => Ok(false),
    value => bool::from_value(value),
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

#[cfg(test)]
mod tests {
  use surrealdb_types::SurrealValue;
  use surrealdb_types::Value;

  use super::ContentsVisibility;
  use super::TurnStep;
  use super::TurnStepContent;
  use super::TurnStepContents;
  use super::TurnStepRole;
  use super::TurnStepStatus;
  use super::TurnStepText;
  use super::UsageMetrics;

  #[test]
  fn usage_metrics_surreal_value_defaults_none_and_null() {
    let none = UsageMetrics::from_value(Value::None).expect("NONE usage should deserialize");
    let null = UsageMetrics::from_value(Value::Null).expect("NULL usage should deserialize");

    for usage in [none, null] {
      assert!(usage.provider.is_none());
      assert!(usage.model.is_none());
      assert!(usage.input_tokens.is_none());
      assert!(usage.output_tokens.is_none());
      assert!(usage.total_tokens.is_none());
      assert!(usage.estimated_cost_usd.is_none());
      assert!(!usage.has_unavailable_token_data);
      assert!(!usage.has_unavailable_cost_data);
    }
  }

  fn sample_step() -> TurnStep {
    TurnStep {
      request:      TurnStepContents {
        contents: vec![TurnStepContent::Text(TurnStepText {
          text:       "Prompt".to_string(),
          signature:  None,
          visibility: ContentsVisibility::Full,
        })],
        role:     TurnStepRole::User,
      },
      response:     TurnStepContents {
        contents: vec![TurnStepContent::Text(TurnStepText {
          text:       "Response".to_string(),
          signature:  None,
          visibility: ContentsVisibility::Full,
        })],
        role:     TurnStepRole::Assistant,
      },
      status:       TurnStepStatus::Completed,
      usage:        UsageMetrics {
        provider:                   Some(shared::agent::Provider::OpenAi),
        model:                      Some("gpt-5".to_string()),
        input_tokens:               Some(12),
        output_tokens:              Some(8),
        total_tokens:               Some(20),
        estimated_cost_usd:         Some(0.01),
        has_unavailable_token_data: false,
        has_unavailable_cost_data:  false,
      },
      created_at:   chrono::Utc::now(),
      completed_at: Some(chrono::Utc::now()),
    }
  }

  #[test]
  fn turn_step_surreal_value_defaults_legacy_none_usage_metrics() {
    let step = sample_step();
    let Value::Object(mut object) = step.into_value() else {
      panic!("turn step should serialize to an object");
    };
    object.insert("usage", Value::None);

    let step = TurnStep::from_value(Value::Object(object)).expect("legacy NONE usage should deserialize");

    assert!(step.usage.provider.is_none());
    assert!(step.usage.model.is_none());
    assert!(step.usage.input_tokens.is_none());
    assert!(step.usage.output_tokens.is_none());
    assert!(step.usage.total_tokens.is_none());
    assert!(step.usage.estimated_cost_usd.is_none());
    assert!(!step.usage.has_unavailable_token_data);
    assert!(!step.usage.has_unavailable_cost_data);
  }
}
