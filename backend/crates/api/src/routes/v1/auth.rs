use std::env;

use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use argon2::password_hash::SaltString;
use axum::Extension;
use axum::Json;
use axum::Router;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use chrono::Duration;
use chrono::Utc;
use persistence::prelude::AuthSessionModel;
use persistence::prelude::AuthSessionRepository;
use persistence::prelude::EmployeeKind;
use persistence::prelude::EmployeeModel;
use persistence::prelude::EmployeePatch;
use persistence::prelude::EmployeePermissions;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::EmployeeRepository;
use persistence::prelude::EmployeeRole;
use persistence::prelude::LoginCredentialModel;
use persistence::prelude::LoginCredentialRepository;
use rand::distr::Alphanumeric;
use rand::distr::SampleString;
use rand::rng;
use sha2::Digest;
use sha2::Sha256;

use crate::config::allow_owner_recovery_bootstrap;
use crate::config::session_cookie_same_site;
use crate::config::session_cookie_secure;
use crate::middleware::SESSION_COOKIE_NAME;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::routes::v1::employees::Employee;
use crate::state::RequestAuth;
use crate::state::RequestExtension;

pub fn public_routes() -> Router {
  Router::new()
    .route("/auth/status", get(status))
    .route("/auth/bootstrap-owner", post(bootstrap_owner))
    .route("/auth/login", post(login))
}

pub fn protected_routes() -> Router {
  Router::new().route("/auth/me", get(me)).route("/auth/logout", post(logout))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct BootstrapOwnerPayload {
  pub name:     String,
  pub icon:     String,
  pub color:    String,
  pub email:    String,
  pub password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct LoginPayload {
  pub email:    String,
  pub password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AuthStatusDto {
  pub has_owner:              bool,
  pub owner_login_configured: bool,
}

fn hash_password(password: &str) -> anyhow::Result<String> {
  let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
  Ok(Argon2::default().hash_password(password.as_bytes(), &salt)?.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
  PasswordHash::new(password_hash)
    .ok()
    .and_then(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).ok())
    .is_some()
}

fn new_session_token() -> String {
  Alphanumeric.sample_string(&mut rng(), 64)
}

pub(crate) fn hash_session_token(token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  hex::encode(hasher.finalize())
}

fn session_ttl_hours() -> i64 {
  env::var("BLPRNT_SESSION_TTL_HOURS")
    .ok()
    .and_then(|value| value.parse().ok())
    .filter(|value| *value > 0)
    .unwrap_or(24 * 7)
}

fn session_cookie(token: &str, expires_at: chrono::DateTime<Utc>) -> String {
  let mut cookie = format!(
    "{SESSION_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite={}; Max-Age={}; Expires={}",
    session_cookie_same_site().as_cookie_value(),
    (expires_at - Utc::now()).num_seconds().max(0),
    expires_at.format("%a, %d %b %Y %H:%M:%S GMT")
  );

  if session_cookie_secure() {
    cookie.push_str("; Secure");
  }

  cookie
}

fn clear_session_cookie() -> String {
  let mut cookie = format!(
    "{SESSION_COOKIE_NAME}=; Path=/; HttpOnly; SameSite={}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    session_cookie_same_site().as_cookie_value()
  );

  if session_cookie_secure() {
    cookie.push_str("; Secure");
  }

  cookie
}

async fn owner_auth_state() -> ApiResult<(Option<EmployeeRecord>, bool)> {
  let owner = EmployeeRepository::list().await?.into_iter().find(|employee| employee.role.is_owner());
  let owner_login_configured = match owner.as_ref() {
    Some(owner) => LoginCredentialRepository::find_by_employee(owner.id.clone()).await?.is_some(),
    None => false,
  };

  Ok((owner, owner_login_configured))
}

fn trimmed_non_empty(value: &str) -> Option<String> {
  let value = value.trim();
  (!value.is_empty()).then(|| value.to_string())
}

async fn update_existing_owner(owner: EmployeeRecord, payload: &BootstrapOwnerPayload) -> ApiResult<EmployeeRecord> {
  let patch = EmployeePatch {
    name: trimmed_non_empty(&payload.name),
    icon: trimmed_non_empty(&payload.icon),
    color: trimmed_non_empty(&payload.color),
    ..Default::default()
  };

  if patch.name.is_none() && patch.icon.is_none() && patch.color.is_none() {
    return Ok(owner);
  }

  Ok(EmployeeRepository::update(owner.id, patch).await?)
}

async fn create_session_response(employee: EmployeeRecord) -> ApiResult<(HeaderValue, Json<Employee>)> {
  let token = new_session_token();
  let expires_at = Utc::now() + Duration::hours(session_ttl_hours());
  let session = AuthSessionRepository::create(AuthSessionModel {
    employee_id: employee.id.clone(),
    token_hash: hash_session_token(&token),
    created_at: Utc::now(),
    expires_at,
    last_seen_at: Some(Utc::now()),
    revoked_at: None,
  })
  .await?;

  let cookie = HeaderValue::from_str(&session_cookie(&token, session.expires_at))
    .map_err(|_| ApiErrorKind::InternalServerError(serde_json::json!("failed to construct session cookie")))?;
  Ok((cookie, Json(employee.into())))
}

#[utoipa::path(
  get,
  path = "/auth/status",
  responses(
    (status = 200, description = "Inspect whether owner auth has been configured", body = AuthStatusDto),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "auth"
)]
pub(super) async fn status() -> ApiResult<Json<AuthStatusDto>> {
  let (owner, owner_login_configured) = owner_auth_state().await?;

  Ok(Json(AuthStatusDto { has_owner: owner.is_some(), owner_login_configured }))
}

#[utoipa::path(
  post,
  path = "/auth/bootstrap-owner",
  request_body = BootstrapOwnerPayload,
  responses(
    (status = 200, description = "Create the initial owner login and session", body = Employee),
    (status = 400, description = "Owner already exists or invalid credentials", body = crate::routes::errors::ApiError),
    (status = 409, description = "Email already exists", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "auth"
)]
pub(super) async fn bootstrap_owner(Json(payload): Json<BootstrapOwnerPayload>) -> ApiResult<Response> {
  if payload.email.trim().is_empty() || payload.password.trim().len() < 8 {
    return Err(
      ApiErrorKind::BadRequest(serde_json::json!("Email is required and password must be at least 8 characters"))
        .into(),
    );
  }

  let (owner, owner_login_configured) = owner_auth_state().await?;
  if owner.is_some() && owner_login_configured {
    return Err(ApiErrorKind::BadRequest(serde_json::json!("Owner login is already configured")).into());
  }
  if owner.is_some() && !owner_login_configured && !allow_owner_recovery_bootstrap() {
    return Err(
      ApiErrorKind::Forbidden(serde_json::json!(
        "Owner recovery bootstrap is disabled. Configure login locally or explicitly allow recovery bootstrap."
      ))
      .into(),
    );
  }
  if owner.is_none()
    && (payload.name.trim().is_empty() || payload.icon.trim().is_empty() || payload.color.trim().is_empty())
  {
    return Err(
      ApiErrorKind::BadRequest(serde_json::json!("Name, icon, and color are required to create the first owner"))
        .into(),
    );
  }

  let owner = match owner {
    Some(owner) => update_existing_owner(owner, &payload).await?,
    None => {
      EmployeeRepository::create(EmployeeModel {
        name: payload.name.trim().to_string(),
        kind: EmployeeKind::Person,
        role: EmployeeRole::Owner,
        title: "Owner".to_string(),
        icon: payload.icon.trim().to_string(),
        color: payload.color.trim().to_string(),
        permissions: EmployeePermissions::new(true, true),
        ..Default::default()
      })
      .await?
    }
  };

  LoginCredentialRepository::create(LoginCredentialModel {
    employee_id:   owner.id.clone(),
    email:         payload.email.trim().to_ascii_lowercase(),
    password_hash: hash_password(payload.password.trim())?,
    password_salt: String::new(),
    created_at:    Utc::now(),
    updated_at:    Utc::now(),
  })
  .await?;

  let (cookie, body) = create_session_response(owner).await?;
  let mut response = (StatusCode::OK, body).into_response();
  response.headers_mut().append(header::SET_COOKIE, cookie);
  Ok(response)
}

#[utoipa::path(
  post,
  path = "/auth/login",
  request_body = LoginPayload,
  responses(
    (status = 200, description = "Create a browser session", body = Employee),
    (status = 401, description = "Invalid credentials", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "auth"
)]
pub(super) async fn login(Json(payload): Json<LoginPayload>) -> ApiResult<Response> {
  let credential = match LoginCredentialRepository::find_by_email(&payload.email.trim().to_ascii_lowercase()).await? {
    Some(credential) => credential,
    None => {
      let (owner, owner_login_configured) = owner_auth_state().await?;
      if owner.is_some() && !owner_login_configured {
        return Err(
          ApiErrorKind::BadRequest(serde_json::json!(
            "Owner login is not configured yet. Finish setup from the bootstrap page."
          ))
          .into(),
        );
      }

      return Err(ApiErrorKind::Unauthorized(serde_json::json!("Invalid email or password")).into());
    }
  };

  if !verify_password(payload.password.trim(), &credential.password_hash) {
    return Err(ApiErrorKind::Unauthorized(serde_json::json!("Invalid email or password")).into());
  }

  let employee = EmployeeRepository::get(credential.employee_id).await?;
  let (cookie, body) = create_session_response(employee).await?;
  let mut response = (StatusCode::OK, body).into_response();
  response.headers_mut().append(header::SET_COOKIE, cookie);
  Ok(response)
}

#[utoipa::path(
  get,
  path = "/auth/me",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 200, description = "Fetch the current authenticated employee", body = Employee),
    (status = 400, description = "Missing auth", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "auth"
)]
pub(super) async fn me(Extension(extension): Extension<RequestExtension>) -> ApiResult<Json<Employee>> {
  Ok(Json(extension.employee.into()))
}

#[utoipa::path(
  post,
  path = "/auth/logout",
  security(("blprnt_employee_id" = [])),
  responses(
    (status = 204, description = "Clear the current authenticated session"),
    (status = 400, description = "Missing auth", body = crate::routes::errors::ApiError),
    (status = 500, description = "Unexpected server error", body = crate::routes::errors::ApiError),
  ),
  tag = "auth"
)]
pub(super) async fn logout(Extension(extension): Extension<RequestExtension>) -> ApiResult<Response> {
  if let RequestAuth::Session { session_id } = extension.auth {
    let _ = AuthSessionRepository::revoke(session_id).await;
  }

  let mut response = StatusCode::NO_CONTENT.into_response();
  let cookie = HeaderValue::from_str(&clear_session_cookie())
    .map_err(|_| ApiErrorKind::InternalServerError(serde_json::json!("failed to construct session cookie")))?;
  response.headers_mut().append(header::SET_COOKIE, cookie);
  Ok(response)
}
