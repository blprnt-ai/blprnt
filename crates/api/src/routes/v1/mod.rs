mod employees;
mod issues;
mod memory;
mod projects;
mod providers;
mod runs;

mod public;

use axum::Router;
use axum::middleware;
use employees::routes as employees_routes;
use issues::routes as issues_routes;
use memory::routes as memory_routes;
use projects::routes as projects_routes;
use providers::routes as providers_routes;
use public::routes as public_routes;
use runs::routes as runs_routes;

use crate::middleware::api_middleware;
use crate::middleware::owner_only;

pub fn routes() -> Router {
  let protected_routes = Router::new()
    .merge(issues_routes())
    .merge(employees_routes())
    .merge(runs_routes())
    .merge(memory_routes())
    .merge(projects_routes())
    .merge(providers_routes().layer(middleware::from_fn(owner_only)))
    .layer(middleware::from_fn(api_middleware));

  let public_routes = Router::new().merge(public_routes());
  let v1_routes = Router::new().merge(protected_routes).merge(public_routes);

  Router::new().nest("/v1", v1_routes)
}

#[cfg(test)]
mod tests;
