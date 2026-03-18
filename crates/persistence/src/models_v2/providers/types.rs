use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

pub const PROVIDERS_TABLE: &str = "providers";

// TODO: Replace id with ProviderId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProviderId(SurrealId);

impl DbId for ProviderId {
  fn id(self) -> SurrealId {
    self.0
  }

  fn inner(self) -> RecordId {
    self.0.inner()
  }
}

impl From<Uuid> for ProviderId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(PROVIDERS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProviderId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}
