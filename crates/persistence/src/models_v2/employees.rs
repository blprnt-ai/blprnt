mod employee_run_turns;
mod employee_runs;

use std::fmt::Display;
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::Provider;
use common::shared::prelude::SurrealId;
pub use employee_run_turns::*;
pub use employee_runs::*;
use macros::SurrealEnumValue;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::Record;

pub const EMPLOYEES_TABLE: &str = "employees";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
pub enum EmployeeStatus {
  Idle,
  Running,
  Terminated,
}

impl Display for EmployeeStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeStatus::Idle => write!(f, "idle"),
      EmployeeStatus::Running => write!(f, "running"),
      EmployeeStatus::Terminated => write!(f, "terminated"),
    }
  }
}

impl FromStr for EmployeeStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "idle" => Ok(EmployeeStatus::Idle),
      "running" => Ok(EmployeeStatus::Running),
      "terminated" => Ok(EmployeeStatus::Terminated),
      _ => Err(anyhow::anyhow!("Invalid employee status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeProviderConfig {
  pub provider: Provider,
  pub slug:     String,
}

impl Default for EmployeeProviderConfig {
  fn default() -> Self {
    Self { provider: Provider::Mock, slug: String::new() }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRuntimeConfig {
  pub heartbeat_interval_sec: i32,
  pub heartbeat_prompt:       String,
  pub wake_on_demand:         bool,
  pub max_concurrent_runs:    i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeModel {
  pub name:            String,
  pub role:            String,
  pub title:           String,
  pub status:          EmployeeStatus,
  pub icon:            String,
  pub color:           String,
  pub capabilities:    Vec<String>,
  pub reports_to:      Option<SurrealId>,
  pub provider_config: EmployeeProviderConfig,
  pub runtime_config:  EmployeeRuntimeConfig,
  #[specta(type = i32)]
  pub created_at:      DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:      DateTime<Utc>,
}

impl Default for EmployeeModel {
  fn default() -> Self {
    Self {
      name:            String::new(),
      role:            String::new(),
      title:           String::new(),
      status:          EmployeeStatus::Idle,
      icon:            String::new(),
      color:           String::new(),
      capabilities:    Vec::new(),
      reports_to:      None,
      provider_config: EmployeeProviderConfig::default(),
      runtime_config:  EmployeeRuntimeConfig::default(),
      created_at:      Utc::now(),
      updated_at:      Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeeRecord {
  pub id:              SurrealId,
  pub name:            String,
  pub role:            String,
  pub title:           String,
  pub status:          EmployeeStatus,
  pub icon:            String,
  pub color:           String,
  pub capabilities:    Vec<String>,
  pub reports_to:      Option<SurrealId>,
  pub provider_config: EmployeeProviderConfig,
  pub runtime_config:  EmployeeRuntimeConfig,
  #[specta(type = i32)]
  pub created_at:      DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:      DateTime<Utc>,
  pub reports:         Vec<SurrealId>,
  pub company:         SurrealId,
}

impl From<EmployeeRecord> for EmployeeModel {
  fn from(record: EmployeeRecord) -> Self {
    Self {
      name:            record.name,
      role:            record.role,
      title:           record.title,
      status:          record.status,
      icon:            record.icon,
      color:           record.color,
      capabilities:    record.capabilities,
      reports_to:      record.reports_to,
      provider_config: record.provider_config,
      runtime_config:  record.runtime_config,
      created_at:      record.created_at,
      updated_at:      record.updated_at,
    }
  }
}

impl EmployeeRecord {
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn role(&self) -> &String {
    &self.role
  }

  pub fn title(&self) -> &String {
    &self.title
  }

  pub fn status(&self) -> &EmployeeStatus {
    &self.status
  }

  pub fn icon(&self) -> &String {
    &self.icon
  }

  pub fn color(&self) -> &String {
    &self.color
  }

  pub fn capabilities(&self) -> &Vec<String> {
    &self.capabilities
  }

  pub fn reports_to(&self) -> &Option<SurrealId> {
    &self.reports_to
  }

  pub fn reports(&self) -> &Vec<SurrealId> {
    &self.reports
  }

  pub fn provider_config(&self) -> &EmployeeProviderConfig {
    &self.provider_config
  }

  pub fn runtime_config(&self) -> &EmployeeRuntimeConfig {
    &self.runtime_config
  }

  pub fn company(&self) -> &SurrealId {
    &self.company
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

impl EmployeeModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    EmployeeRunModel::migrate(db).await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS reports_to ON TABLE employees TYPE option<record<employees>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS company ON TABLE employees TYPE option<record<companies>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS reports ON TABLE employees COMPUTED <~employees;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS runs ON TABLE employees COMPUTED <~employee_runs;
      "#,
    )
    .await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct EmployeePatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:            Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub role:            Option<String>,
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
  pub reports_to:      Option<SurrealId>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provider_config: Option<EmployeeProviderConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runtime_config:  Option<EmployeeRuntimeConfig>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at:      Option<DateTime<Utc>>,
}

pub struct EmployeeRepository;

impl EmployeeRepository {
  pub async fn create(model: EmployeeModel, company: SurrealId) -> Result<EmployeeRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(EMPLOYEES_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create employee"))?;

    let result: Option<Record> = db
      .query("UPDATE $employee_id SET company = $company_id")
      .bind(("employee_id", record_id.clone()))
      .bind(("company_id", company.inner()))
      .await?
      .take(0)
      .context("Failed to relate employee to company")?;

    match result {
      Some(result) => Self::get(result.id.into()).await,
      None => {
        bail!("Failed to create employee");
      }
    }
  }

  pub async fn get(id: SurrealId) -> Result<EmployeeRecord> {
    let db = SurrealConnection::db().await;
    let record: EmployeeRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Employee not found"))?;
    Ok(record)
  }

  pub async fn list(company: SurrealId) -> Result<Vec<EmployeeRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<EmployeeRecord> = db
      .query("SELECT * FROM $company_id.employees.*")
      .bind(("company_id", company.inner()))
      .await?
      .take(0)
      .context("Failed to list employees")?;
    Ok(records)
  }

  pub async fn update(id: SurrealId, patch: EmployeePatch) -> Result<EmployeeRecord> {
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
      model.reports_to = Some(reports_to);
    }

    if let Some(provider_config) = patch.provider_config {
      model.provider_config = provider_config;
    }

    if let Some(runtime_config) = patch.runtime_config {
      model.runtime_config = runtime_config;
    }

    model.updated_at = Utc::now();
    let record: EmployeeRecord =
      db.update(id.inner()).merge(model).await?.ok_or(anyhow::anyhow!("Failed to update employee"))?;

    Ok(record)
  }

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete employee"))?;

    Ok(())
  }
}
