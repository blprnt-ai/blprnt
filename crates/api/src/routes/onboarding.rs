use axum::Json;
use axum::Router;
use axum::routing::post;

use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new().route("/onboarding", post(onboarding))
}

async fn onboarding() -> ApiResult<Json<()>> {
  todo!()
}
