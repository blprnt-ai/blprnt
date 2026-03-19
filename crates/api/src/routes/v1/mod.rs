mod employees;
mod issues;
mod runs;

use axum::Router;
use employees::routes as employees_routes;
use issues::routes as issues_routes;
use runs::routes as runs_routes;

pub fn routes() -> Router {
  Router::new().nest("/v1", Router::new().merge(issues_routes()).merge(employees_routes()).merge(runs_routes()))
}
