use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const PROVIDERS_TABLE: &str = "providers";

// TODO: Replace id with ProviderId
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct ProviderId(SurrealId);

impl DbId for ProviderId {
  fn id(&self) -> SurrealId {
    self.0.clone()
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
