mod routes;
mod state;

use std::str::FromStr;

use axum::Router;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use persistence::Uuid;
use routes::v1_routes;

use crate::state::RequestExtension;

const EMPLOYEE_ID: &str = "x-blprnt-employee-id";
const PROJECT_ID: &str = "x-blprnt-project-id";
const COMPANY_ID: &str = "x-blprnt-company-id";
const RUN_ID: &str = "x-blprnt-run-id";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing::info!("Starting Blprnt Core");

  let app = Router::new()
    .layer(middleware::from_fn(async |mut request: Request, next: Next| {
      let headers = request.headers();
      let employee = headers.get(EMPLOYEE_ID).and_then(|v| v.to_str().ok()).map(|v| Uuid::from_str(v).unwrap().into());
      let company = headers.get(COMPANY_ID).and_then(|v| v.to_str().ok()).map(|v| Uuid::from_str(v).unwrap().into());
      let project = headers.get(PROJECT_ID).and_then(|v| v.to_str().ok()).map(|v| Uuid::from_str(v).unwrap().into());
      let run = headers.get(RUN_ID).and_then(|v| v.to_str().ok()).map(|v| Uuid::from_str(v).unwrap().into());

      let extension = RequestExtension { employee, project, company, run };
      request.extensions_mut().insert(extension);
      next.run(request).await
    }))
    .merge(v1_routes());

  let listener = tokio::net::TcpListener::bind("0.0.0.0:9171").await.unwrap();
  tracing::info!("Listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();

  Ok(())
}
