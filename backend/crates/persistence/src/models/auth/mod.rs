mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use shared::errors::DatabaseConflict;
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
use crate::prelude::EMPLOYEES_TABLE;
use crate::prelude::EmployeeId;
use crate::prelude::Record;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct LoginCredentialModel {
  pub employee_id:    EmployeeId,
  pub email:          String,
  pub password_hash:  String,
  pub password_salt:  String,
  pub created_at:     DateTime<Utc>,
  pub updated_at:     DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct LoginCredentialRecord {
  pub id:             LoginCredentialId,
  pub employee_id:    EmployeeId,
  pub email:          String,
  pub password_hash:  String,
  pub password_salt:  String,
  pub created_at:     DateTime<Utc>,
  pub updated_at:     DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct AuthSessionModel {
  pub employee_id:       EmployeeId,
  pub token_hash:        String,
  pub created_at:        DateTime<Utc>,
  pub expires_at:        DateTime<Utc>,
  pub last_seen_at:      Option<DateTime<Utc>>,
  pub revoked_at:        Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct AuthSessionRecord {
  pub id:                AuthSessionId,
  pub employee_id:       EmployeeId,
  pub token_hash:        String,
  pub created_at:        DateTime<Utc>,
  pub expires_at:        DateTime<Utc>,
  pub last_seen_at:      Option<DateTime<Utc>>,
  pub revoked_at:        Option<DateTime<Utc>>,
}

impl LoginCredentialModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {LOGIN_CREDENTIALS_TABLE} SCHEMALESS;")).await?;
    db.query(format!("DEFINE TABLE IF NOT EXISTS {AUTH_SESSIONS_TABLE} SCHEMALESS;")).await?;
    db.query(
      format!(
        "DEFINE FIELD IF NOT EXISTS employee_id ON TABLE {LOGIN_CREDENTIALS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE CASCADE;"
      ),
    )
    .await?;
    db.query(
      format!(
        "DEFINE FIELD IF NOT EXISTS employee_id ON TABLE {AUTH_SESSIONS_TABLE} TYPE record<{EMPLOYEES_TABLE}> REFERENCE ON DELETE CASCADE;"
      ),
    )
    .await?;
    Ok(())
  }
}

pub struct LoginCredentialRepository;

impl LoginCredentialRepository {
  pub async fn create(model: LoginCredentialModel) -> DatabaseResult<LoginCredentialRecord> {
    let db = SurrealConnection::db().await;

    if Self::find_by_email(&model.email).await?.is_some() {
      return Err(DatabaseError::Conflict {
        entity: DatabaseEntity::LoginCredential,
        reason: DatabaseConflict::AlreadyExists,
      });
    }

    let record_id = RecordId::new(LOGIN_CREDENTIALS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Create,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::LoginCredential })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: LoginCredentialId) -> DatabaseResult<LoginCredentialRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::LoginCredential })
  }

  pub async fn find_by_email(email: &str) -> DatabaseResult<Option<LoginCredentialRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {LOGIN_CREDENTIALS_TABLE} WHERE email = $email LIMIT 1"))
      .bind(("email", email.to_string()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })
  }

  pub async fn find_by_employee(employee_id: EmployeeId) -> DatabaseResult<Option<LoginCredentialRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {LOGIN_CREDENTIALS_TABLE} WHERE employee_id = $employee_id LIMIT 1"))
      .bind(("employee_id", employee_id))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::LoginCredential,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })
  }
}

pub struct AuthSessionRepository;

impl AuthSessionRepository {
  pub async fn create(model: AuthSessionModel) -> DatabaseResult<AuthSessionRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(AUTH_SESSIONS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::AuthSession,
        operation: DatabaseOperation::Create,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::AuthSession })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: AuthSessionId) -> DatabaseResult<AuthSessionRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::AuthSession,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::AuthSession })
  }

  pub async fn find_active_by_token_hash(token_hash: &str) -> DatabaseResult<Option<AuthSessionRecord>> {
    let db = SurrealConnection::db().await;
    let now = Utc::now();
    db.query(format!(
      "SELECT * FROM {AUTH_SESSIONS_TABLE} WHERE token_hash = $token_hash AND revoked_at = NONE AND expires_at > $now LIMIT 1"
    ))
    .bind(("token_hash", token_hash.to_string()))
    .bind(("now", now))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::AuthSession,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity: DatabaseEntity::AuthSession,
      operation: DatabaseOperation::Get,
      source: e.into(),
    })
  }

  pub async fn revoke(id: AuthSessionId) -> DatabaseResult<AuthSessionRecord> {
    let db = SurrealConnection::db().await;
    let record = Self::get(id.clone()).await?;
    let _: Record = db
      .update(id.clone().inner())
      .merge(AuthSessionModel {
        employee_id: record.employee_id,
        token_hash: record.token_hash,
        created_at: record.created_at,
        expires_at: record.expires_at,
        last_seen_at: record.last_seen_at,
        revoked_at: Some(Utc::now()),
      })
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::AuthSession,
        operation: DatabaseOperation::Update,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::AuthSession })?;

    Self::get(id).await
  }

  pub async fn touch(id: AuthSessionId) -> DatabaseResult<AuthSessionRecord> {
    let db = SurrealConnection::db().await;
    let record = Self::get(id.clone()).await?;
    let _: Record = db
      .update(id.clone().inner())
      .merge(AuthSessionModel {
        employee_id: record.employee_id,
        token_hash: record.token_hash,
        created_at: record.created_at,
        expires_at: record.expires_at,
        last_seen_at: Some(Utc::now()),
        revoked_at: record.revoked_at,
      })
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::AuthSession,
        operation: DatabaseOperation::Update,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::AuthSession })?;

    Self::get(id).await
  }
}