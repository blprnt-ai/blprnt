use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use chrono::Utc;
use persistence::Uuid;
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

#[utoipa::path(
  get,
  path = "/providers",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List providers", body = [ProviderDto]),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access providers", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "providers"
)]
pub(super) async fn list_providers() -> ApiResult<Json<Vec<ProviderDto>>> {
  Ok(Json(ProviderRepository::list().await?.into_iter().map(|p| p.into()).collect()))
}

#[utoipa::path(
  get,
  path = "/providers/{provider_id}",
  security(("blprnt_employee_id" = [])),
  params(("provider_id" = Uuid, Path, description = "Provider id")),
  responses(
    (status = 200, description = "Fetch a provider", body = ProviderDto),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access providers", body = crate::routes::errors::ApiError),
    (status = 404, description = "Provider not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "providers"
)]
pub(super) async fn get_provider(Path(provider_id): Path<Uuid>) -> ApiResult<Json<ProviderDto>> {
  let provider_id: ProviderId = provider_id.into();
  Ok(Json(ProviderRepository::get(provider_id).await?.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct CreateProviderPayload {
  provider: Provider,
  api_key:  Option<String>,
  base_url: Option<String>,
}

#[utoipa::path(
  post,
  path = "/providers",
  security(("blprnt_employee_id" = [])),
  request_body = CreateProviderPayload,
  responses(
    (status = 200, description = "Create or link a provider", body = ProviderDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access providers", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "providers"
)]
pub(super) async fn create_provider(Json(payload): Json<CreateProviderPayload>) -> ApiResult<Json<ProviderDto>> {
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

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct UpdateProviderPayload {
  provider: Provider,
  api_key:  Option<String>,
  base_url: Option<String>,
}

impl From<UpdateProviderPayload> for ProviderPatch {
  fn from(payload: UpdateProviderPayload) -> Self {
    ProviderPatch { base_url: payload.base_url, updated_at: Some(Utc::now()) }
  }
}

#[utoipa::path(
  patch,
  path = "/providers/{provider_id}",
  security(("blprnt_employee_id" = [])),
  params(("provider_id" = Uuid, Path, description = "Provider id")),
  request_body = UpdateProviderPayload,
  responses(
    (status = 200, description = "Update a provider", body = ProviderDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access providers", body = crate::routes::errors::ApiError),
    (status = 404, description = "Provider not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "providers"
)]
pub(super) async fn update_provider(
  Path(provider_id): Path<Uuid>,
  Json(payload): Json<UpdateProviderPayload>,
) -> ApiResult<Json<ProviderDto>> {
  let provider_id: ProviderId = provider_id.into();

  match payload.provider.clone() {
    Provider::ClaudeCode => provider_helpers::link_claude_account().await,
    Provider::Codex => provider_helpers::link_codex_account().await,
    provider if payload.api_key.is_some() => {
      let mut provider = ProviderModel::new(provider);
      provider.base_url = payload.base_url.clone();
      provider_helpers::upsert_provider_with_api_key(provider, payload.api_key.clone().unwrap()).await
    }
    provider => Err(ApiErrorKind::BadRequest(json!({"message": "API key is required", "provider": provider})).into()),
  }?;

  Ok(Json(ProviderRepository::update(provider_id, payload.into()).await?.into()))
}

#[utoipa::path(
  delete,
  path = "/providers/{provider_id}",
  security(("blprnt_employee_id" = [])),
  params(("provider_id" = Uuid, Path, description = "Provider id")),
  responses(
    (status = 204, description = "Delete a provider"),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access providers", body = crate::routes::errors::ApiError),
    (status = 404, description = "Provider not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "providers"
)]
pub(super) async fn delete_provider(Path(provider_id): Path<Uuid>) -> ApiResult<StatusCode> {
  let provider_id: ProviderId = provider_id.into();
  ProviderRepository::delete(provider_id).await?;
  Ok(StatusCode::NO_CONTENT)
}
