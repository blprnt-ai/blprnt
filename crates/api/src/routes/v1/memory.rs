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
}

#[derive(Debug, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct MemoryFileQuery {
  path: String,
}

#[derive(Debug, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
struct SearchMemoryPayload {
  query: String,
  limit: Option<usize>,
}

async fn list_memory(Path(project_id): Path<Uuid>) -> ApiResult<Json<MemoryListResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.list().await?))
}

async fn list_employee_memory(Extension(extension): Extension<RequestExtension>) -> ApiResult<Json<MemoryListResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.list().await?))
}

async fn read_memory_file(
  Path(project_id): Path<Uuid>,
  Query(query): Query<MemoryFileQuery>,
) -> ApiResult<Json<MemoryReadResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.read(&query.path).await?))
}

async fn read_employee_memory_file(
  Extension(extension): Extension<RequestExtension>,
  Query(query): Query<MemoryFileQuery>,
) -> ApiResult<Json<MemoryReadResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.read(&query.path).await?))
}

async fn search_memory(
  Path(project_id): Path<Uuid>,
  Json(payload): Json<SearchMemoryPayload>,
) -> ApiResult<Json<MemorySearchResult>> {
  let project_id = ProjectId::from(project_id);
  let service = ProjectMemoryService::new(project_id).await?;
  Ok(Json(service.search(&payload.query, payload.limit).await?))
}

async fn search_employee_memory(
  Extension(extension): Extension<RequestExtension>,
  Json(payload): Json<SearchMemoryPayload>,
) -> ApiResult<Json<MemorySearchResult>> {
  let service = EmployeeMemoryService::new(extension.employee.id).await?;
  Ok(Json(service.search(&payload.query, payload.limit).await?))
}
