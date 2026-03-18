use axum::Json;
use axum::Router;
use axum::routing::post;

use crate::routes::errors::AppResult;

pub fn routes() -> Router {
  Router::new().route("/onboarding", post(onboarding))
}

async fn onboarding() -> AppResult<Json<()>> {
  todo!()
}
