use axum::Json;
use axum::Router;
use axum::routing::get;
use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::ApiKey;
use utoipa::openapi::security::ApiKeyValue;
use utoipa::openapi::security::SecurityScheme;

pub fn routes() -> Router {
  Router::new().route("/openapi.json", get(openapi_json))
}

#[utoipa::path(
  get,
  path = "/openapi.json",
  responses(
    (status = 200, description = "OpenAPI document for the HTTP API", body = Object),
  ),
  tag = "public"
)]
async fn openapi_json() -> Json<serde_json::Value> {
  Json(serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI document should serialize"))
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
    openapi_json,
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
    super::telegram::list_telegram_links,
    super::telegram::telegram_webhook
  ),
  components(
    schemas(
      crate::routes::errors::ApiError,
      super::auth::AuthStatusDto,
      crate::dto::RunDto,
      crate::dto::RunSummaryDto,
      crate::dto::TurnDto,
      crate::dto::TelegramConfigDto,
      crate::dto::TelegramLinkDto,
      crate::dto::TelegramLinkCodeDto,
      crate::dto::TelegramMessageCorrelationDto,
      persistence::prelude::RunStatus,
      persistence::prelude::RunTrigger,
      persistence::prelude::TelegramDeliveryMode,
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
    (name = "public", description = "Public endpoints"),
    (name = "employees", description = "Employee management"),
    (name = "issues", description = "Issue management"),
    (name = "memory", description = "Project and employee memory"),
    (name = "projects", description = "Project management"),
    (name = "providers", description = "Provider management"),
    (name = "runs", description = "Run management"),
    (name = "telegram", description = "Telegram integration management"),
    (name = "skills", description = "Skill discovery")
  )
)]
struct ApiDoc;
