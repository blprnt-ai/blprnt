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
use persistence::prelude::ProjectModel;
use persistence::prelude::ProjectPatch;
use persistence::prelude::ProjectRepository;

use crate::dto::ProjectDto;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/projects", get(list_projects))
    .route("/projects", post(create_project))
    .route("/projects/{project_id}", get(get_project))
    .route("/projects/{project_id}", patch(update_project))
    .route("/projects/{project_id}", delete(delete_project))
}

async fn list_projects() -> ApiResult<Json<Vec<ProjectDto>>> {
  Ok(Json(ProjectRepository::list().await?.into_iter().map(|p| p.into()).collect()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct CreateProjectPayload {
  name:                String,
  working_directories: Vec<String>,
}

impl From<CreateProjectPayload> for ProjectModel {
  fn from(payload: CreateProjectPayload) -> Self {
    Self {
      name:                payload.name,
      working_directories: payload.working_directories,
      created_at:          Utc::now(),
      updated_at:          Utc::now(),
    }
  }
}

async fn create_project(Json(payload): Json<CreateProjectPayload>) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::create(payload.into()).await?.into()))
}

async fn get_project(Path(project_id): Path<Uuid>) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::get(project_id.into()).await?.into()))
}

async fn update_project(
  Path(project_id): Path<Uuid>,
  Json(payload): Json<ProjectPatch>,
) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::update(project_id.into(), payload).await?.into()))
}

async fn delete_project(Path(project_id): Path<Uuid>) -> ApiResult<StatusCode> {
  ProjectRepository::delete(project_id.into()).await?;
  Ok(StatusCode::NO_CONTENT)
}
