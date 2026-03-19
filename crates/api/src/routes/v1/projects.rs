use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::patch;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectPatch;
use persistence::prelude::ProjectRecord;
use persistence::prelude::ProjectRepository;

use crate::routes::errors::ApiError;
use crate::routes::errors::ApiResult;

pub fn routes() -> Router {
  Router::new()
    .route("/projects", get(list_projects))
    .route("/projects/:project_id", get(get_project))
    .route("/projects/:project_id", patch(update_project))
    .route("/projects/:project_id", delete(delete_project))
}

async fn list_projects() -> ApiResult<Json<Vec<ProjectRecord>>> {
  Ok(Json(ProjectRepository::list().await.map_err(ApiError::from)?))
}

async fn get_project(Path(project_id): Path<ProjectId>) -> ApiResult<Json<ProjectRecord>> {
  Ok(Json(ProjectRepository::get(project_id).await.map_err(ApiError::from)?))
}

async fn update_project(
  Path(project_id): Path<ProjectId>,
  Json(payload): Json<ProjectPatch>,
) -> ApiResult<Json<ProjectRecord>> {
  Ok(Json(ProjectRepository::update(project_id, payload).await.map_err(ApiError::from)?))
}

async fn delete_project(Path(project_id): Path<ProjectId>) -> ApiResult<StatusCode> {
  ProjectRepository::delete(project_id).await.map_err(ApiError::from)?;
  Ok(StatusCode::NO_CONTENT)
}
