mod employees;
mod issues;
mod projects;
mod providers;
mod runs;

use axum::Router;
use axum::middleware;
use employees::routes as employees_routes;
use issues::routes as issues_routes;
use projects::routes as projects_routes;
use providers::routes as providers_routes;
use runs::routes as runs_routes;

use crate::middleware::owner_only;

pub fn routes() -> Router {
  Router::new().nest(
    "/v1",
    Router::new()
      .merge(issues_routes())
      .merge(employees_routes())
      .merge(runs_routes())
      .merge(projects_routes())
      .merge(providers_routes().layer(middleware::from_fn(owner_only))),
  )
}
