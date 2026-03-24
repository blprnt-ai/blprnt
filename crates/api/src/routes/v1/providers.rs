use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use persistence::prelude::ProviderId;
use persistence::prelude::ProviderModel;
use persistence::prelude::ProviderPatch;
use persistence::prelude::ProviderRepository;
use serde_json::json;
use shared::agent::Provider;

use crate::dto::ProviderDto;
use crate::provider_helpers;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/providers", get(list_providers))
    .route("/providers", post(create_provider))
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

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct CreateProviderPayload {
  provider: Provider,
  api_key:  Option<String>,
  base_url: Option<String>,
}

async fn create_provider(Json(payload): Json<CreateProviderPayload>) -> ApiResult<Json<ProviderDto>> {
  let provider = match payload.provider {
    Provider::ClaudeCode => provider_helpers::link_claude_account().await,
    Provider::Codex => provider_helpers::link_codex_account().await,
    provider if payload.api_key.is_some() => {
      let mut provider = ProviderModel::new(provider);
      provider.base_url = payload.base_url;
      provider_helpers::upsert_provider_with_api_key(provider, payload.api_key.unwrap()).await
    }
    provider => Err(ApiErrorKind::BadRequest(json!({"message": "API key is required", "provider": provider})).into()),
  };

  Ok(Json(provider?.into()))
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
