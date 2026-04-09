mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use shared::errors::DatabaseEntity;
use shared::errors::DatabaseError;
use shared::errors::DatabaseOperation;
use shared::errors::DatabaseResult;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::EmployeeId;
use crate::prelude::IssueId;
use crate::prelude::Record;
use crate::prelude::RunId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramConfigModel {
  pub bot_username: Option<String>,
  pub parse_mode:   Option<TelegramParseMode>,
  pub enabled:      bool,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl Default for TelegramConfigModel {
  fn default() -> Self {
    Self {
      bot_username: None,
      parse_mode:   None,
      enabled:      false,
      created_at:   Utc::now(),
      updated_at:   Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramConfigRecord {
  pub id:           TelegramConfigId,
  pub bot_username: Option<String>,
  pub parse_mode:   Option<TelegramParseMode>,
  pub enabled:      bool,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl From<TelegramConfigRecord> for TelegramConfigModel {
  fn from(record: TelegramConfigRecord) -> Self {
    Self {
      bot_username: record.bot_username,
      parse_mode:   record.parse_mode,
      enabled:      record.enabled,
      created_at:   record.created_at,
      updated_at:   record.updated_at,
    }
  }
}

impl TelegramConfigModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TELEGRAM_CONFIGS_TABLE} SCHEMALESS;")).await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TELEGRAM_LINKS_TABLE} SCHEMALESS;")).await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TELEGRAM_LINK_CODES_TABLE} SCHEMALESS;")).await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TELEGRAM_ISSUE_WATCHES_TABLE} SCHEMALESS;")).await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {TELEGRAM_MESSAGE_CORRELATIONS_TABLE} SCHEMALESS;")).await?;
    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct TelegramConfigPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bot_username: Option<Option<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parse_mode:   Option<Option<TelegramParseMode>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled:      Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:   Option<DateTime<Utc>>,
}

pub struct TelegramConfigRepository;

impl TelegramConfigRepository {
  pub async fn create(model: TelegramConfigModel) -> DatabaseResult<TelegramConfigRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TELEGRAM_CONFIGS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::TelegramConfig })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: TelegramConfigId) -> DatabaseResult<TelegramConfigRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramConfig })
  }

  pub async fn get_active() -> DatabaseResult<Option<TelegramConfigRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {TELEGRAM_CONFIGS_TABLE} WHERE enabled = true ORDER BY updated_at DESC LIMIT 1"))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })
  }

  pub async fn get_latest() -> DatabaseResult<Option<TelegramConfigRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {TELEGRAM_CONFIGS_TABLE} ORDER BY updated_at DESC LIMIT 1"))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })
  }

  pub async fn upsert_singleton(model: TelegramConfigModel) -> DatabaseResult<TelegramConfigRecord> {
    if let Some(existing) = Self::get_latest().await? {
      Self::update(
        existing.id,
        TelegramConfigPatch {
          bot_username: Some(model.bot_username),
          parse_mode:   Some(model.parse_mode),
          enabled:      Some(model.enabled),
          updated_at:   Some(Utc::now()),
        },
      )
      .await
    } else {
      Self::create(model).await
    }
  }

  pub async fn update(id: TelegramConfigId, patch: TelegramConfigPatch) -> DatabaseResult<TelegramConfigRecord> {
    let db = SurrealConnection::db().await;
    let mut model: TelegramConfigModel = Self::get(id.clone()).await?.into();

    if let Some(bot_username) = patch.bot_username {
      model.bot_username = bot_username;
    }
    if let Some(parse_mode) = patch.parse_mode {
      model.parse_mode = parse_mode;
    }
    if let Some(enabled) = patch.enabled {
      model.enabled = enabled;
    }

    model.updated_at = patch.updated_at.unwrap_or_else(Utc::now);

    let _: Record = db
      .update(id.inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramConfig,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramConfig })?;

    Self::get(id).await
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkCodeModel {
  pub employee_id:     EmployeeId,
  pub code_hash:       String,
  pub code_last4:      String,
  pub expires_at:      DateTime<Utc>,
  pub claimed_at:      Option<DateTime<Utc>>,
  pub claimed_chat_id: Option<i64>,
  pub claimed_user_id: Option<i64>,
  pub created_at:      DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkCodeRecord {
  pub id:              TelegramLinkCodeId,
  pub employee_id:     EmployeeId,
  pub code_hash:       String,
  pub code_last4:      String,
  pub expires_at:      DateTime<Utc>,
  pub claimed_at:      Option<DateTime<Utc>>,
  pub claimed_chat_id: Option<i64>,
  pub claimed_user_id: Option<i64>,
  pub created_at:      DateTime<Utc>,
}

pub struct TelegramLinkCodeRepository;

impl TelegramLinkCodeRepository {
  pub async fn create(model: TelegramLinkCodeModel) -> DatabaseResult<TelegramLinkCodeRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TELEGRAM_LINK_CODES_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLinkCode,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::TelegramLinkCode })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: TelegramLinkCodeId) -> DatabaseResult<TelegramLinkCodeRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLinkCode,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramLinkCode })
  }

  pub async fn find_claimable_by_hash(code_hash: &str) -> DatabaseResult<Option<TelegramLinkCodeRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {TELEGRAM_LINK_CODES_TABLE} WHERE code_hash = $code_hash AND claimed_at = NONE AND expires_at >= time::now() ORDER BY created_at DESC LIMIT 1"
    ))
    .bind(("code_hash", code_hash.to_string()))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramLinkCode,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramLinkCode,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })
  }

  pub async fn claim(id: TelegramLinkCodeId, chat_id: i64, user_id: i64) -> DatabaseResult<TelegramLinkCodeRecord> {
    let db = SurrealConnection::db().await;
    let mut record = Self::get(id.clone()).await?;
    record.claimed_at = Some(Utc::now());
    record.claimed_chat_id = Some(chat_id);
    record.claimed_user_id = Some(user_id);

    let _: Record = db
      .update(id.inner())
      .merge(record.clone())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLinkCode,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramLinkCode })?;

    Self::get(id).await
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkModel {
  pub employee_id:              EmployeeId,
  pub telegram_user_id:         i64,
  pub telegram_chat_id:         i64,
  pub status:                   TelegramLinkStatus,
  pub notification_preferences: TelegramNotificationPreferences,
  pub created_at:               DateTime<Utc>,
  pub updated_at:               DateTime<Utc>,
  pub last_seen_at:             Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramLinkRecord {
  pub id:                       TelegramLinkId,
  pub employee_id:              EmployeeId,
  pub telegram_user_id:         i64,
  pub telegram_chat_id:         i64,
  pub status:                   TelegramLinkStatus,
  pub notification_preferences: TelegramNotificationPreferences,
  pub created_at:               DateTime<Utc>,
  pub updated_at:               DateTime<Utc>,
  pub last_seen_at:             Option<DateTime<Utc>>,
}

pub struct TelegramLinkRepository;

impl TelegramLinkRepository {
  pub async fn create(model: TelegramLinkModel) -> DatabaseResult<TelegramLinkRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TELEGRAM_LINKS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLink,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::TelegramLink })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: TelegramLinkId) -> DatabaseResult<TelegramLinkRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLink,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramLink })
  }

  pub async fn list_for_employee(employee_id: EmployeeId) -> DatabaseResult<Vec<TelegramLinkRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {TELEGRAM_LINKS_TABLE} WHERE employee_id = $employee_id ORDER BY created_at DESC"))
      .bind(("employee_id", employee_id))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLink,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLink,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })
  }

  pub async fn find_by_chat_and_user(chat_id: i64, user_id: i64) -> DatabaseResult<Option<TelegramLinkRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {TELEGRAM_LINKS_TABLE} WHERE telegram_chat_id = $chat_id AND telegram_user_id = $user_id AND status = 'linked' LIMIT 1"
    ))
    .bind(("chat_id", chat_id))
    .bind(("user_id", user_id))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramLink,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramLink,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })
  }

  pub async fn upsert_link(
    employee_id: EmployeeId,
    telegram_user_id: i64,
    telegram_chat_id: i64,
  ) -> DatabaseResult<TelegramLinkRecord> {
    let db = SurrealConnection::db().await;

    if let Some(existing) = Self::find_by_chat_and_user(telegram_chat_id, telegram_user_id).await? {
      let mut updated = existing.clone();
      updated.employee_id = employee_id;
      updated.status = TelegramLinkStatus::Linked;
      updated.last_seen_at = Some(Utc::now());
      updated.updated_at = Utc::now();

      let _: Record = db
        .update(existing.id.inner())
        .merge(updated)
        .await
        .map_err(|e| DatabaseError::Operation {
          entity:    DatabaseEntity::TelegramLink,
          operation: DatabaseOperation::Update,
          source:    e.into(),
        })?
        .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramLink })?;

      return Self::get(existing.id).await;
    }

    Self::create(TelegramLinkModel {
      employee_id,
      telegram_user_id,
      telegram_chat_id,
      status: TelegramLinkStatus::Linked,
      notification_preferences: TelegramNotificationPreferences::default(),
      created_at: Utc::now(),
      updated_at: Utc::now(),
      last_seen_at: Some(Utc::now()),
    })
    .await
  }

  pub async fn touch_last_seen(id: TelegramLinkId) -> DatabaseResult<TelegramLinkRecord> {
    let db = SurrealConnection::db().await;
    let mut model = Self::get(id.clone()).await?;
    model.last_seen_at = Some(Utc::now());
    model.updated_at = Utc::now();

    let _: Record = db
      .update(id.inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramLink,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramLink })?;

    Self::get(id).await
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramIssueWatchModel {
  pub employee_id: EmployeeId,
  pub issue_id:    IssueId,
  pub created_at:  DateTime<Utc>,
  pub updated_at:  DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramIssueWatchRecord {
  pub id:          TelegramIssueWatchId,
  pub employee_id: EmployeeId,
  pub issue_id:    IssueId,
  pub created_at:  DateTime<Utc>,
  pub updated_at:  DateTime<Utc>,
}

pub struct TelegramIssueWatchRepository;

impl TelegramIssueWatchRepository {
  pub async fn create(model: TelegramIssueWatchModel) -> DatabaseResult<TelegramIssueWatchRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TELEGRAM_ISSUE_WATCHES_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramIssueWatch,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::TelegramIssueWatch })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: TelegramIssueWatchId) -> DatabaseResult<TelegramIssueWatchRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramIssueWatch,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramIssueWatch })
  }

  pub async fn find(employee_id: EmployeeId, issue_id: IssueId) -> DatabaseResult<Option<TelegramIssueWatchRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {TELEGRAM_ISSUE_WATCHES_TABLE} WHERE employee_id = $employee_id AND issue_id = $issue_id LIMIT 1"
    ))
    .bind(("employee_id", employee_id))
    .bind(("issue_id", issue_id))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::TelegramIssueWatch,
      operation: DatabaseOperation::Get,
      source:    e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::TelegramIssueWatch,
      operation: DatabaseOperation::Get,
      source:    e.into(),
    })
  }

  pub async fn list_for_issue(issue_id: IssueId) -> DatabaseResult<Vec<TelegramIssueWatchRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {TELEGRAM_ISSUE_WATCHES_TABLE} WHERE issue_id = $issue_id ORDER BY created_at ASC"))
      .bind(("issue_id", issue_id))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramIssueWatch,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramIssueWatch,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })
  }

  pub async fn watch(employee_id: EmployeeId, issue_id: IssueId) -> DatabaseResult<TelegramIssueWatchRecord> {
    if let Some(existing) = Self::find(employee_id.clone(), issue_id.clone()).await? {
      return Ok(existing);
    }

    Self::create(TelegramIssueWatchModel { employee_id, issue_id, created_at: Utc::now(), updated_at: Utc::now() })
      .await
  }

  pub async fn unwatch(employee_id: EmployeeId, issue_id: IssueId) -> DatabaseResult<bool> {
    let Some(existing) = Self::find(employee_id, issue_id).await? else {
      return Ok(false);
    };

    let db = SurrealConnection::db().await;
    let deleted: Option<Record> = db.delete(existing.id.inner()).await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::TelegramIssueWatch,
      operation: DatabaseOperation::Delete,
      source:    e.into(),
    })?;
    Ok(deleted.is_some())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramMessageCorrelationModel {
  pub telegram_chat_id:    i64,
  pub telegram_message_id: i64,
  pub direction:           TelegramMessageDirection,
  pub kind:                TelegramCorrelationKind,
  pub issue_id:            Option<IssueId>,
  pub run_id:              Option<RunId>,
  pub employee_id:         Option<EmployeeId>,
  pub text_preview:        Option<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TelegramMessageCorrelationRecord {
  pub id:                  TelegramMessageCorrelationId,
  pub telegram_chat_id:    i64,
  pub telegram_message_id: i64,
  pub direction:           TelegramMessageDirection,
  pub kind:                TelegramCorrelationKind,
  pub issue_id:            Option<IssueId>,
  pub run_id:              Option<RunId>,
  pub employee_id:         Option<EmployeeId>,
  pub text_preview:        Option<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

pub struct TelegramMessageCorrelationRepository;

impl TelegramMessageCorrelationRepository {
  pub async fn create(model: TelegramMessageCorrelationModel) -> DatabaseResult<TelegramMessageCorrelationRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(TELEGRAM_MESSAGE_CORRELATIONS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramMessageCorrelation,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::TelegramMessageCorrelation })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: TelegramMessageCorrelationId) -> DatabaseResult<TelegramMessageCorrelationRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramMessageCorrelation,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramMessageCorrelation })
  }

  pub async fn find_by_chat_message(
    chat_id: i64,
    message_id: i64,
  ) -> DatabaseResult<Option<TelegramMessageCorrelationRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {TELEGRAM_MESSAGE_CORRELATIONS_TABLE} WHERE telegram_chat_id = $chat_id AND telegram_message_id = $message_id LIMIT 1"
    ))
    .bind(("chat_id", chat_id))
    .bind(("message_id", message_id))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramMessageCorrelation,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramMessageCorrelation,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })
  }

  pub async fn list_outbound_for_run(run_id: RunId) -> DatabaseResult<Vec<TelegramMessageCorrelationRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {TELEGRAM_MESSAGE_CORRELATIONS_TABLE} WHERE run_id = $run_id AND direction = $direction ORDER BY created_at DESC"
    ))
    .bind(("run_id", run_id))
    .bind(("direction", TelegramMessageDirection::Outbound))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramMessageCorrelation,
      operation: DatabaseOperation::List,
      source: e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::TelegramMessageCorrelation,
      operation: DatabaseOperation::List,
      source: e.into(),
    })
  }

  pub async fn update(
    id: TelegramMessageCorrelationId,
    patch: TelegramMessageCorrelationPatch,
  ) -> DatabaseResult<TelegramMessageCorrelationRecord> {
    let db = SurrealConnection::db().await;
    let mut model = Self::get(id.clone()).await?;

    if let Some(issue_id) = patch.issue_id {
      model.issue_id = issue_id;
    }
    if let Some(run_id) = patch.run_id {
      model.run_id = run_id;
    }
    if let Some(employee_id) = patch.employee_id {
      model.employee_id = employee_id;
    }
    if let Some(text_preview) = patch.text_preview {
      model.text_preview = text_preview;
    }
    model.updated_at = patch.updated_at.unwrap_or_else(Utc::now);

    let _: Record = db
      .update(id.inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::TelegramMessageCorrelation,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::TelegramMessageCorrelation })?;

    Self::get(id).await
  }
}
