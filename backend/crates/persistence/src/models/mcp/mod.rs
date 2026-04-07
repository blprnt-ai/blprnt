mod types;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use shared::errors::DatabaseEntity;
use shared::errors::DatabaseError;
use shared::errors::DatabaseOperation;
use shared::errors::DatabaseResult;
use shared::tools::McpServerAuthState;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;
pub use types::*;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::ProjectId;
use crate::prelude::Record;
use crate::prelude::RunId;
use crate::prelude::RUNS_TABLE;
use crate::prelude::PROJECTS_TABLE;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct McpServerModel {
  pub project_id:    ProjectId,
  pub display_name:  String,
  pub description:   String,
  pub transport:     String,
  pub endpoint_url:  String,
  #[serde(default)]
  pub auth_state:    McpServerAuthState,
  #[serde(default)]
  pub auth_summary:  Option<String>,
  #[serde(default)]
  pub enabled:       bool,
  pub created_at:    DateTime<Utc>,
  pub updated_at:    DateTime<Utc>,
}

impl McpServerModel {
  pub fn new(project_id: ProjectId, display_name: String, description: String, transport: String, endpoint_url: String) -> Self {
    Self {
      project_id,
      display_name,
      description,
      transport,
      endpoint_url,
      auth_state: McpServerAuthState::NotConnected,
      auth_summary: None,
      enabled: true,
      created_at: Utc::now(),
      updated_at: Utc::now(),
    }
  }

  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {MCP_SERVERS_TABLE} SCHEMALESS;")).await?;
    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS project_id ON TABLE {MCP_SERVERS_TABLE} TYPE record<{PROJECTS_TABLE}> REFERENCE ON DELETE CASCADE;"
    ))
    .await?;
    db.query(format!("DEFINE INDEX IF NOT EXISTS idx_mcp_servers_project ON TABLE {MCP_SERVERS_TABLE} FIELDS project_id;")).await?;

    db.query(format!("DEFINE TABLE IF NOT EXISTS {RUN_ENABLED_MCP_SERVERS_TABLE} SCHEMALESS;")).await?;
    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS run_id ON TABLE {RUN_ENABLED_MCP_SERVERS_TABLE} TYPE record<{RUNS_TABLE}> REFERENCE ON DELETE CASCADE;"
    ))
    .await?;
    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS server_id ON TABLE {RUN_ENABLED_MCP_SERVERS_TABLE} TYPE record<{MCP_SERVERS_TABLE}> REFERENCE ON DELETE CASCADE;"
    ))
    .await?;
    db.query(format!("DEFINE INDEX IF NOT EXISTS idx_run_enabled_mcp_servers_run ON TABLE {RUN_ENABLED_MCP_SERVERS_TABLE} FIELDS run_id;")).await?;
    db.query(format!(
      "DEFINE INDEX IF NOT EXISTS idx_run_enabled_mcp_servers_unique ON TABLE {RUN_ENABLED_MCP_SERVERS_TABLE} FIELDS run_id, server_id UNIQUE;"
    ))
    .await?;

    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct McpServerRecord {
  pub id:           McpServerId,
  pub project_id:   ProjectId,
  pub display_name: String,
  pub description:  String,
  pub transport:    String,
  pub endpoint_url: String,
  pub auth_state:   McpServerAuthState,
  pub auth_summary: Option<String>,
  pub enabled:      bool,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl From<McpServerRecord> for McpServerModel {
  fn from(record: McpServerRecord) -> Self {
    Self {
      project_id: record.project_id,
      display_name: record.display_name,
      description: record.description,
      transport: record.transport,
      endpoint_url: record.endpoint_url,
      auth_state: record.auth_state,
      auth_summary: record.auth_summary,
      enabled: record.enabled,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export, optional_fields)]
pub struct McpServerPatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub display_name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:  Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub transport:    Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub endpoint_url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_state:   Option<McpServerAuthState>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub auth_summary: Option<Option<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub enabled:      Option<bool>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunEnabledMcpServerModel {
  pub run_id:      RunId,
  pub server_id:   McpServerId,
  pub enabled_at:  DateTime<Utc>,
}

impl RunEnabledMcpServerModel {
  pub fn new(run_id: RunId, server_id: McpServerId) -> Self {
    Self { run_id, server_id, enabled_at: Utc::now() }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct RunEnabledMcpServerRecord {
  pub id:         RunEnabledMcpServerId,
  pub run_id:     RunId,
  pub server_id:  McpServerId,
  pub enabled_at: DateTime<Utc>,
}

pub struct McpServerRepository;

impl McpServerRepository {
  pub async fn create(model: McpServerModel) -> DatabaseResult<McpServerRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(MCP_SERVERS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::McpServer,
        operation: DatabaseOperation::Create,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::McpServer })?;

    Self::get(record_id.into()).await
  }

  pub async fn get(id: McpServerId) -> DatabaseResult<McpServerRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::McpServer,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::McpServer })
  }

  pub async fn list(project_id: Option<ProjectId>) -> DatabaseResult<Vec<McpServerRecord>> {
    let db = SurrealConnection::db().await;
    let mut query = if project_id.is_some() {
      db.query(format!("SELECT * FROM {MCP_SERVERS_TABLE} WHERE project_id = $project_id ORDER BY created_at ASC"))
    } else {
      db.query(format!("SELECT * FROM {MCP_SERVERS_TABLE} ORDER BY created_at ASC"))
    };

    if let Some(project_id) = project_id {
      query = query.bind(("project_id", project_id.inner()));
    }

    query
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::McpServer,
        operation: DatabaseOperation::List,
        source: e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::McpServer,
        operation: DatabaseOperation::List,
        source: e.into(),
      })
  }

  pub async fn update(id: McpServerId, patch: McpServerPatch) -> DatabaseResult<McpServerRecord> {
    let db = SurrealConnection::db().await;
    let mut model: McpServerModel = Self::get(id.clone()).await?.into();

    if let Some(display_name) = patch.display_name {
      model.display_name = display_name;
    }
    if let Some(description) = patch.description {
      model.description = description;
    }
    if let Some(transport) = patch.transport {
      model.transport = transport;
    }
    if let Some(endpoint_url) = patch.endpoint_url {
      model.endpoint_url = endpoint_url;
    }
    if let Some(auth_state) = patch.auth_state {
      model.auth_state = auth_state;
    }
    if let Some(auth_summary) = patch.auth_summary {
      model.auth_summary = auth_summary;
    }
    if let Some(enabled) = patch.enabled {
      model.enabled = enabled;
    }
    model.updated_at = Utc::now();

    let _: Record = db
      .update(id.inner())
      .merge(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::McpServer,
        operation: DatabaseOperation::Update,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::McpServer })?;

    Self::get(id).await
  }
}

pub struct RunEnabledMcpServerRepository;

impl RunEnabledMcpServerRepository {
  pub async fn enable(run_id: RunId, server_id: McpServerId) -> DatabaseResult<RunEnabledMcpServerRecord> {
    let db = SurrealConnection::db().await;
    let existing: Option<RunEnabledMcpServerRecord> = db
      .query(format!(
        "SELECT * FROM {RUN_ENABLED_MCP_SERVERS_TABLE} WHERE run_id = $run_id AND server_id = $server_id LIMIT 1"
      ))
      .bind(("run_id", run_id.inner()))
      .bind(("server_id", server_id.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?;

    if let Some(existing) = existing {
      return Ok(existing);
    }

    let record_id = RecordId::new(RUN_ENABLED_MCP_SERVERS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(RunEnabledMcpServerModel::new(run_id, server_id))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::Create,
        source: e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::RunEnabledMcpServer })?;

    let record_id: RunEnabledMcpServerId = record_id.into();
    let record: Option<RunEnabledMcpServerRecord> = db.select(record_id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::Get,
        source: e.into(),
      })?
      ;

    record.ok_or(DatabaseError::NotFound { entity: DatabaseEntity::RunEnabledMcpServer })
  }

  pub async fn list_for_run(run_id: RunId) -> DatabaseResult<Vec<RunEnabledMcpServerRecord>> {
    let db = SurrealConnection::db().await;
    db.query(format!("SELECT * FROM {RUN_ENABLED_MCP_SERVERS_TABLE} WHERE run_id = $run_id ORDER BY enabled_at ASC"))
      .bind(("run_id", run_id.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::List,
        source: e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity: DatabaseEntity::RunEnabledMcpServer,
        operation: DatabaseOperation::List,
        source: e.into(),
      })
  }
}