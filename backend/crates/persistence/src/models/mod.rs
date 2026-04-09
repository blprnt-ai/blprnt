mod auth;
mod employees;
mod issues;
mod mcp;
mod projects;
mod providers;
mod runs;
mod telegram;
mod turns;

pub use auth::*;
pub use employees::*;
pub use issues::*;
pub use mcp::*;
pub use projects::*;
pub use providers::*;
pub use runs::*;
use surrealdb_types::SurrealValue;
pub use telegram::*;
pub use turns::*;

#[derive(Clone, Debug, serde::Deserialize, SurrealValue)]
pub struct Record {
  pub id: surrealdb_types::RecordId,
}
