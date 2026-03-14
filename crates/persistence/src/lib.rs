#![allow(clippy::redundant_field_names)]

mod connection;
mod models_v2;
use tracing as _;

pub mod prelude {
  pub use common::shared::prelude::SurrealId;

  pub use crate::models_v2::*;
}
