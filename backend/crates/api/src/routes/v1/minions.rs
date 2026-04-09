use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::middleware;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::MinionId;
use persistence::prelude::MinionModel;
use persistence::prelude::MinionPatch;
use persistence::prelude::MinionRecord;
use persistence::prelude::MinionRepository;
use persistence::prelude::MinionSource;

use crate::dto::MinionDto;
use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/minions", get(list_minions))
    .route("/minions", post(create_minion))
    .route("/minions/{minion_id}", get(get_minion))
    .route("/minions/{minion_id}", patch(update_minion))
    .route("/minions/{minion_id}", delete(delete_minion))
    .layer(middleware::from_fn(owner_only))
}

#[derive(Clone)]
struct SystemMinionDefinition {
  slug: &'static str,
  display_name: &'static str,
  description: &'static str,
  enabled: bool,
}

const SYSTEM_MINIONS: &[SystemMinionDefinition] = &[SystemMinionDefinition {
  slug: "dreamer",
  display_name: "Dreamer",
  description: "Built-in minion that synthesizes employee and project memory during dreaming runs.",
  enabled: true,
}];

fn system_minion_dto(definition: &SystemMinionDefinition) -> MinionDto {
  MinionDto {
    id: definition.slug.to_string(),
    source: MinionSource::System,
    slug: definition.slug.to_string(),
    display_name: definition.display_name.to_string(),
    description: definition.description.to_string(),
    enabled: definition.enabled,
    prompt: None,
    editable: false,
    created_at: DateTime::<Utc>::UNIX_EPOCH,
    updated_at: DateTime::<Utc>::UNIX_EPOCH,
  }
}

fn custom_minion_dto(record: MinionRecord) -> MinionDto {
  MinionDto {
    id: record.id.uuid().to_string(),
    source: MinionSource::Custom,
    slug: record.slug,
    display_name: record.display_name,
    description: record.description,
    enabled: record.enabled,
    prompt: record.prompt,
    editable: true,
    created_at: record.created_at,
    updated_at: record.updated_at,
  }
}

fn system_minion_by_id(id: &str) -> Option<&'static SystemMinionDefinition> {
  SYSTEM_MINIONS.iter().find(|minion| minion.slug == id)
}

fn custom_minion_id(minion_id: &str) -> ApiResult<MinionId> {
  let uuid = Uuid::parse_str(minion_id)
    .map_err(|_| ApiErrorKind::BadRequest(serde_json::json!({ "message": "Minion id must be a valid uuid" })))?;
  Ok(uuid.into())
}
pub(super) async fn list_minions() -> ApiResult<Json<Vec<MinionDto>>> {
  let mut minions = SYSTEM_MINIONS.iter().map(system_minion_dto).collect::<Vec<_>>();
  minions.extend(MinionRepository::list().await?.into_iter().map(custom_minion_dto));
  Ok(Json(minions))
}
pub(super) async fn get_minion(Path(minion_id): Path<String>) -> ApiResult<Json<MinionDto>> {
  if let Some(system_minion) = system_minion_by_id(&minion_id) {
    return Ok(Json(system_minion_dto(system_minion)));
  }

  Ok(Json(custom_minion_dto(MinionRepository::get(custom_minion_id(&minion_id)?).await?)))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct CreateMinionPayload {
  pub slug: String,
  pub display_name: String,
  pub description: String,
  pub enabled: Option<bool>,
  pub prompt: Option<String>,
}
pub(super) async fn create_minion(Json(payload): Json<CreateMinionPayload>) -> ApiResult<Json<MinionDto>> {
  if system_minion_by_id(&payload.slug).is_some() {
    return Err(ApiErrorKind::BadRequest(serde_json::json!({ "message": "System minion slugs are reserved" })).into());
  }

  let mut model = MinionModel::new(payload.slug, payload.display_name, payload.description, payload.prompt);
  if let Some(enabled) = payload.enabled {
    model.enabled = enabled;
  }

  Ok(Json(custom_minion_dto(MinionRepository::create(model).await?)))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct MinionPatchPayload {
  pub slug: Option<String>,
  pub display_name: Option<String>,
  pub description: Option<String>,
  pub enabled: Option<bool>,
  #[serde(default, deserialize_with = "deserialize_nullable_patch_field")]
  pub prompt: Option<Option<String>>,
}

fn deserialize_nullable_patch_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
  D: serde::Deserializer<'de>,
  T: serde::Deserialize<'de>,
{
  Ok(Some(Option::<T>::deserialize(deserializer)?))
}

impl From<MinionPatchPayload> for MinionPatch {
  fn from(payload: MinionPatchPayload) -> Self {
    Self {
      slug: payload.slug,
      display_name: payload.display_name,
      description: payload.description,
      enabled: payload.enabled,
      prompt: payload.prompt,
    }
  }
}

pub(super) async fn update_minion(
  Path(minion_id): Path<String>,
  Json(payload): Json<MinionPatchPayload>,
) -> ApiResult<Json<MinionDto>> {
  if system_minion_by_id(&minion_id).is_some() {
    return Err(ApiErrorKind::UnprocessableEntity(serde_json::json!({ "message": "System minions are read-only" })).into());
  }

  if payload.slug.as_deref().is_some_and(|slug| system_minion_by_id(slug).is_some()) {
    return Err(ApiErrorKind::BadRequest(serde_json::json!({ "message": "System minion slugs are reserved" })).into());
  }

  Ok(Json(custom_minion_dto(MinionRepository::update(custom_minion_id(&minion_id)?, payload.into()).await?)))
}
pub(super) async fn delete_minion(Path(minion_id): Path<String>) -> ApiResult<StatusCode> {
  if system_minion_by_id(&minion_id).is_some() {
    return Err(ApiErrorKind::UnprocessableEntity(serde_json::json!({ "message": "System minions are read-only" })).into());
  }

  MinionRepository::delete(custom_minion_id(&minion_id)?).await?;
  Ok(StatusCode::NO_CONTENT)
}