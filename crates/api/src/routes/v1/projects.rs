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

#[utoipa::path(
  get,
  path = "/projects",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List projects", body = [ProjectDto]),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "projects"
)]
pub(super) async fn list_projects() -> ApiResult<Json<Vec<ProjectDto>>> {
  Ok(Json(ProjectRepository::list().await?.into_iter().map(|p| p.into()).collect()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct CreateProjectPayload {
  description:         String,
  name:                String,
  working_directories: Vec<String>,
}

impl From<CreateProjectPayload> for ProjectModel {
  fn from(payload: CreateProjectPayload) -> Self {
    Self {
      description:         payload.description,
      name:                payload.name,
      working_directories: payload.working_directories,
      created_at:          Utc::now(),
      updated_at:          Utc::now(),
    }
  }
}

#[utoipa::path(
  post,
  path = "/projects",
  security(("blprnt_employee_id" = [])),
  request_body = CreateProjectPayload,
  responses(
    (status = 200, description = "Create a project", body = ProjectDto),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "projects"
)]
pub(super) async fn create_project(Json(payload): Json<CreateProjectPayload>) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::create(payload.into()).await?.into()))
}

#[utoipa::path(
  get,
  path = "/projects/{project_id}",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  responses(
    (status = 200, description = "Fetch a project", body = ProjectDto),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "projects"
)]
pub(super) async fn get_project(Path(project_id): Path<Uuid>) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::get(project_id.into()).await?.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct ProjectPatchPayload {
  description:         Option<String>,
  name:                Option<String>,
  working_directories: Option<Vec<String>>,
}

impl From<ProjectPatchPayload> for ProjectPatch {
  fn from(payload: ProjectPatchPayload) -> Self {
    Self {
      description:         payload.description,
      name:                payload.name,
      working_directories: payload.working_directories,
      updated_at:          None,
    }
  }
}

#[utoipa::path(
  patch,
  path = "/projects/{project_id}",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  request_body = ProjectPatchPayload,
  responses(
    (status = 200, description = "Update a project", body = ProjectDto),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "projects"
)]
pub(super) async fn update_project(
  Path(project_id): Path<Uuid>,
  Json(payload): Json<ProjectPatchPayload>,
) -> ApiResult<Json<ProjectDto>> {
  Ok(Json(ProjectRepository::update(project_id.into(), payload.into()).await?.into()))
}

#[utoipa::path(
  delete,
  path = "/projects/{project_id}",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  responses(
    (status = 204, description = "Delete a project"),
    (status = 400, description = "Missing or invalid employee id", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "projects"
)]
pub(super) async fn delete_project(Path(project_id): Path<Uuid>) -> ApiResult<StatusCode> {
  ProjectRepository::delete(project_id.into()).await?;
  Ok(StatusCode::NO_CONTENT)
}
