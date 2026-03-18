mod issues;

use axum::Router;
use issues::routes as issues_routes;

pub fn routes() -> Router {
  Router::new().nest(
    "/api/v1",
    // Issues routes
    Router::new().merge(issues_routes()),
  )
}
