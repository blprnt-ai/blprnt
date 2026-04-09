use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::routing::get;
use axum::routing::post;
use memory::EmployeeMemoryService;
use memory::MemoryListResult;
use memory::MemoryReadResult;
use memory::MemorySearchResult;
use memory::ProjectMemoryService;
use memory::ProjectPlanReadResult;
use memory::ProjectPlansListResult;
use memory::ProjectPlansService;
use persistence::Uuid;
use persistence::prelude::ProjectId;

use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  Router::new()
    .route("/employees/me/memory", get(list_employee_memory))
    .route("/employees/me/memory/file", get(read_employee_memory_file))
    .route("/employees/me/memory/search", post(search_employee_memory))
    .route("/projects/{project_id}/memory", get(list_memory))
    .route("/projects/{project_id}/memory/file", get(read_memory_file))
    .route("/projects/{project_id}/memory/search", post(search_memory))
    .route("/projects/{project_id}/plans", get(list_project_plans))
    .route("/projects/{project_id}/plans/file", get(read_project_plan_file))
}

#[derive(Debug, serde::Deserialize, ts_rs::TS, utoipa::IntoParams, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct MemoryFileQuery {
  path: String,
}

#[derive(Debug, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct SearchMemoryPayload {
  query: String,
  limit: Option<usize>,
}

#[utoipa::path(
  get,
  path = "/projects/{project_id}/memory",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  responses(
    (status = 200, description = "List project memory files", body = MemoryListResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn list_memory(Path(project_id): Path<Uuid>) -> ApiResult<Json<MemoryListResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.list().await?))
}

#[utoipa::path(
  get,
  path = "/employees/me/memory",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List employee memory files", body = MemoryListResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn list_employee_memory(
  Extension(extension): Extension<RequestExtension>,
) -> ApiResult<Json<MemoryListResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.list().await?))
}

#[utoipa::path(
  get,
  path = "/projects/{project_id}/memory/file",
  security(("blprnt_employee_id" = [])),
  params(
    ("project_id" = Uuid, Path, description = "Project id"),
    MemoryFileQuery
  ),
  responses(
    (status = 200, description = "Read a project memory file", body = MemoryReadResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn read_memory_file(
  Path(project_id): Path<Uuid>,
  Query(query): Query<MemoryFileQuery>,
) -> ApiResult<Json<MemoryReadResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.read(&query.path).await?))
}

#[utoipa::path(
  get,
  path = "/employees/me/memory/file",
  security(("blprnt_employee_id" = [])),
  params(MemoryFileQuery),
  responses(
    (status = 200, description = "Read an employee memory file", body = MemoryReadResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn read_employee_memory_file(
  Extension(extension): Extension<RequestExtension>,
  Query(query): Query<MemoryFileQuery>,
) -> ApiResult<Json<MemoryReadResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.read(&query.path).await?))
}

#[utoipa::path(
  get,
  path = "/projects/{project_id}/plans",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  responses(
    (status = 200, description = "List project plan files", body = ProjectPlansListResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn list_project_plans(Path(project_id): Path<Uuid>) -> ApiResult<Json<ProjectPlansListResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectPlansService::new(project_id).await?;
  Ok(Json(service.list().await?))
}

#[utoipa::path(
  get,
  path = "/projects/{project_id}/plans/file",
  security(("blprnt_employee_id" = [])),
  params(
    ("project_id" = Uuid, Path, description = "Project id"),
    MemoryFileQuery
  ),
  responses(
    (status = 200, description = "Read a project plan file", body = ProjectPlanReadResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn read_project_plan_file(
  Path(project_id): Path<Uuid>,
  Query(query): Query<MemoryFileQuery>,
) -> ApiResult<Json<ProjectPlanReadResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectPlansService::new(project_id).await?;
  Ok(Json(service.read(&query.path).await?))
}

#[utoipa::path(
  post,
  path = "/projects/{project_id}/memory/search",
  security(("blprnt_employee_id" = [])),
  params(("project_id" = Uuid, Path, description = "Project id")),
  request_body = SearchMemoryPayload,
  responses(
    (status = 200, description = "Search project memory", body = MemorySearchResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "Project not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn search_memory(
  Path(project_id): Path<Uuid>,
  Json(payload): Json<SearchMemoryPayload>,
) -> ApiResult<Json<MemorySearchResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.search(&payload.query, payload.limit).await?))
}

#[utoipa::path(
  post,
  path = "/employees/me/memory/search",
  security(("blprnt_employee_id" = [])),
  request_body = SearchMemoryPayload,
  responses(
    (status = 200, description = "Search employee memory", body = MemorySearchResult),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "memory"
)]
pub(super) async fn search_employee_memory(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<SearchMemoryPayload>,
) -> ApiResult<Json<MemorySearchResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.search(&payload.query, payload.limit).await?))
}
