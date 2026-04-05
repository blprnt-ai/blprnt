use axum::Extension;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::delete;
use persistence::prelude::SurrealConnection;
use serde_json::json;

use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new().route("/dev/database", delete(nuke_database))
}

async fn nuke_database(Extension(extension): Extension<RequestExtension>) -> ApiResult<StatusCode> {
  if !extension.employee.is_owner() {
    return Err(ApiErrorKind::Forbidden(json!("Forbidden")).into());
  }

  SurrealConnection::reset().await?;
  Ok(StatusCode::NO_CONTENT)
}
