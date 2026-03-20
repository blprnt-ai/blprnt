use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use persistence::prelude::ProviderId;
use persistence::prelude::ProviderPatch;
use persistence::prelude::ProviderRepository;

use crate::dto::ProviderDto;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/providers", get(list_providers))
    .route("/providers/{provider_id}", get(get_provider))
    .route("/providers/{provider_id}", patch(update_provider))
    .route("/providers/{provider_id}", delete(delete_provider))
}

async fn list_providers() -> ApiResult<Json<Vec<ProviderDto>>> {
  Ok(Json(ProviderRepository::list().await?.into_iter().map(|p| p.into()).collect()))
}

async fn get_provider(Path(provider_id): Path<ProviderId>) -> ApiResult<Json<ProviderDto>> {
  Ok(Json(ProviderRepository::get(provider_id).await?.into()))
}

async fn update_provider(
  Path(provider_id): Path<ProviderId>,
  Json(payload): Json<ProviderPatch>,
) -> ApiResult<Json<ProviderDto>> {
  Ok(Json(ProviderRepository::update(provider_id, payload).await?.into()))
}

async fn delete_provider(Path(provider_id): Path<ProviderId>) -> ApiResult<StatusCode> {
  ProviderRepository::delete(provider_id).await?;
  Ok(StatusCode::NO_CONTENT)
}
