use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::Query;
use axum::middleware;
use axum::routing::get;
use axum::routing::patch;
use axum::routing::post;
use persistence::Uuid;
use persistence::prelude::McpServerModel;
use persistence::prelude::McpServerPatch;
use persistence::prelude::McpServerRepository;
use persistence::prelude::RunEnabledMcpServerRepository;
use persistence::prelude::RunId;
use shared::tools::McpServerAuthState;

use crate::dto::McpServerDto;
use crate::dto::RunEnabledMcpServerDto;
use crate::mcp_oauth;
use crate::middleware::owner_only;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;

pub fn routes() -> Router {
  let owner_routes = Router::new()
    .route("/mcp-servers", get(list_mcp_servers))
    .route("/mcp-servers", post(create_mcp_server))
    .route("/mcp-servers/{server_id}", patch(update_mcp_server))
    .route("/mcp-servers/{server_id}/oauth", get(get_mcp_oauth_status))
    .route("/mcp-servers/{server_id}/oauth/launch", post(launch_mcp_oauth))
    .route("/mcp-servers/{server_id}/oauth/reconnect", post(reconnect_mcp_oauth))
    .route("/mcp-servers/{server_id}/oauth/complete", post(complete_mcp_oauth))
    .layer(middleware::from_fn(owner_only));

  let shared_routes = Router::new()
    .route("/runs/{run_id}/mcp-servers", get(list_run_enabled_mcp_servers))
    .route("/mcp-servers/{server_id}/oauth/callback", get(complete_mcp_oauth_callback));

  Router::new().merge(owner_routes).merge(shared_routes)
}

#[utoipa::path(
  get,
  path = "/mcp-servers",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "List configured MCP servers", body = [McpServerDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn list_mcp_servers() -> ApiResult<Json<Vec<McpServerDto>>> {
  Ok(Json(McpServerRepository::list().await?.into_iter().map(Into::into).collect()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct CreateMcpServerPayload {
  pub display_name: String,
  pub description:  String,
  pub transport:    String,
  pub endpoint_url: String,
  #[serde(default)]
  pub auth_state:   Option<McpServerAuthState>,
  #[serde(default)]
  pub auth_summary: Option<String>,
  #[serde(default = "default_enabled")]
  pub enabled:      bool,
}

fn default_enabled() -> bool {
  true
}

#[utoipa::path(
  post,
  path = "/mcp-servers",
  security(("blprnt_employee_id" = [])),
  request_body = CreateMcpServerPayload,
  responses(
    (status = 200, description = "Create a configured MCP server", body = McpServerDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn create_mcp_server(Json(payload): Json<CreateMcpServerPayload>) -> ApiResult<Json<McpServerDto>> {
  let mut model =
    McpServerModel::new(payload.display_name, payload.description, payload.transport, payload.endpoint_url);
  if let Some(auth_state) = payload.auth_state {
    model.auth_state = auth_state;
  }
  model.auth_summary = payload.auth_summary;
  model.enabled = payload.enabled;
  Ok(Json(McpServerRepository::create(model).await?.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub(super) struct McpServerPatchPayload {
  pub display_name: Option<String>,
  pub description:  Option<String>,
  pub transport:    Option<String>,
  pub endpoint_url: Option<String>,
  pub auth_state:   Option<McpServerAuthState>,
  pub auth_summary: Option<Option<String>>,
  pub enabled:      Option<bool>,
}

impl From<McpServerPatchPayload> for McpServerPatch {
  fn from(payload: McpServerPatchPayload) -> Self {
    Self {
      display_name: payload.display_name,
      description:  payload.description,
      transport:    payload.transport,
      endpoint_url: payload.endpoint_url,
      auth_state:   payload.auth_state,
      auth_summary: payload.auth_summary,
      enabled:      payload.enabled,
    }
  }
}

#[utoipa::path(
  patch,
  path = "/mcp-servers/{server_id}",
  security(("blprnt_employee_id" = [])),
  params(("server_id" = Uuid, Path, description = "Configured MCP server id")),
  request_body = McpServerPatchPayload,
  responses(
    (status = 200, description = "Update a configured MCP server", body = McpServerDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn update_mcp_server(
  Path(server_id): Path<Uuid>,
  Json(payload): Json<McpServerPatchPayload>,
) -> ApiResult<Json<McpServerDto>> {
  Ok(Json(McpServerRepository::update(server_id.into(), payload.into()).await?.into()))
}

#[utoipa::path(
  get,
  path = "/mcp-servers/{server_id}/oauth",
  security(("blprnt_employee_id" = [])),
  params(("server_id" = Uuid, Path, description = "Configured MCP server id")),
  responses(
    (status = 200, description = "Get MCP OAuth status", body = crate::mcp_oauth::McpOauthStatusDto),
    (status = 403, description = "Only the owner can access MCP OAuth management", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn get_mcp_oauth_status(Path(server_id): Path<Uuid>) -> ApiResult<Json<mcp_oauth::McpOauthStatusDto>> {
  let server = McpServerRepository::get(server_id.into()).await?;
  Ok(Json(
    mcp_oauth::status(&server)
      .await
      .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?,
  ))
}

#[utoipa::path(
  post,
  path = "/mcp-servers/{server_id}/oauth/launch",
  security(("blprnt_employee_id" = [])),
  params(("server_id" = Uuid, Path, description = "Configured MCP server id")),
  responses(
    (status = 200, description = "Start MCP OAuth authorization", body = crate::mcp_oauth::McpOauthLaunchDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access MCP OAuth management", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn launch_mcp_oauth(Path(server_id): Path<Uuid>) -> ApiResult<Json<mcp_oauth::McpOauthLaunchDto>> {
  let server = McpServerRepository::get(server_id.into()).await?;
  Ok(Json(mcp_oauth::launch(&server, false).await?))
}

#[utoipa::path(
  post,
  path = "/mcp-servers/{server_id}/oauth/reconnect",
  security(("blprnt_employee_id" = [])),
  params(("server_id" = Uuid, Path, description = "Configured MCP server id")),
  responses(
    (status = 200, description = "Start MCP OAuth reconnect flow", body = crate::mcp_oauth::McpOauthLaunchDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access MCP OAuth management", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn reconnect_mcp_oauth(Path(server_id): Path<Uuid>) -> ApiResult<Json<mcp_oauth::McpOauthLaunchDto>> {
  let server = McpServerRepository::get(server_id.into()).await?;
  Ok(Json(mcp_oauth::launch(&server, true).await?))
}

#[utoipa::path(
  post,
  path = "/mcp-servers/{server_id}/oauth/complete",
  security(("blprnt_employee_id" = [])),
  params(("server_id" = Uuid, Path, description = "Configured MCP server id")),
  request_body = crate::mcp_oauth::McpOauthCompletePayload,
  responses(
    (status = 200, description = "Complete MCP OAuth authorization", body = crate::mcp_oauth::McpOauthStatusDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Only the owner can access MCP OAuth management", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn complete_mcp_oauth(
  Path(server_id): Path<Uuid>,
  Json(payload): Json<mcp_oauth::McpOauthCompletePayload>,
) -> ApiResult<Json<mcp_oauth::McpOauthStatusDto>> {
  let server = McpServerRepository::get(server_id.into()).await?;
  Ok(Json(mcp_oauth::complete(&server, &payload.code, &payload.state).await?))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::IntoParams)]
#[ts(export)]
pub(super) struct McpOauthCallbackQuery {
  code:  String,
  state: String,
}

#[utoipa::path(
  get,
  path = "/mcp-servers/{server_id}/oauth/callback",
  params(
    ("server_id" = Uuid, Path, description = "Configured MCP server id"),
    McpOauthCallbackQuery
  ),
  responses(
    (status = 200, description = "Complete MCP OAuth authorization via provider callback", body = crate::mcp_oauth::McpOauthStatusDto),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 404, description = "MCP server not found", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn complete_mcp_oauth_callback(
  Path(server_id): Path<Uuid>,
  Query(query): Query<McpOauthCallbackQuery>,
) -> ApiResult<Json<mcp_oauth::McpOauthStatusDto>> {
  let server = McpServerRepository::get(server_id.into()).await?;
  Ok(Json(mcp_oauth::complete(&server, &query.code, &query.state).await?))
}

#[utoipa::path(
  get,
  path = "/runs/{run_id}/mcp-servers",
  security(("blprnt_employee_id" = [])),
  params(("run_id" = Uuid, Path, description = "Run id")),
  responses(
    (status = 200, description = "List run-enabled MCP servers", body = [RunEnabledMcpServerDto]),
    (status = 400, description = "Bad request", body = crate::routes::errors::ApiError),
    (status = 403, description = "Forbidden", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "mcp_servers"
)]
pub(super) async fn list_run_enabled_mcp_servers(
  Path(run_id): Path<Uuid>,
  Extension(extension): Extension<RequestExtension>,
) -> ApiResult<Json<Vec<RunEnabledMcpServerDto>>> {
  let requested_run_id: RunId = run_id.into();
  if let Some(active_run_id) = extension.run_id.as_ref() {
    if active_run_id != &requested_run_id && !extension.employee.is_owner() {
      return Err(
        ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to inspect another run's MCP enablement"))
          .into(),
      );
    }
  } else if !extension.employee.is_owner() {
    return Err(
      ApiErrorKind::Forbidden(serde_json::json!("You are not authorized to inspect run MCP enablement")).into(),
    );
  }

  Ok(Json(RunEnabledMcpServerRepository::list_for_run(requested_run_id).await?.into_iter().map(Into::into).collect()))
}
