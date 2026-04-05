use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::middleware;
use axum::routing::get;
use axum::routing::post;
use chrono::Utc;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::TelegramConfigModel;
use persistence::prelude::TelegramConfigRepository;
use persistence::prelude::TelegramCorrelationKind;
use persistence::prelude::TelegramDeliveryMode;
use persistence::prelude::TelegramLinkRepository;
use persistence::prelude::TelegramMessageCorrelationPatch;
use persistence::prelude::TelegramMessageCorrelationRepository;
use persistence::prelude::TelegramParseMode;
use serde::Deserialize;
use serde_json::json;
use vault::set_stronghold_secret;

use crate::dto::TelegramConfigDto;
use crate::dto::TelegramLinkCodeDto;
use crate::dto::TelegramLinkDto;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;
use crate::telegram;

const TELEGRAM_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

pub fn protected_routes() -> Router {
  Router::new()
    .route("/integrations/telegram/config", get(get_telegram_config))
    .route("/integrations/telegram/config", post(upsert_telegram_config))
    .route("/integrations/telegram/link-codes", post(create_telegram_link_code))
    .route("/integrations/telegram/links/{employee_id}", get(list_telegram_links))
    .layer(middleware::from_fn(crate::middleware::owner_only))
}

pub fn public_routes() -> Router {
  Router::new().route("/integrations/telegram/webhook", post(telegram_webhook))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpsertTelegramConfigPayload {
  pub bot_token:      String,
  pub webhook_secret: String,
  pub bot_username:   Option<String>,
  pub webhook_url:    Option<String>,
  pub delivery_mode:  TelegramDeliveryMode,
  pub parse_mode:     Option<TelegramParseMode>,
  pub enabled:        bool,
}

#[utoipa::path(
  get,
  path = "/integrations/telegram/config",
  security(("blprnt_employee_id" = [])),
  responses((status = 200, body = Option<TelegramConfigDto>)),
  tag = "telegram"
)]
pub(super) async fn get_telegram_config() -> ApiResult<Json<Option<TelegramConfigDto>>> {
  Ok(Json(TelegramConfigRepository::get_latest().await?.map(Into::into)))
}

#[utoipa::path(
  post,
  path = "/integrations/telegram/config",
  security(("blprnt_employee_id" = [])),
  request_body = UpsertTelegramConfigPayload,
  responses((status = 200, body = TelegramConfigDto)),
  tag = "telegram"
)]
pub(super) async fn upsert_telegram_config(Json(payload): Json<UpsertTelegramConfigPayload>) -> ApiResult<Json<TelegramConfigDto>> {
  let record = TelegramConfigRepository::upsert_singleton(TelegramConfigModel {
    bot_username: payload.bot_username,
    webhook_url: payload.webhook_url,
    delivery_mode: payload.delivery_mode,
    parse_mode: payload.parse_mode,
    enabled: payload.enabled,
    created_at: Utc::now(),
    updated_at: Utc::now(),
  })
  .await?;

  set_stronghold_secret(
    vault::Vault::Key,
    crate::telegram::telegram_bot_token_key(record.id.uuid()),
    &payload.bot_token,
  )
  .await
  .map_err(|error| {
    ApiErrorKind::InternalServerError(json!({"message": "failed to store bot token", "source": error.to_string()}))
  })?;
  set_stronghold_secret(
    vault::Vault::Key,
    crate::telegram::telegram_webhook_secret_key(record.id.uuid()),
    &payload.webhook_secret,
  )
  .await
  .map_err(|error| {
    ApiErrorKind::InternalServerError(json!({"message": "failed to store webhook secret", "source": error.to_string()}))
  })?;

  Ok(Json(record.into()))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateTelegramLinkCodePayload {
  pub employee_id: Uuid,
}

#[derive(Debug, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateTelegramLinkCodeResponse {
  pub code:   String,
  pub record: TelegramLinkCodeDto,
}

#[utoipa::path(
  post,
  path = "/integrations/telegram/link-codes",
  security(("blprnt_employee_id" = [])),
  request_body = CreateTelegramLinkCodePayload,
  responses((status = 200, body = CreateTelegramLinkCodeResponse)),
  tag = "telegram"
)]
pub(super) async fn create_telegram_link_code(
  Extension(_extension): Extension<RequestExtension>,
  Json(payload): Json<CreateTelegramLinkCodePayload>,
) -> ApiResult<Json<CreateTelegramLinkCodeResponse>> {
  let (code, record) = telegram::create_link_code(payload.employee_id.into()).await.map_err(|error| {
    ApiErrorKind::InternalServerError(json!({"message": "failed to create link code", "source": error.to_string()}))
  })?;
  Ok(Json(CreateTelegramLinkCodeResponse { code, record: record.into() }))
}

#[utoipa::path(
  get,
  path = "/integrations/telegram/links/{employee_id}",
  security(("blprnt_employee_id" = [])),
  params(("employee_id" = Uuid, Path, description = "Employee id")),
  responses((status = 200, body = [TelegramLinkDto])),
  tag = "telegram"
)]
pub(super) async fn list_telegram_links(Path(employee_id): Path<Uuid>) -> ApiResult<Json<Vec<TelegramLinkDto>>> {
  Ok(Json(
    TelegramLinkRepository::list_for_employee(employee_id.into())
      .await?
      .into_iter()
      .map(Into::into)
      .collect(),
  ))
}

#[derive(Debug, Deserialize)]
pub struct TelegramWebhookPayload {
  pub message: Option<TelegramIncomingMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramIncomingMessage {
  pub message_id: i64,
  pub text: Option<String>,
  pub chat: TelegramChat,
  pub from: Option<TelegramUser>,
  pub reply_to_message: Option<TelegramReplyMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
  pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUser {
  pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct TelegramReplyMessage {
  pub message_id: i64,
}

#[utoipa::path(
  post,
  path = "/integrations/telegram/webhook",
  request_body = serde_json::Value,
  responses((status = 200, body = serde_json::Value), (status = 401, body = crate::routes::errors::ApiError)),
  tag = "telegram"
)]
pub(super) async fn telegram_webhook(
  headers: HeaderMap,
  Json(payload): Json<TelegramWebhookPayload>,
) -> ApiResult<Json<serde_json::Value>> {
  let header_secret = headers.get(TELEGRAM_SECRET_HEADER).and_then(|value| value.to_str().ok());
  let verified = telegram::verify_webhook_secret(header_secret).await.map_err(|error| {
    ApiErrorKind::InternalServerError(json!({"message": "failed to validate webhook", "source": error.to_string()}))
  })?;

  if !verified {
    return Err(ApiErrorKind::Unauthorized(json!("Invalid telegram webhook secret")).into());
  }

  let Some(message) = payload.message else {
    return Ok(Json(json!({"ok": true, "ignored": true})));
  };

  let telegram_user_id = message.from.as_ref().map(|user| user.id);
  let linked_employee = match telegram_user_id {
    Some(user_id) => TelegramLinkRepository::find_by_chat_and_user(message.chat.id, user_id).await?,
    None => None,
  };

  let reply_context = if let Some(reply) = message.reply_to_message.as_ref() {
    TelegramMessageCorrelationRepository::find_by_chat_message(message.chat.id, reply.message_id).await?
  } else {
    None
  };
  let trimmed_text = message.text.as_deref().map(str::trim);
  let inferred_kind = if trimmed_text.is_some_and(|text| text.starts_with("/link ")) {
    TelegramCorrelationKind::LinkCode
  } else if let Some(ref correlation) = reply_context {
    correlation.kind.clone()
  } else {
    TelegramCorrelationKind::Unknown
  };

  let inbound_correlation = telegram::correlate_inbound_message(
    message.chat.id,
    message.message_id,
    linked_employee.as_ref().map(|link| link.employee_id.clone()),
    message.text.clone(),
    inferred_kind,
    reply_context.as_ref().and_then(|correlation| correlation.issue_id.clone()),
    reply_context.as_ref().and_then(|correlation| correlation.run_id.clone()),
  )
  .await
  .map_err(|error| ApiErrorKind::InternalServerError(json!({"message": "failed to persist inbound message", "source": error.to_string()})))?;

  if let Some(ref link) = linked_employee {
    let _ = TelegramLinkRepository::touch_last_seen(link.id.clone()).await;
  }

  if let Some(text) = trimmed_text {
    if let Some(code) = text.strip_prefix("/link ") {
      let Some(user_id) = telegram_user_id else {
        return Err(ApiErrorKind::BadRequest(json!("Telegram user id is required for link flow")).into());
      };

      let linked = telegram::link_from_code(code.trim(), message.chat.id, user_id).await.map_err(|error| {
        ApiErrorKind::InternalServerError(json!({"message": "failed to link telegram chat", "source": error.to_string()}))
      })?;

      if let Some(ref link) = linked {
        let _ = TelegramMessageCorrelationRepository::update(
          inbound_correlation.id,
          TelegramMessageCorrelationPatch {
            employee_id: Some(Some(link.employee_id.clone())),
            updated_at: Some(Utc::now()),
            ..Default::default()
          },
        )
        .await;
      }

      let delivery_error = telegram::send_link_feedback(
        message.chat.id,
        message.message_id,
        linked.as_ref().map(|link| link.employee_id.clone()),
        linked.is_some(),
      )
      .await;

      return Ok(Json(json!({
        "ok": true,
        "linked": linked.is_some(),
        "delivery_error": delivery_error
      })));
    }
  }

  if let Some(link) = linked_employee.clone() {
    let employee = persistence::prelude::EmployeeRepository::get(link.employee_id).await?;
    let outcome = telegram::handle_linked_message(
      employee,
      message.chat.id,
      message.message_id,
      trimmed_text,
      reply_context.as_ref().and_then(|correlation| correlation.issue_id.clone()),
      reply_context.as_ref().and_then(|correlation| correlation.run_id.clone()),
    )
    .await
    .map_err(|error| ApiErrorKind::BadRequest(json!({"message": error.to_string()})))?;

    return Ok(Json(json!({
      "ok": true,
      "linked": true,
      "reply_context_found": reply_context.is_some(),
      "delivery_error": outcome.delivery_error,
    })));
  }

  Ok(Json(json!({
    "ok": true,
    "linked": linked_employee.is_some(),
    "reply_context_found": reply_context.is_some()
  })))
}