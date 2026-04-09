use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::middleware;
use axum::routing::get;
use axum::routing::post;
use chrono::Utc;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::TelegramConfigModel;
use persistence::prelude::TelegramConfigRepository;
use persistence::prelude::TelegramCorrelationKind;
use persistence::prelude::TelegramLinkRepository;
use persistence::prelude::TelegramMessageCorrelationPatch;
use persistence::prelude::TelegramMessageCorrelationRepository;
use persistence::prelude::TelegramParseMode;
use serde::Deserialize;
use serde_json::json;
use vault::get_stronghold_secret;
use vault::set_stronghold_secret;

use crate::dto::TelegramConfigDto;
use crate::dto::TelegramLinkCodeDto;
use crate::dto::TelegramLinkDto;
use crate::routes::errors::ApiErrorKind;
use crate::routes::errors::ApiResult;
use crate::state::RequestExtension;
use crate::telegram;

pub fn protected_routes() -> Router {
  Router::new()
    .route("/integrations/telegram/config", get(get_telegram_config))
    .route("/integrations/telegram/config", post(upsert_telegram_config))
    .route("/integrations/telegram/link-codes", post(create_telegram_link_code))
    .route("/integrations/telegram/links/{employee_id}", get(list_telegram_links))
    .layer(middleware::from_fn(crate::middleware::owner_only))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpsertTelegramConfigPayload {
  pub bot_token:    String,
  pub bot_username: Option<String>,
  pub parse_mode:   Option<TelegramParseMode>,
  pub enabled:      bool,
}

pub(super) async fn get_telegram_config() -> ApiResult<Json<Option<TelegramConfigDto>>> {
  Ok(Json(TelegramConfigRepository::get_latest().await?.map(Into::into)))
}

pub(super) async fn upsert_telegram_config(
  Json(payload): Json<UpsertTelegramConfigPayload>,
) -> ApiResult<Json<TelegramConfigDto>> {
  let existing = TelegramConfigRepository::get_latest().await?;
  let trimmed_bot_token = payload.bot_token.trim();
  let should_store_bot_token = !trimmed_bot_token.is_empty();

  if !should_store_bot_token {
    let Some(existing) = existing.as_ref() else {
      return Err(ApiErrorKind::BadRequest(json!("bot_token is required when creating telegram config")).into());
    };

    let existing_token =
      get_stronghold_secret(vault::Vault::Key, crate::telegram::telegram_bot_token_key(existing.id.uuid())).await;
    let has_existing_token = existing_token.as_deref().is_some_and(|token| !token.trim().is_empty());

    if !has_existing_token {
      return Err(
        ApiErrorKind::BadRequest(json!("bot_token is required when no telegram bot token is stored yet")).into(),
      );
    }
  }

  let record = TelegramConfigRepository::upsert_singleton(TelegramConfigModel {
    bot_username: payload.bot_username,
    parse_mode:   payload.parse_mode,
    enabled:      payload.enabled,
    created_at:   Utc::now(),
    updated_at:   Utc::now(),
  })
  .await?;

  if should_store_bot_token {
    set_stronghold_secret(
      vault::Vault::Key,
      crate::telegram::telegram_bot_token_key(record.id.uuid()),
      trimmed_bot_token,
    )
    .await
    .map_err(|error| {
      ApiErrorKind::InternalServerError(json!({"message": "failed to store bot token", "source": error.to_string()}))
    })?;
  }

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

pub(super) async fn create_telegram_link_code(
  Extension(_extension): Extension<RequestExtension>,
  Json(payload): Json<CreateTelegramLinkCodePayload>,
) -> ApiResult<Json<CreateTelegramLinkCodeResponse>> {
  let (code, record) = telegram::create_link_code(payload.employee_id.into()).await.map_err(|error| {
    ApiErrorKind::InternalServerError(json!({"message": "failed to create link code", "source": error.to_string()}))
  })?;
  Ok(Json(CreateTelegramLinkCodeResponse { code, record: record.into() }))
}

pub(super) async fn list_telegram_links(Path(employee_id): Path<Uuid>) -> ApiResult<Json<Vec<TelegramLinkDto>>> {
  Ok(Json(TelegramLinkRepository::list_for_employee(employee_id.into()).await?.into_iter().map(Into::into).collect()))
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramWebhookPayload {
  pub message: Option<TelegramIncomingMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramIncomingMessage {
  pub message_id:       i64,
  pub text:             Option<String>,
  pub chat:             TelegramChat,
  pub from:             Option<TelegramUser>,
  pub reply_to_message: Option<TelegramReplyMessage>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramChat {
  pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramUser {
  pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramReplyMessage {
  pub message_id: i64,
}

pub(crate) async fn process_telegram_update(payload: TelegramWebhookPayload) -> ApiResult<Json<serde_json::Value>> {
  let Some(message) = payload.message else {
    return Ok(Json(json!({"ok": true, "ignored": true})));
  };

  if TelegramMessageCorrelationRepository::find_by_chat_message(message.chat.id, message.message_id).await?.is_some() {
    return Ok(Json(json!({"ok": true, "duplicate": true})));
  }

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
  .map_err(|error| {
    ApiErrorKind::InternalServerError(
      json!({"message": "failed to persist inbound message", "source": error.to_string()}),
    )
  })?;

  if let Some(ref link) = linked_employee {
    let _ = TelegramLinkRepository::touch_last_seen(link.id.clone()).await;
  }

  if let Some(text) = trimmed_text {
    if let Some(code) = text.strip_prefix("/link ") {
      let Some(user_id) = telegram_user_id else {
        return Err(ApiErrorKind::BadRequest(json!("Telegram user id is required for link flow")).into());
      };

      let linked = telegram::link_from_code(code.trim(), message.chat.id, user_id).await.map_err(|error| {
        ApiErrorKind::InternalServerError(
          json!({"message": "failed to link telegram chat", "source": error.to_string()}),
        )
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

  let delivery_error = if let Some(text) = trimmed_text {
    if text.starts_with('/') {
      telegram::send_unlinked_command_feedback(message.chat.id, message.message_id).await
    } else {
      None
    }
  } else {
    None
  };

  Ok(Json(json!({
    "ok": true,
    "linked": linked_employee.is_some(),
    "reply_context_found": reply_context.is_some(),
    "delivery_error": delivery_error,
  })))
}
