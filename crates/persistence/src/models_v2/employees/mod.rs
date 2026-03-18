mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::RUNS_TABLE;
use crate::prelude::Record;
use crate::prelude::RunModel;
use crate::prelude::errors::DatabaseError;
use crate::prelude::errors::DatabaseResult;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct EmployeeModel {
  pub name:            String,
  pub kind:            EmployeeKind,
  pub role:            EmployeeRole,
  pub title:           String,
  pub status:          EmployeeStatus,
  pub icon:            String,
  pub color:           String,
  pub capabilities:    Vec<String>,
  pub permissions:     EmployeePermissions,
  pub reports_to:      Option<EmployeeId>,
  pub provider_config: Option<EmployeeProviderConfig>,
  pub runtime_config:  Option<EmployeeRuntimeConfig>,
  pub created_at:      DateTime<Utc>,
  pub updated_at:      DateTime<Utc>,
}

impl Default for EmployeeModel {
  fn default() -> Self {
    Self {
      name:            String::new(),
      kind:            EmployeeKind::default(),
      role:            EmployeeRole::Custom("employee".to_string()),
      title:           String::new(),
      status:          EmployeeStatus::Idle,
      icon:            String::new(),
      color:           String::new(),
      capabilities:    Vec::new(),
      reports_to:      None,
      provider_config: None,
      runtime_config:  None,
      permissions:     EmployeePermissions::default(),
      created_at:      Utc::now(),
      updated_at:      Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct EmployeeRecord {
  pub id:              EmployeeId,
  pub name:            String,
  pub kind:            EmployeeKind,
  pub role:            EmployeeRole,
  pub title:           String,
  pub status:          EmployeeStatus,
  pub icon:            String,
  pub color:           String,
  pub capabilities:    Vec<String>,
  pub permissions:     EmployeePermissions,
  pub reports_to:      Option<EmployeeId>,
  pub provider_config: Option<EmployeeProviderConfig>,
  pub runtime_config:  Option<EmployeeRuntimeConfig>,
  pub created_at:      DateTime<Utc>,
  pub updated_at:      DateTime<Utc>,
  pub reports:         Vec<EmployeeId>,
}

impl EmployeeRecord {
  pub fn is_owner(&self) -> bool {
    self.role.is_owner()
  }

  pub fn is_ceo(&self) -> bool {
    self.role.is_ceo()
  }

  pub fn can_hire(&self) -> bool {
    self.role.can_hire() || self.permissions.can_hire
  }

  pub fn can_update_employee(&self) -> bool {
    self.role.can_update_employee() || self.permissions.can_update_employee
  }
}

impl From<EmployeeRecord> for EmployeeModel {
  fn from(record: EmployeeRecord) -> Self {
    Self {
      name:            record.name,
      kind:            record.kind,
      role:            record.role,
      title:           record.title,
      status:          record.status,
      icon:            record.icon,
      color:           record.color,
      capabilities:    record.capabilities,
      permissions:     record.permissions,
      reports_to:      record.reports_to,
      provider_config: record.provider_config,
      runtime_config:  record.runtime_config,
      created_at:      record.created_at,
      updated_at:      record.updated_at,
    }
  }
}

impl EmployeeModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    RunModel::migrate(db).await?;

    db.query(
      format!("DEFINE FIELD IF NOT EXISTS reports_to ON TABLE {EMPLOYEES_TABLE} TYPE option<record<{EMPLOYEES_TABLE}>> REFERENCE ON DELETE UNSET;"),
    )
    .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS reports ON TABLE {EMPLOYEES_TABLE} COMPUTED <~{EMPLOYEES_TABLE};"))
      .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS runs ON TABLE {EMPLOYEES_TABLE} COMPUTED <~{RUNS_TABLE};")).await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct EmployeePatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:            Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub role:            Option<EmployeeRole>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title:           Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status:          Option<EmployeeStatus>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub icon:            Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub color:           Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub capabilities:    Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reports_to:      Option<Option<EmployeeId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provider_config: Option<EmployeeProviderConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runtime_config:  Option<EmployeeRuntimeConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:      Option<DateTime<Utc>>,
}

pub struct EmployeeRepository;

impl EmployeeRepository {
  pub async fn create(model: EmployeeModel) -> DatabaseResult<EmployeeRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(EMPLOYEES_TABLE, Uuid::new_v7());

    let record: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::FailedToCreateEmployee(e.into()))?
      .ok_or(DatabaseError::EmployeeNotFoundAfterCreation)?;

    Self::get(record.id.into()).await
  }

  pub async fn get(id: EmployeeId) -> DatabaseResult<EmployeeRecord> {
    let db = SurrealConnection::db().await;
    let record: EmployeeRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::FailedToGetEmployee(e.into()))?
      .ok_or(DatabaseError::EmployeeNotFound)?;
    Ok(record)
  }

  pub async fn list() -> DatabaseResult<Vec<EmployeeRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<EmployeeRecord> = db
      .query(format!("SELECT * FROM {EMPLOYEES_TABLE}"))
      .await
      .map_err(|e| DatabaseError::FailedToListEmployees(e.into()))?
      .take(0)
      .map_err(|e| DatabaseError::FailedToListEmployees(e.into()))?;

    Ok(records)
  }

  pub async fn update(id: EmployeeId, patch: EmployeePatch) -> DatabaseResult<EmployeeRecord> {
    let db = SurrealConnection::db().await;
    let mut model: EmployeeModel = Self::get(id.clone()).await?.into();

    if let Some(name) = patch.name {
      model.name = name;
    }

    if let Some(role) = patch.role {
      model.role = role;
    }

    if let Some(title) = patch.title {
      model.title = title;
    }

    if let Some(status) = patch.status {
      model.status = status;
    }

    if let Some(icon) = patch.icon {
      model.icon = icon;
    }

    if let Some(color) = patch.color {
      model.color = color;
    }

    if let Some(capabilities) = patch.capabilities {
      model.capabilities = capabilities;
    }

    if let Some(reports_to) = patch.reports_to {
      model.reports_to = reports_to;
    }

    if let Some(provider_config) = patch.provider_config {
      model.provider_config = Some(provider_config);
    }

    if let Some(runtime_config) = patch.runtime_config {
      model.runtime_config = Some(runtime_config);
    }

    model.updated_at = Utc::now();
    let record: EmployeeRecord = db
      .update(id.clone().inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::FailedToUpdateEmployee(e.into()))?
      .ok_or(DatabaseError::EmployeeNotFound)?;

    Ok(record)
  }

  pub async fn delete(id: EmployeeId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::FailedToDeleteEmployee(e.into()))?
      .ok_or(DatabaseError::EmployeeNotFound)?;

    Ok(())
  }
}
