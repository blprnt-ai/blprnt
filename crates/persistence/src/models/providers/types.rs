use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

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

impl From<SurrealUuid> for ProviderId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(PROVIDERS_TABLE, uuid).into())
  }
}

impl From<Uuid> for ProviderId {
  fn from(uuid: Uuid) -> Self {
    let uuid = RecordIdKey::Uuid(uuid.into());
    Self(RecordId::new(PROVIDERS_TABLE, uuid).into())
  }
}

impl From<RecordId> for ProviderId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}
