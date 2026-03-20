mod static_files;
mod v1;

pub mod errors;

use axum::Router;

use self::static_files::routes as static_file_routes;
use self::v1::routes as v1_routes;

pub fn routes() -> Router {
  Router::new().nest("/api", v1_routes()).merge(static_file_routes())
}
