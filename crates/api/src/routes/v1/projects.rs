use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectPatch;
use persistence::prelude::ProjectRepository;

use crate::dto::ProjectDto;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/projects", get(list_projects))
    .route("/projects/{project_id}", get(get_project))
    .route("/projects/{project_id}", patch(update_project))
    .route("/projects/{project_id}", delete(delete_project))
}

async fn list_projects() -> ApiResult<Json<Vec<ProjectDto>>> {
  Ok(Json(ProjectRepository::list().await?.into_iter().map(|p| p.into()).collect()))
}

async fn get_project(Path(project_id): Path<ProjectId>) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::get(project_id).await?.into()))
}

async fn update_project(
  Path(project_id): Path<ProjectId>,
  Json(payload): Json<ProjectPatch>,
) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::update(project_id, payload).await?.into()))
}

async fn delete_project(Path(project_id): Path<ProjectId>) -> ApiResult<StatusCode> {
  ProjectRepository::delete(project_id).await?;
  Ok(StatusCode::NO_CONTENT)
}
