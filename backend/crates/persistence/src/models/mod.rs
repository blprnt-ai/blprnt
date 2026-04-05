mod auth;
mod employees;
mod issues;
mod projects;
mod providers;
mod runs;
mod telegram;
mod turns;

pub use auth::*;
pub use employees::*;
pub use issues::*;
pub use projects::*;
pub use providers::*;
pub use runs::*;
pub use telegram::*;
use surrealdb_types::SurrealValue;
pub use turns::*;

#[derive(Clone, Debug, serde::Deserialize, SurrealValue)]
pub struct Record {
  pub id: surrealdb_types::RecordId,
}
