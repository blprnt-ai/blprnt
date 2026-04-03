use chrono::DateTime;
use chrono::Utc;
use macros::SurrealEnumValue;
use std::fmt::Display;
use std::str::FromStr;
use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::RunId;
use crate::prelude::SurrealId;

pub const TELEGRAM_CONFIGS_TABLE: &str = "telegram_configs";
pub const TELEGRAM_LINKS_TABLE: &str = "telegram_links";
pub const TELEGRAM_LINK_CODES_TABLE: &str = "telegram_link_codes";
pub const TELEGRAM_ISSUE_WATCHES_TABLE: &str = "telegram_issue_watches";
pub const TELEGRAM_MESSAGE_CORRELATIONS_TABLE: &str = "telegram_message_correlations";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramConfigId(pub SurrealId);

impl DbId for TelegramConfigId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TelegramConfigId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TELEGRAM_CONFIGS_TABLE, uuid).into())
  }
}

impl From<Uuid> for TelegramConfigId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TELEGRAM_CONFIGS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for TelegramConfigId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkId(pub SurrealId);

impl DbId for TelegramLinkId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TelegramLinkId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TELEGRAM_LINKS_TABLE, uuid).into())
  }
}

impl From<Uuid> for TelegramLinkId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TELEGRAM_LINKS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for TelegramLinkId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramIssueWatchId(pub SurrealId);

impl DbId for TelegramIssueWatchId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TelegramIssueWatchId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TELEGRAM_ISSUE_WATCHES_TABLE, uuid).into())
  }
}

impl From<Uuid> for TelegramIssueWatchId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TELEGRAM_ISSUE_WATCHES_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for TelegramIssueWatchId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkCodeId(pub SurrealId);

impl DbId for TelegramLinkCodeId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TelegramLinkCodeId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TELEGRAM_LINK_CODES_TABLE, uuid).into())
  }
}

impl From<Uuid> for TelegramLinkCodeId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TELEGRAM_LINK_CODES_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for TelegramLinkCodeId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramMessageCorrelationId(pub SurrealId);

impl DbId for TelegramMessageCorrelationId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for TelegramMessageCorrelationId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(TELEGRAM_MESSAGE_CORRELATIONS_TABLE, uuid).into())
  }
}

impl From<Uuid> for TelegramMessageCorrelationId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(TELEGRAM_MESSAGE_CORRELATIONS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for TelegramMessageCorrelationId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryMode {
  Webhook,
  Polling,
}

impl Display for TelegramDeliveryMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Webhook => write!(f, "webhook"),
      Self::Polling => write!(f, "polling"),
    }
  }
}

impl FromStr for TelegramDeliveryMode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "webhook" => Ok(Self::Webhook),
      "polling" => Ok(Self::Polling),
      _ => Err(anyhow::anyhow!("Invalid telegram delivery mode: {s}")),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TelegramParseMode {
  MarkdownV2,
  Html,
}

impl Display for TelegramParseMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MarkdownV2 => write!(f, "markdown_v2"),
      Self::Html => write!(f, "html"),
    }
  }
}

impl FromStr for TelegramParseMode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "markdown_v2" => Ok(Self::MarkdownV2),
      "html" => Ok(Self::Html),
      _ => Err(anyhow::anyhow!("Invalid telegram parse mode: {s}")),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TelegramLinkStatus {
  Linked,
  Revoked,
}

impl Display for TelegramLinkStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Linked => write!(f, "linked"),
      Self::Revoked => write!(f, "revoked"),
    }
  }
}

impl FromStr for TelegramLinkStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "linked" => Ok(Self::Linked),
      "revoked" => Ok(Self::Revoked),
      _ => Err(anyhow::anyhow!("Invalid telegram link status: {s}")),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TelegramMessageDirection {
  Inbound,
  Outbound,
}

impl Display for TelegramMessageDirection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Inbound => write!(f, "inbound"),
      Self::Outbound => write!(f, "outbound"),
    }
  }
}

impl FromStr for TelegramMessageDirection {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "inbound" => Ok(Self::Inbound),
      "outbound" => Ok(Self::Outbound),
      _ => Err(anyhow::anyhow!("Invalid telegram message direction: {s}")),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum TelegramCorrelationKind {
  Unknown,
  LinkCode,
  Issue,
  Run,
  Notification,
}

impl Display for TelegramCorrelationKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Unknown => write!(f, "unknown"),
      Self::LinkCode => write!(f, "link_code"),
      Self::Issue => write!(f, "issue"),
      Self::Run => write!(f, "run"),
      Self::Notification => write!(f, "notification"),
    }
  }
}

impl FromStr for TelegramCorrelationKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "unknown" => Ok(Self::Unknown),
      "link_code" => Ok(Self::LinkCode),
      "issue" => Ok(Self::Issue),
      "run" => Ok(Self::Run),
      "notification" => Ok(Self::Notification),
      _ => Err(anyhow::anyhow!("Invalid telegram correlation kind: {s}")),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TelegramNotificationPreferences {
  pub issue_notifications: bool,
  pub run_notifications:   bool,
}

impl Default for TelegramNotificationPreferences {
  fn default() -> Self {
    Self { issue_notifications: true, run_notifications: true }
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramMessageCorrelationPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub issue_id:     Option<Option<IssueId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub run_id:       Option<Option<RunId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub employee_id:  Option<Option<EmployeeId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:   Option<DateTime<Utc>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text_preview: Option<Option<String>>,
}