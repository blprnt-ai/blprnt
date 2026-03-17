use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::DateTime;
use chrono::Utc;
use common::agent::AgentKind;
use common::models::ReasoningEffort;
use common::shared::prelude::QueueMode;
use common::shared::prelude::SurrealId;
use common::tools::PlanDocumentStatus;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::models_v2::Record;

pub const SESSIONS_TABLE: &str = "sessions";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SessionPlan {
  pub id:     String,
  pub status: PlanDocumentStatus,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SessionModelV2 {
  pub name:               String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:        Option<String>,
  pub agent_kind:         AgentKind,
  pub yolo:               bool,
  pub read_only:          bool,
  pub network_access:     bool,
  pub reasoning_effort:   ReasoningEffort,
  pub queue_mode:         Option<QueueMode>,
  pub token_usage:        u32,
  #[serde(default)]
  pub model_override:     String,
  #[serde(skip_serializing_if = "Option::is_none", default, alias = "personality_id")]
  pub personality_key:    Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default)]
  pub web_search_enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plan:               Option<SessionPlan>,
  #[specta(type = i32)]
  pub created_at:         DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:         DateTime<Utc>,
}

impl Default for SessionModelV2 {
  fn default() -> Self {
    Self {
      name:               String::new(),
      description:        None,
      agent_kind:         AgentKind::Crew,
      yolo:               false,
      read_only:          false,
      network_access:     true,
      reasoning_effort:   ReasoningEffort::Medium,
      queue_mode:         Some(QueueMode::Queue),
      token_usage:        0,
      model_override:     String::new(),
      personality_key:    None,
      web_search_enabled: None,
      plan:               None,
      created_at:         Utc::now(),
      updated_at:         Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SessionRecord {
  pub id:                 SurrealId,
  pub name:               String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:        Option<String>,
  pub agent_kind:         AgentKind,
  pub yolo:               bool,
  pub read_only:          bool,
  pub network_access:     bool,
  pub reasoning_effort:   ReasoningEffort,
  pub queue_mode:         Option<QueueMode>,
  pub token_usage:        u32,
  pub model_override:     String,
  #[serde(skip_serializing_if = "Option::is_none", default, alias = "personality_id")]
  pub personality_key:    Option<String>,
  #[serde(skip_serializing_if = "Option::is_none", default, rename = "web_search_enabled")]
  pub web_search_enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub plan:               Option<SessionPlan>,
  #[specta(type = i32)]
  pub created_at:         DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:         DateTime<Utc>,
  pub project:            SurrealId,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent_id:          Option<SurrealId>,
}

impl From<SessionRecord> for SessionModelV2 {
  fn from(record: SessionRecord) -> Self {
    Self {
      name:               record.name.clone(),
      description:        record.description.clone(),
      agent_kind:         record.agent_kind,
      yolo:               record.yolo,
      read_only:          record.read_only,
      network_access:     record.network_access,
      reasoning_effort:   record.reasoning_effort,
      queue_mode:         record.queue_mode,
      token_usage:        record.token_usage,
      model_override:     record.model_override.clone(),
      personality_key:    record.personality_key,
      web_search_enabled: record.web_search_enabled,
      plan:               record.plan.clone(),
      created_at:         record.created_at,
      updated_at:         record.updated_at,
    }
  }
}

impl SessionRecord {
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn description(&self) -> &Option<String> {
    &self.description
  }

  pub fn agent_kind(&self) -> &AgentKind {
    &self.agent_kind
  }

  pub fn yolo(&self) -> &bool {
    &self.yolo
  }

  pub fn read_only(&self) -> &bool {
    &self.read_only
  }

  pub fn network_access(&self) -> &bool {
    &self.network_access
  }

  pub fn reasoning_effort(&self) -> &ReasoningEffort {
    &self.reasoning_effort
  }

  pub fn queue_mode(&self) -> &Option<QueueMode> {
    &self.queue_mode
  }

  pub fn token_usage(&self) -> &u32 {
    &self.token_usage
  }

  pub fn model_override(&self) -> &String {
    &self.model_override
  }

  pub fn personality_key(&self) -> &Option<String> {
    &self.personality_key
  }

  pub fn web_search_enabled(&self) -> &Option<bool> {
    &self.web_search_enabled
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

impl SessionModelV2 {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS project ON TABLE sessions TYPE option<record<projects>> REFERENCE ON DELETE CASCADE;
    "#,
    )
    .await?;
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS parent_id ON TABLE sessions TYPE option<record<sessions>> REFERENCE ON DELETE CASCADE;
    "#,
    )
    .await?;
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS messages ON TABLE sessions COMPUTED <~messages;
    "#,
    )
    .await?;
    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct SessionPatchV2 {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:               Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description:        Option<Option<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub agent_kind:         Option<AgentKind>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub yolo:               Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub read_only:          Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_access:     Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none", alias = "personality_id")]
  pub personality_key:    Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model_id:           Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reasoning_effort:   Option<ReasoningEffort>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub queue_mode:         Option<QueueMode>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_usage:        Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model_override:     Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub web_search_enabled: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at:         Option<DateTime<Utc>>,
}

pub struct SessionRepositoryV2;

impl SessionRepositoryV2 {
  pub async fn create(model: SessionModelV2, project_id: SurrealId) -> Result<SessionRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(SESSIONS_TABLE, Uuid::new_v7());

    let session: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create session"))?;

    let result: Option<Record> = db
      .query("UPDATE $sessions_id SET project = $project_id")
      .bind(("sessions_id", session))
      .bind(("project_id", project_id.inner()))
      .await?
      .take(0)
      .context("Failed to relate session to project")?;

    match result {
      Some(result) => Self::get(result.id.into()).await,
      None => {
        bail!("Failed to create session");
      }
    }
  }

  pub async fn relate_parent(parent_id: SurrealId, session_id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    db.query("UPDATE $session_id SET parent_id = $parent_id")
      .bind(("session_id", session_id.inner()))
      .bind(("parent_id", parent_id.inner()))
      .await?;

    Ok(())
  }

  pub async fn get(id: SurrealId) -> Result<SessionRecord> {
    let db = SurrealConnection::db().await;

    let query = r#"SELECT *, project.* FROM $session_id"#;

    let result: Option<SessionRecord> =
      db.query(query).bind(("session_id", id.inner())).await?.take(0).context("Failed to get session")?;

    result.ok_or(anyhow::anyhow!("Session not found"))
  }

  pub async fn list(project_id: SurrealId) -> Result<Vec<SessionRecord>> {
    let db = SurrealConnection::db().await;
    let results = db.query("SELECT * FROM $project_id.child_sessions.*").bind(("project_id", project_id.inner())).await;

    results?.take(0).context("Failed to list sessions")
  }

  pub async fn last_used_session_id() -> Option<SessionRecord> {
    let db = SurrealConnection::db().await;
    let result: Option<SessionRecord> =
      db.query("SELECT * FROM sessions ORDER BY updated_at DESC LIMIT 1").await.ok()?.take(0).ok()?;

    result
  }

  pub async fn update(id: SurrealId, patch: SessionPatchV2) -> Result<SessionRecord> {
    let db = SurrealConnection::db().await;

    let mut session_model: SessionModelV2 = Self::get(id.clone()).await?.into();
    session_model.updated_at = Utc::now();

    if let Some(name) = patch.name {
      session_model.name = name;
    }

    if let Some(description) = patch.description {
      session_model.description = description;
    }

    if let Some(agent_kind) = patch.agent_kind {
      session_model.agent_kind = agent_kind;
    }

    if let Some(yolo) = patch.yolo {
      session_model.yolo = yolo;
    }

    if let Some(read_only) = patch.read_only {
      session_model.read_only = read_only;
    }

    if let Some(network_access) = patch.network_access {
      session_model.network_access = network_access;
    }

    if let Some(reasoning_effort) = patch.reasoning_effort {
      session_model.reasoning_effort = reasoning_effort;
    }

    if let Some(personality_key) = patch.personality_key {
      session_model.personality_key = Some(personality_key);
    }

    if let Some(queue_mode) = patch.queue_mode {
      session_model.queue_mode = Some(queue_mode);
    }

    if let Some(token_usage) = patch.token_usage {
      session_model.token_usage = token_usage;
    }

    if let Some(model_override) = patch.model_override {
      session_model.model_override = model_override;
    }

    if let Some(web_search_enabled) = patch.web_search_enabled {
      session_model.web_search_enabled = Some(web_search_enabled);
    }

    session_model.updated_at = Utc::now();

    let _: Option<Record> = db.update(id.inner()).merge(session_model).await?;

    Self::get(id).await
  }

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete session"))?;

    Ok(())
  }
}
