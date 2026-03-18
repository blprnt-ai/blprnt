mod onboarding;
mod static_files;
mod v1;

pub mod errors;

use axum::Router;
use axum::middleware;

use self::onboarding::routes as onboarding_routes;
use self::static_files::routes as static_file_routes;
use self::v1::routes as v1_routes;
use crate::middleware::api_middleware;

pub fn routes() -> Router {
  let protected_routes = Router::new().merge(v1_routes()).layer(middleware::from_fn(api_middleware));
  let public_routes = Router::new().merge(onboarding_routes());

  let api = Router::new().merge(protected_routes).merge(public_routes);

  Router::new().nest("/api", api).merge(static_file_routes())
}
