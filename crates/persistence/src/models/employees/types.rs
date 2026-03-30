use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use macros::SurrealEnumValue;
use shared::agent::Provider;
use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const EMPLOYEES_TABLE: &str = "employees";

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct EmployeeId(pub SurrealId);

impl DbId for EmployeeId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for EmployeeId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(EMPLOYEES_TABLE, uuid).into())
  }
}

impl From<Uuid> for EmployeeId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(EMPLOYEES_TABLE, uuid).into())
  }
}

impl From<RecordId> for EmployeeId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

impl ts_rs::TS for EmployeeId {
  type OptionInnerType = Self;
  type WithoutGenerics = Self;

  fn name(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn inline(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn decl(_: &ts_rs::Config) -> String {
    "type EmployeeId = string;".to_string()
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeKind {
  #[default]
  Agent,
  Person,
}

impl EmployeeKind {
  pub fn is_agent(&self) -> bool {
    matches!(self, EmployeeKind::Agent)
  }

  pub fn is_person(&self) -> bool {
    matches!(self, EmployeeKind::Person)
  }
}

impl Display for EmployeeKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeKind::Agent => write!(f, "agent"),
      EmployeeKind::Person => write!(f, "person"),
    }
  }
}

impl FromStr for EmployeeKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "agent" => Ok(EmployeeKind::Agent),
      "person" => Ok(EmployeeKind::Person),
      _ => Err(anyhow::anyhow!("Invalid employee kind: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeRole {
  Owner,
  Ceo,
  Manager,
  Staff,
  Custom(String),
}

impl EmployeeRole {
  pub fn is_owner(&self) -> bool {
    matches!(self, EmployeeRole::Owner)
  }

  pub fn is_ceo(&self) -> bool {
    matches!(self, EmployeeRole::Ceo)
  }

  pub fn can_hire(&self) -> bool {
    matches!(self, EmployeeRole::Owner | EmployeeRole::Ceo | EmployeeRole::Manager)
  }

  pub fn can_hire_role(&self, role: &EmployeeRole) -> bool {
    match self {
      EmployeeRole::Owner => !role.is_owner(),
      EmployeeRole::Ceo => matches!(role, EmployeeRole::Manager | EmployeeRole::Staff),
      EmployeeRole::Manager => matches!(role, EmployeeRole::Staff),
      EmployeeRole::Staff | EmployeeRole::Custom(_) => false,
    }
  }

  pub fn can_update_employee(&self) -> bool {
    matches!(self, EmployeeRole::Owner | EmployeeRole::Ceo)
  }
}

impl Display for EmployeeRole {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeRole::Owner => write!(f, "owner"),
      EmployeeRole::Ceo => write!(f, "ceo"),
      EmployeeRole::Manager => write!(f, "manager"),
      EmployeeRole::Staff => write!(f, "staff"),
      EmployeeRole::Custom(role) => write!(f, "{}", role),
    }
  }
}

impl FromStr for EmployeeRole {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "owner" => Ok(EmployeeRole::Owner),
      "ceo" => Ok(EmployeeRole::Ceo),
      "manager" => Ok(EmployeeRole::Manager),
      "staff" => Ok(EmployeeRole::Staff),
      _ => Ok(EmployeeRole::Custom(s.to_string())),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
  Idle,
  Paused,
  Running,
  Terminated,
}

impl Display for EmployeeStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeStatus::Idle => write!(f, "idle"),
      EmployeeStatus::Paused => write!(f, "paused"),
      EmployeeStatus::Running => write!(f, "running"),
      EmployeeStatus::Terminated => write!(f, "terminated"),
    }
  }
}

impl FromStr for EmployeeStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "idle" => Ok(EmployeeStatus::Idle),
      "paused" => Ok(EmployeeStatus::Paused),
      "running" => Ok(EmployeeStatus::Running),
      "terminated" => Ok(EmployeeStatus::Terminated),
      _ => Err(anyhow::anyhow!("Invalid employee status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct EmployeeProviderConfig {
  pub provider: Provider,
  pub slug:     String,
}

impl Default for EmployeeProviderConfig {
  fn default() -> Self {
    Self { provider: Provider::Mock, slug: String::new() }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct EmployeeSkillRef {
  pub name: String,
  pub path: String,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct EmployeeRuntimeConfig {
  #[ts(type = "number")]
  pub heartbeat_interval_sec: i64,
  pub heartbeat_prompt:       String,
  pub wake_on_demand:         bool,
  #[ts(type = "number")]
  pub max_concurrent_runs:    i64,
  pub skill_stack:            Vec<EmployeeSkillRef>,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct EmployeePermissions {
  pub(super) can_hire:            bool,
  pub(super) can_update_employee: bool,
}

impl EmployeePermissions {
  pub fn new(can_hire: bool, can_update_employee: bool) -> Self {
    Self { can_hire, can_update_employee }
  }
}
