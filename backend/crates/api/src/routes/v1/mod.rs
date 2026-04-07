mod auth;
#[cfg(debug_assertions)]
mod dev;
mod employees;
pub(crate) mod issues;
mod mcp_servers;
mod memory;
mod openapi;
mod projects;
mod providers;
pub(crate) mod runs;
mod skills;
pub(crate) mod telegram;

mod public;

use axum::Router;
use axum::middleware;
use auth::protected_routes as auth_protected_routes;
use auth::public_routes as auth_public_routes;
#[cfg(debug_assertions)]
use dev::routes as dev_routes;
use employees::routes as employees_routes;
use issues::routes as issues_routes;
use mcp_servers::routes as mcp_servers_routes;
use memory::routes as memory_routes;
use openapi::routes as openapi_routes;
use projects::routes as projects_routes;
use providers::routes as providers_routes;
use public::routes as public_routes;
use runs::routes as runs_routes;
use skills::routes as skills_routes;
use telegram::protected_routes as telegram_protected_routes;

use crate::middleware::api_middleware;

pub fn routes() -> Router {
  let protected_routes = Router::new()
    .merge(auth_protected_routes())
    .merge(issues_routes())
    .merge(mcp_servers_routes())
    .merge(employees_routes())
    .merge(runs_routes())
    .merge(telegram_protected_routes())
    .merge(skills_routes())
    .merge(memory_routes())
    .merge(projects_routes())
    .merge(providers_routes());

  #[cfg(debug_assertions)]
  let protected_routes = protected_routes.merge(dev_routes());
  let protected_routes = protected_routes.layer(middleware::from_fn(api_middleware));

  let public_routes = Router::new()
    .merge(public_routes())
    .merge(auth_public_routes())
    .merge(openapi_routes());
  let v1_routes = Router::new().merge(protected_routes).merge(public_routes);

  Router::new().nest("/v1", v1_routes)
}

#[cfg(test)]
mod tests;
