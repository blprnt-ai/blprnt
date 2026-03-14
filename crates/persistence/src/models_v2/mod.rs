use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;

mod projects;
pub use projects::*;

mod sessions;
pub use sessions::*;

mod messages;
pub use messages::*;

mod providers;
pub use providers::*;

#[derive(Clone, Debug, serde::Deserialize, SurrealValue)]
pub struct Record {
  pub id: RecordId,
}
