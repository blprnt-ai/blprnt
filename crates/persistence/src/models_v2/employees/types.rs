use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use common::shared::prelude::DbId;
use common::shared::prelude::Provider;
use common::shared::prelude::SurrealId;
use macros::SurrealEnumValue;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

pub const EMPLOYEES_TABLE: &str = "employees";

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeId(SurrealId);

impl DbId for EmployeeId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for EmployeeId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(EMPLOYEES_TABLE, uuid).into())
  }
}

impl From<RecordId> for EmployeeId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeRole {
  Owner,
  Ceo,
  Custom(String),
}

impl EmployeeRole {
  pub fn is_owner(&self) -> bool {
    matches!(self, EmployeeRole::Owner)
  }

  pub fn is_ceo(&self) -> bool {
    matches!(self, EmployeeRole::Ceo)
  }
}

impl Display for EmployeeRole {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeRole::Owner => write!(f, "owner"),
      EmployeeRole::Ceo => write!(f, "ceo"),
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
      _ => Ok(EmployeeRole::Custom(s.to_string())),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
  Idle,
  Running,
  Terminated,
}

impl Display for EmployeeStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeStatus::Idle => write!(f, "idle"),
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
      "running" => Ok(EmployeeStatus::Running),
      "terminated" => Ok(EmployeeStatus::Terminated),
      _ => Err(anyhow::anyhow!("Invalid employee status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeProviderConfig {
  pub provider: Provider,
  pub slug:     String,
}

impl Default for EmployeeProviderConfig {
  fn default() -> Self {
    Self { provider: Provider::Mock, slug: String::new() }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRuntimeConfig {
  pub heartbeat_interval_sec: i32,
  pub heartbeat_prompt:       String,
  pub wake_on_demand:         bool,
  pub max_concurrent_runs:    i32,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeePermissions {
  pub(super) can_hire:            bool,
  pub(super) can_update_employee: bool,
}
