#![allow(clippy::redundant_field_names)]

mod surreal_id;

mod connection;
mod models;
pub use surrealdb_types::Uuid;
use tracing as _;

pub mod prelude {
  pub use crate::connection::DbConnection;
  pub use crate::connection::SurrealConnection;
  pub use crate::models::*;
  pub use crate::surreal_id::DbId;
  pub use crate::surreal_id::SurrealId;
}
