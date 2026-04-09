use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use anyhow::Result;
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;
use macros::SurrealEnumValue;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const MINIONS_TABLE: &str = "minions";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct MinionId(SurrealId);

impl DbId for MinionId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for MinionId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(MINIONS_TABLE, uuid).into())
  }
}

impl From<Uuid> for MinionId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(MINIONS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for MinionId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

impl ts_rs::TS for MinionId {
  type OptionInnerType = Self;
  type WithoutGenerics = Self;

  fn name(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn inline(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn decl(_: &ts_rs::Config) -> String {
    "type MinionId = string;".to_string()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealEnumValue, ts_rs::TS, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MinionSource {
  System,
  Custom,
}

impl Display for MinionSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MinionSource::System => write!(f, "system"),
      MinionSource::Custom => write!(f, "custom"),
    }
  }
}

impl FromStr for MinionSource {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "system" => Ok(MinionSource::System),
      "custom" => Ok(MinionSource::Custom),
      _ => Err(anyhow::anyhow!("Invalid minion source: {}", s)),
    }
  }
}