use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

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

impl From<Uuid> for ProjectId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(PROJECTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProjectId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}
