use std::collections::BTreeSet;

use axum::Json;
use axum::Router;
use axum::routing::get;
use serde_json::Map;
use serde_json::Value;
use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::ApiKey;
use utoipa::openapi::security::ApiKeyValue;
use utoipa::openapi::security::SecurityScheme;

pub fn routes() -> Router {
  Router::new()
    .route("/auth/openapi.json", get(auth_openapi_json))
    .route("/employees/openapi.json", get(employees_openapi_json))
    .route("/issues/openapi.json", get(issues_openapi_json))
    .route("/mcp-servers/openapi.json", get(mcp_servers_openapi_json))
    .route("/memory/openapi.json", get(memory_openapi_json))
    .route("/projects/openapi.json", get(projects_openapi_json))
    .route("/providers/openapi.json", get(providers_openapi_json))
    .route("/public/openapi.json", get(public_openapi_json))
    .route("/runs/openapi.json", get(runs_openapi_json))
    .route("/skills/openapi.json", get(skills_openapi_json))
    .route("/telegram/openapi.json", get(telegram_openapi_json))
}

#[utoipa::path(
  get,
  path = "/auth/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for auth routes", body = Object),
  ),
  tag = "auth"
)]
async fn auth_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("auth"))
}

#[utoipa::path(
  get,
  path = "/employees/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for employee routes", body = Object),
  ),
  tag = "employees"
)]
async fn employees_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("employees"))
}

#[utoipa::path(
  get,
  path = "/issues/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for issue routes", body = Object),
  ),
  tag = "issues"
)]
async fn issues_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("issues"))
}

#[utoipa::path(
  get,
  path = "/mcp-servers/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for MCP server routes", body = Object),
  ),
  tag = "mcp_servers"
)]
async fn mcp_servers_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("mcp_servers"))
}

#[utoipa::path(
  get,
  path = "/memory/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for memory routes", body = Object),
  ),
  tag = "memory"
)]
async fn memory_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("memory"))
}

#[utoipa::path(
  get,
  path = "/projects/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for project routes", body = Object),
  ),
  tag = "projects"
)]
async fn projects_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("projects"))
}

#[utoipa::path(
  get,
  path = "/providers/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for provider routes", body = Object),
  ),
  tag = "providers"
)]
async fn providers_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("providers"))
}

#[utoipa::path(
  get,
  path = "/public/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for public routes", body = Object),
  ),
  tag = "public"
)]
async fn public_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("public"))
}

#[utoipa::path(
  get,
  path = "/runs/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for run routes", body = Object),
  ),
  tag = "runs"
)]
async fn runs_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("runs"))
}

#[utoipa::path(
  get,
  path = "/skills/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for skill routes", body = Object),
  ),
  tag = "skills"
)]
async fn skills_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("skills"))
}

#[utoipa::path(
  get,
  path = "/telegram/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for Telegram routes", body = Object),
  ),
  tag = "telegram"
)]
async fn telegram_openapi_json() -> Json<Value> {
  Json(scoped_openapi_json("telegram"))
}

fn scoped_openapi_json(tag: &str) -> Value {
  let mut document = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI document should serialize");
  filter_openapi_by_tag(&mut document, tag);
  document
}

fn filter_openapi_by_tag(document: &mut Value, tag: &str) {
  let Some(root) = document.as_object_mut() else {
    return;
  };

  filter_tags(root, tag);
  filter_paths(root, tag);
  filter_components(root);
}

fn filter_tags(root: &mut Map<String, Value>, tag: &str) {
  let Some(tags) = root.get_mut("tags").and_then(Value::as_array_mut) else {
    return;
  };

  tags.retain(|entry| entry.get("name").and_then(Value::as_str) == Some(tag));
}

fn filter_paths(root: &mut Map<String, Value>, tag: &str) {
  let Some(paths) = root.get_mut("paths").and_then(Value::as_object_mut) else {
    return;
  };

  let mut kept_paths = Map::new();
  for (path, path_item) in paths.iter() {
    let Some(path_item_object) = path_item.as_object() else {
      continue;
    };

    let filtered_operations = path_item_object
      .iter()
      .filter(|(_, operation)| operation_has_tag(operation, tag))
      .map(|(method, operation)| (method.clone(), operation.clone()))
      .collect::<Map<String, Value>>();

    if !filtered_operations.is_empty() {
      kept_paths.insert(path.clone(), Value::Object(filtered_operations));
    }
  }

  *paths = kept_paths;
}

fn operation_has_tag(operation: &Value, tag: &str) -> bool {
  operation
    .get("tags")
    .and_then(Value::as_array)
    .map(|tags| tags.iter().any(|value| value.as_str() == Some(tag)))
    .unwrap_or(false)
}

fn filter_components(root: &mut Map<String, Value>) {
  let mut referenced_schemas = BTreeSet::new();

  if let Some(paths) = root.get("paths") {
    collect_schema_refs(paths, &mut referenced_schemas);
  }

  let Some(components) = root.get_mut("components").and_then(Value::as_object_mut) else {
    return;
  };

  let Some(all_schemas) = components.get("schemas").and_then(Value::as_object).cloned() else {
    return;
  };

  let mut pending = referenced_schemas.iter().cloned().collect::<Vec<_>>();
  while let Some(schema_name) = pending.pop() {
    let Some(schema) = all_schemas.get(&schema_name) else {
      continue;
    };

    let mut nested_refs = BTreeSet::new();
    collect_schema_refs(schema, &mut nested_refs);
    for nested in nested_refs {
      if referenced_schemas.insert(nested.clone()) {
        pending.push(nested);
      }
    }
  }

  if let Some(schemas) = components.get_mut("schemas").and_then(Value::as_object_mut) {
    schemas.retain(|name, _| referenced_schemas.contains(name));
  }
}

fn collect_schema_refs(value: &Value, refs: &mut BTreeSet<String>) {
  match value {
    Value::Object(object) => {
      if let Some(reference) = object.get("$ref").and_then(Value::as_str)
        && let Some(schema_name) = reference.strip_prefix("#/components/schemas/")
      {
        refs.insert(schema_name.to_string());
      }

      for child in object.values() {
        collect_schema_refs(child, refs);
      }
    }
    Value::Array(items) => {
      for item in items {
        collect_schema_refs(item, refs);
      }
    }
    _ => {}
  }
}

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    let mut components = openapi.components.take().unwrap_or_else(|| ComponentsBuilder::new().build());
    components.add_security_scheme(
      "blprnt_employee_id",
      SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-blprnt-employee-id"))),
    );
    openapi.components = Some(components);
  }
}

#[derive(OpenApi)]
#[openapi(
  info(title = "blprnt API", version = env!("CARGO_PKG_VERSION")),
  servers((url = "/api/v1", description = "Versioned API base path")),
  modifiers(&SecurityAddon),
  paths(
    auth_openapi_json,
    employees_openapi_json,
    issues_openapi_json,
    mcp_servers_openapi_json,
    memory_openapi_json,
    projects_openapi_json,
    providers_openapi_json,
    public_openapi_json,
    runs_openapi_json,
    skills_openapi_json,
    telegram_openapi_json,
    super::auth::status,
    super::auth::bootstrap_owner,
    super::auth::login,
    super::auth::me,
    super::auth::logout,
    super::public::owner_onboarding,
    super::public::get_owner,
    super::skills::list_skills,
    super::projects::list_projects,
    super::projects::create_project,
    super::projects::get_project,
    super::projects::update_project,
    super::projects::delete_project,
    super::mcp_servers::list_mcp_servers,
    super::mcp_servers::create_mcp_server,
    super::mcp_servers::update_mcp_server,
    super::mcp_servers::get_mcp_oauth_status,
    super::mcp_servers::launch_mcp_oauth,
    super::mcp_servers::reconnect_mcp_oauth,
    super::mcp_servers::complete_mcp_oauth,
    super::mcp_servers::complete_mcp_oauth_callback,
    super::mcp_servers::list_run_enabled_mcp_servers,
    super::providers::list_providers,
    super::providers::create_provider,
    super::providers::get_provider,
    super::providers::update_provider,
    super::providers::delete_provider,
    super::memory::list_employee_memory,
    super::memory::read_employee_memory_file,
    super::memory::search_employee_memory,
    super::memory::list_memory,
    super::memory::read_memory_file,
    super::memory::search_memory,
    super::memory::list_project_plans,
    super::memory::read_project_plan_file,
    super::employees::get_me,
    super::employees::get_employee,
    super::employees::list_employees,
    super::employees::org_chart,
    super::employees::import_employee,
    super::employees::create_employee,
    super::employees::update_employee,
    super::employees::terminate_employee,
    super::issues::create_issue,
    super::issues::get_issue,
    super::issues::get_my_work,
     super::issues::list_issue_runs,
    super::issues::list_issues,
    super::issues::list_issue_children,
    super::issues::update_issue,
    super::issues::get_comments,
    super::issues::add_comment,
    super::issues::add_attachment,
    super::issues::assign_issue,
    super::issues::unassign_issue,
    super::issues::checkout_issue,
    super::issues::release_issue,
    super::runs::list_runs,
    super::runs::get_run,
    super::runs::trigger_run,
    super::runs::append_message,
    super::runs::cancel_run,
    super::telegram::get_telegram_config,
    super::telegram::upsert_telegram_config,
    super::telegram::create_telegram_link_code,
    super::telegram::list_telegram_links
  ),
  components(
    schemas(
      crate::routes::errors::ApiError,
      memory::ProjectPlanListItem,
      memory::ProjectPlanReadResult,
      memory::ProjectPlansListResult,
      super::auth::AuthStatusDto,
      crate::dto::RunDto,
      crate::dto::McpServerDto,
      crate::dto::RunEnabledMcpServerDto,
      crate::mcp_oauth::McpOauthMetadataDto,
      crate::mcp_oauth::McpOauthLaunchDto,
      crate::mcp_oauth::McpOauthStatusDto,
      crate::mcp_oauth::McpOauthCompletePayload,
      crate::dto::RunSummaryDto,
      crate::dto::TurnDto,
      crate::dto::TelegramConfigDto,
      crate::dto::TelegramLinkDto,
      crate::dto::TelegramLinkCodeDto,
      crate::dto::TelegramMessageCorrelationDto,
      crate::dto::MyWorkItemDto,
      crate::dto::MyWorkReasonDto,
      crate::dto::MyWorkResponseDto,
      persistence::prelude::RunStatus,
      persistence::prelude::RunTrigger,
      shared::tools::McpServerAuthState,
      persistence::prelude::TelegramParseMode,
      persistence::prelude::TelegramLinkStatus,
      persistence::prelude::TelegramMessageDirection,
      persistence::prelude::TelegramCorrelationKind,
      persistence::prelude::TelegramNotificationPreferences,
      persistence::prelude::ReasoningEffort,
      persistence::prelude::TurnStep,
      persistence::prelude::TurnStepContents,
      persistence::prelude::TurnStepContent,
      persistence::prelude::TurnStepText,
      persistence::prelude::TurnStepImage,
      persistence::prelude::TurnStepThinking,
      persistence::prelude::TurnStepToolUse,
      persistence::prelude::TurnStepToolResult,
      persistence::prelude::TurnStepStatus,
      persistence::prelude::UsageMetrics,
      shared::agent::Provider
    )
  ),
  tags(
    (name = "auth", description = "Authentication and session management"),
    (name = "public", description = "Public endpoints"),
    (name = "employees", description = "Employee management"),
    (name = "issues", description = "Issue management"),
    (name = "memory", description = "Project and employee memory"),
    (name = "projects", description = "Project management"),
    (name = "mcp_servers", description = "Configured MCP servers and run-scoped MCP enablement"),
    (name = "providers", description = "Provider management"),
    (name = "runs", description = "Run management"),
    (name = "telegram", description = "Telegram integration management"),
    (name = "skills", description = "Skill discovery")
  )
)]
struct ApiDoc;
