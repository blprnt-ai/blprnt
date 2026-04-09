use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const MCP_SERVERS_TABLE: &str = "mcp_servers";
pub const RUN_ENABLED_MCP_SERVERS_TABLE: &str = "run_enabled_mcp_servers";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct McpServerId(SurrealId);

impl DbId for McpServerId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for McpServerId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(MCP_SERVERS_TABLE, uuid).into())
  }
}

impl From<Uuid> for McpServerId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(MCP_SERVERS_TABLE, uuid).into())
  }
}

impl From<RecordId> for McpServerId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

impl ts_rs::TS for McpServerId {
  type OptionInnerType = Self;
  type WithoutGenerics = Self;

  fn name(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn inline(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn decl(_: &ts_rs::Config) -> String {
    "type McpServerId = string;".to_string()
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunEnabledMcpServerId(SurrealId);

impl DbId for RunEnabledMcpServerId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for RunEnabledMcpServerId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(RUN_ENABLED_MCP_SERVERS_TABLE, uuid).into())
  }
}

impl From<Uuid> for RunEnabledMcpServerId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(RUN_ENABLED_MCP_SERVERS_TABLE, uuid).into())
  }
}

impl From<RecordId> for RunEnabledMcpServerId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}
