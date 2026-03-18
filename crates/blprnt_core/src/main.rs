mod routes;
mod state;

use std::str::FromStr;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::Response;
use persistence::Uuid;
use routes::v1_routes;

use crate::routes::errors::AppErrorKind;
use crate::routes::errors::AppResult;
use crate::state::RequestExtension;

const EMPLOYEE_ID: &str = "x-blprnt-employee-id";
const PROJECT_ID: &str = "x-blprnt-project-id";
const COMPANY_ID: &str = "x-blprnt-company-id";
const RUN_ID: &str = "x-blprnt-run-id";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();
  tracing::info!("Starting Blprnt Core");

  let app = Router::new().merge(v1_routes()).layer(middleware::from_fn(
    async |mut request: Request, next: Next| -> AppResult<Response<Body>> {
      let headers = request.headers();
      let company = headers
        .get(COMPANY_ID)
        .ok_or(AppErrorKind::BadRequest(serde_json::json!("Company ID is required")))?
        .to_str()
        .ok()
        .and_then(|v| Uuid::from_str(v).ok())
        .map(Into::into)
        .ok_or(AppErrorKind::BadRequest(serde_json::json!("Company ID is invalid")))?;

      let employee =
        headers.get(EMPLOYEE_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);
      let project =
        headers.get(PROJECT_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);
      let run = headers.get(RUN_ID).and_then(|v| v.to_str().ok()).and_then(|v| Uuid::from_str(v).ok()).map(Into::into);

      let extension = RequestExtension { employee, project, company, run };
      request.extensions_mut().insert(extension);

      Ok(next.run(request).await)
    },
  ));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:9171").await?;
  tracing::info!("Listening on {}", listener.local_addr()?);
  axum::serve(listener, app).await?;

  Ok(())
}
