use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

pub const PROJECTS_TABLE: &str = "projects";

// TODO: Replace id with ProjectId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectId(SurrealId);

impl DbId for ProjectId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
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
