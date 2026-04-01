use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const PROJECTS_TABLE: &str = "projects";

// TODO: Replace id with ProjectId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProjectId(SurrealId);

impl DbId for ProjectId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for ProjectId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(PROJECTS_TABLE, uuid).into())
  }
}

impl From<Uuid> for ProjectId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(PROJECTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProjectId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

impl ts_rs::TS for ProjectId {
  type OptionInnerType = Self;
  type WithoutGenerics = Self;

  fn name(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn inline(_: &ts_rs::Config) -> String {
    "string".to_string()
  }

  fn decl(_: &ts_rs::Config) -> String {
    "type ProjectId = string;".to_string()
  }
}
