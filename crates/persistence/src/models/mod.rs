mod employees;
mod issues;
mod projects;
mod providers;
mod runs;
mod turns;

pub use employees::*;
pub use issues::*;
pub use projects::*;
pub use providers::*;
pub use runs::*;
use surrealdb_types::SurrealValue;
pub use turns::*;

#[derive(Clone, Debug, serde::Deserialize, SurrealValue)]
pub struct Record {
  pub id: surrealdb_types::RecordId,
}
