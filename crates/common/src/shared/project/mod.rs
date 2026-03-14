use crate::shared::prelude::PlanningItemStatus;
use crate::shared::prelude::SurrealId;

pub mod prelude;

mod path_list;
mod schema;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanningReorderItem {
  pub id:     SurrealId,
  pub status: PlanningItemStatus,
}
