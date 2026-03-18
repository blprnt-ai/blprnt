#![allow(clippy::redundant_field_names)]

mod surreal_id;

mod connection;
mod models_v2;
pub use surrealdb_types::Uuid;
use tracing as _;

pub mod prelude {
  pub use crate::models_v2::*;
  pub use crate::surreal_id::DbId;
  pub use crate::surreal_id::SurrealId;
}
