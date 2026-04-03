use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::RecordId;
use surrealdb_types::RecordIdKey;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid as SurrealUuid;
use uuid::Uuid;

use crate::prelude::DbId;
use crate::prelude::SurrealId;

pub const LOGIN_CREDENTIALS_TABLE: &str = "login_credentials";
pub const AUTH_SESSIONS_TABLE: &str = "auth_sessions";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct LoginCredentialId(pub SurrealId);

impl DbId for LoginCredentialId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for LoginCredentialId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(LOGIN_CREDENTIALS_TABLE, uuid).into())
  }
}

impl From<Uuid> for LoginCredentialId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(LOGIN_CREDENTIALS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for LoginCredentialId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct AuthSessionId(pub SurrealId);

impl DbId for AuthSessionId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<SurrealUuid> for AuthSessionId {
  fn from(uuid: SurrealUuid) -> Self {
    Self(RecordId::new(AUTH_SESSIONS_TABLE, uuid).into())
  }
}

impl From<Uuid> for AuthSessionId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(AUTH_SESSIONS_TABLE, RecordIdKey::Uuid(uuid.into())).into())
  }
}

impl From<RecordId> for AuthSessionId {
  fn from(id: RecordId) -> Self {
    Self(id.into())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AuthSessionMetadata {
  pub issued_at: DateTime<Utc>,
  pub expires_at: DateTime<Utc>,
}