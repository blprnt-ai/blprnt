#![allow(clippy::field_reassign_with_default)]

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::DateTime;
use chrono::Utc;
use common::agent::ToolId;
use common::models::ReasoningEffort;
use common::session_dispatch::prelude::SubagentDetails;
use common::shared::prelude::HistoryVisibility;
use common::shared::prelude::MessageContent;
use common::shared::prelude::MessageRole;
use common::shared::prelude::PartialVisibility;
use common::shared::prelude::SurrealId;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::models_v2::Record;

pub const MESSAGES_TABLE: &str = "messages";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageModelV2 {
  pub rel_id:           String,
  #[specta(type = String)]
  pub turn_id:          Uuid,
  #[specta(type = String)]
  pub step_id:          Uuid,
  pub role:             MessageRole,
  pub content:          MessageContent,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_usage:      Option<u32>,
  pub visibility:       HistoryVisibility,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reasoning_effort: Option<ReasoningEffort>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub memory_dirty:     Option<bool>,
  #[specta(type = i32)]
  pub created_at:       DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:       DateTime<Utc>,
}

impl Default for MessageModelV2 {
  fn default() -> Self {
    Self {
      rel_id:           "".to_string(),
      turn_id:          Uuid::nil(),
      step_id:          Uuid::nil(),
      token_usage:      None,
      role:             MessageRole::User,
      content:          MessageContent::default(),
      visibility:       HistoryVisibility::None,
      reasoning_effort: None,
      memory_dirty:     None,
      created_at:       Utc::now(),
      updated_at:       Utc::now(),
    }
  }
}

impl MessageModelV2 {
  pub fn with_memory_dirty(mut self, memory_dirty: bool) -> Self {
    self.memory_dirty = Some(memory_dirty);
    self
  }

  pub fn user_full_message(content: MessageContent, reasoning_effort: ReasoningEffort) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::User;
    model.visibility = HistoryVisibility::Full;
    model.content = content;
    model.reasoning_effort = Some(reasoning_effort);

    model
  }

  pub fn user_assistant_message(content: MessageContent, reasoning_effort: ReasoningEffort) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::User;
    model.visibility = HistoryVisibility::Assistant;
    model.content = content;
    model.reasoning_effort = Some(reasoning_effort);
    model
  }

  pub fn assistant_full_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::Assistant;
    model.visibility = HistoryVisibility::Full;
    model.content = content;

    model
  }

  pub fn assistant_user_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::Assistant;
    model.visibility = HistoryVisibility::User;
    model.content = content;

    model
  }

  pub fn assistant_none_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::Assistant;
    model.visibility = HistoryVisibility::None;
    model.content = content;

    model
  }

  pub fn assistant_tool_response_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::Assistant;
    model.visibility = HistoryVisibility::Partial(PartialVisibility::ToolResult);
    model.content = content;

    model
  }

  pub fn assistant_assistant_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::Assistant;
    model.visibility = HistoryVisibility::Assistant;
    model.content = content;

    model
  }

  pub fn system_user_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::System;
    model.visibility = HistoryVisibility::User;
    model.content = content;

    model
  }

  pub fn system_assistant_message(content: MessageContent) -> Self {
    let mut model = Self::default();
    model.role = MessageRole::System;
    model.visibility = HistoryVisibility::Assistant;
    model.content = content;

    model
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct MessageRecord {
  pub id:               SurrealId,
  pub rel_id:           String,
  #[specta(type = String)]
  pub turn_id:          Uuid,
  #[specta(type = String)]
  pub step_id:          Uuid,
  pub role:             MessageRole,
  pub content:          MessageContent,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_usage:      Option<u32>,
  pub visibility:       HistoryVisibility,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reasoning_effort: Option<ReasoningEffort>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub memory_dirty:     Option<bool>,
  #[specta(type = i32)]
  pub created_at:       DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:       DateTime<Utc>,
  pub session_id:       SurrealId,
  pub parent_id:        Option<SurrealId>,
}

impl From<MessageRecord> for MessageModelV2 {
  fn from(record: MessageRecord) -> Self {
    Self {
      rel_id:           record.rel_id,
      turn_id:          record.turn_id,
      step_id:          record.step_id,
      role:             record.role,
      content:          record.content,
      token_usage:      record.token_usage,
      visibility:       record.visibility,
      reasoning_effort: record.reasoning_effort,
      memory_dirty:     record.memory_dirty,
      created_at:       record.created_at,
      updated_at:       record.updated_at,
    }
  }
}

impl MessageRecord {
  pub fn content(&self) -> &MessageContent {
    &self.content
  }

  pub fn role(&self) -> &MessageRole {
    &self.role
  }

  pub fn visibility(&self) -> &HistoryVisibility {
    &self.visibility
  }

  pub fn reasoning_effort(&self) -> &Option<ReasoningEffort> {
    &self.reasoning_effort
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn token_usage(&self) -> &Option<u32> {
    &self.token_usage
  }

  pub fn memory_dirty(&self) -> Option<bool> {
    self.memory_dirty
  }

  pub fn is_user(&self) -> bool {
    self.role == MessageRole::User
  }

  pub fn is_assistant(&self) -> bool {
    self.role == MessageRole::Assistant
  }

  pub fn is_user_visible(&self) -> bool {
    self.visibility == HistoryVisibility::User
  }

  pub fn is_tool_request(&self) -> bool {
    self.visibility == PartialVisibility::ToolRequest.into()
  }
}

impl MessageModelV2 {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS parent_id ON TABLE messages TYPE option<record<messages>> REFERENCE ON DELETE UNSET;
    "#,
    )
    .await?;
    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS session_id ON TABLE messages TYPE option<record<sessions>> REFERENCE ON DELETE CASCADE;
    "#,
    )
    .await?;

    // This can take a while to index, so we do it in a background task
    tokio::spawn({
      let db = db.clone();
      async move {
        let _ = db
          .query(
            r#"
          DEFINE INDEX IF NOT EXISTS dirty_idx ON TABLE messages COLUMNS memory_dirty, session_id.parent_id;
        "#,
          )
          .await;
      }
    });

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct MessagePatchV2 {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content:          Option<MessageContent>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reasoning_effort: Option<ReasoningEffort>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub visibility:       Option<HistoryVisibility>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub token_usage:      Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub memory_dirty:     Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub updated_at:       Option<DateTime<Utc>>,
}

pub struct MessageRepositoryV2;

impl MessageRepositoryV2 {
  pub async fn create(model: MessageModelV2, session_id: SurrealId) -> Result<MessageRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(MESSAGES_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create message"))?;
    let result: Option<Record> = db
      .query("UPDATE $message_id SET session_id = $session_id")
      .bind(("message_id", record_id.clone()))
      .bind(("session_id", session_id.inner()))
      .await?
      .take(0)
      .context("Failed to relate message to session")?;

    match result {
      Some(result) => Self::get(result.id.into()).await,
      None => {
        bail!("Failed to create message");
      }
    }
  }

  pub async fn relate_parent(parent_id: SurrealId, message_id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    db.query("UPDATE $message_id SET parent_id = $parent_id")
      .bind(("message_id", message_id.inner()))
      .bind(("parent_id", parent_id.inner()))
      .await?;

    Ok(())
  }

  pub async fn get(id: SurrealId) -> Result<MessageRecord> {
    let db = SurrealConnection::db().await;
    db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Message not found"))
  }

  pub async fn get_by_rel_id(rel_id: String) -> Result<MessageRecord> {
    let db = SurrealConnection::db().await;
    let result: Option<MessageRecord> = db
      .query(format!(r#"SELECT * FROM {MESSAGES_TABLE} WHERE rel_id = $rel_id"#))
      .bind(("rel_id", rel_id))
      .await?
      .take(0)
      .context("Failed to get message by rel_id")?;

    result.ok_or(anyhow::anyhow!("Message not found"))
  }

  pub async fn get_pending_tool_requests(
    session_id: SurrealId,
  ) -> Result<Vec<(SurrealId, String, ToolId, Uuid, Uuid, Option<SubagentDetails>)>> {
    let db = SurrealConnection::db().await;
    let query = r#"
        SELECT 
          id,
          response_id,
          content.id AS tool_use_id,
          content.tool_id AS tool_id,
          content.subagent_details AS subagent_details,
          turn_id,
          step_id
        FROM $session_id.messages.*
        WHERE content.tool_id IS NOT NONE
        AND (visibility = $visibility_user OR visibility = $visibility_tool_request)
        ORDER BY id ASC
      "#;

    #[derive(serde::Deserialize, SurrealValue)]
    struct QueryResult {
      id:               SurrealId,
      tool_use_id:      String,
      tool_id:          ToolId,
      subagent_details: Option<SubagentDetails>,
      turn_id:          Uuid,
      step_id:          Uuid,
    }

    let result: Vec<QueryResult> = db
      .query(query)
      .bind(("session_id", session_id.inner()))
      .bind(("visibility_user", HistoryVisibility::User))
      .bind(("visibility_tool_request", HistoryVisibility::Partial(PartialVisibility::ToolRequest)))
      .await
      .context("history::get_pending_tool_requests::query")?
      .take(0)
      .context("history::get_pending_tool_requests::take")?;

    Ok(result.into_iter().map(|r| (r.id, r.tool_use_id, r.tool_id, r.turn_id, r.step_id, r.subagent_details)).collect())
  }

  pub async fn get_pending_user_interaction_requests(
    session_id: SurrealId,
  ) -> Result<Vec<(SurrealId, String, Uuid, Uuid)>> {
    let db = SurrealConnection::db().await;
    let query = r#"
      r#"
        SELECT 
          id,
          content.id AS tool_use_id,
          turn_id,
          step_id
        FROM $session_id.messages.*
        WHERE content.tool_id = $tool_id
        AND visibility = $visibility
        ORDER BY id ASC
      "#;

    #[derive(serde::Deserialize, SurrealValue)]
    struct QueryResult {
      id:          SurrealId,
      tool_use_id: String,
      turn_id:     Uuid,
      step_id:     Uuid,
    }

    let result: Vec<QueryResult> = db
      .query(query)
      .bind(("session_id", session_id.inner()))
      .bind(("tool_id", ToolId::AskQuestion))
      .bind(("visibility", HistoryVisibility::User))
      .await
      .context("history::get_pending_user_interaction_requests::query")?
      .take(0)
      .context("history::get_pending_user_interaction_requests::take")?;

    Ok(result.into_iter().map(|r| (r.id, r.tool_use_id, r.turn_id, r.step_id)).collect())
  }

  pub async fn first_by_step_id(step_id: Uuid) -> Result<Option<MessageRecord>> {
    let db = SurrealConnection::db().await;
    let query = r#"SELECT * FROM messages WHERE step_id = $step_id ORDER BY id ASC LIMIT 1"#;

    db.query(query)
      .bind(("step_id", step_id))
      .await
      .context("history::first_by_step_id::query")?
      .take(0)
      .context("history::first_by_step_id::take")
  }

  pub async fn last_10_user_messages(session_id: SurrealId) -> Result<Vec<MessageRecord>> {
    let db = SurrealConnection::db().await;
    let query = r#"SELECT * FROM $session_id.messages.* WHERE role = $role ORDER BY id DESC LIMIT 10"#;
    db.query(query)
      .bind(("session_id", session_id.inner()))
      .bind(("role", MessageRole::User))
      .await?
      .take(0)
      .context("history::last_10_user_messages::take")
  }

  pub async fn last_user_messages(session_id: SurrealId, limit: usize) -> Result<Vec<MessageRecord>> {
    let db = SurrealConnection::db().await;
    let query = r#"SELECT * FROM $session_id.messages.* WHERE role = $role ORDER BY id DESC LIMIT $limit"#;
    db.query(query)
      .bind(("session_id", session_id.inner()))
      .bind(("role", MessageRole::User))
      .bind(("limit", limit))
      .await?
      .take(0)
      .context("history::last_user_messages::take")
  }

  pub async fn list(session_id: SurrealId) -> Result<Vec<MessageRecord>> {
    let db = SurrealConnection::db().await;

    let result = db.query("SELECT * FROM $session_id.messages.*").bind(("session_id", session_id.inner())).await;

    result?.take(0).context("Failed to list messages")
  }

  pub async fn list_dirty_session_ids() -> Result<Vec<SurrealId>> {
    let db = SurrealConnection::db().await;

    db.query(
      r#"
        SELECT VALUE session_id
        FROM messages
        WHERE memory_dirty = true
          AND session_id IS NOT NONE
          AND session_id.parent_id IS NONE
          AND content.Text.text IS NOT NONE
        GROUP BY session_id
        ORDER BY session_id ASC
      "#,
    )
    .await?
    .take(0)
    .context("Failed to list dirty session ids")
  }

  pub async fn list_dirty_for_session(session_id: SurrealId) -> Result<Vec<MessageRecord>> {
    let db = SurrealConnection::db().await;
    let query = r#"
      SELECT *
      FROM $session_id.messages.*
      WHERE memory_dirty = true
      AND content.Text.text IS NOT NONE
      ORDER BY created_at ASC, id ASC
    "#;

    let result: Vec<MessageRecord> = db
      .query(query)
      .bind(("session_id", session_id.inner()))
      .await?
      .take(0)
      .context("Failed to list dirty messages for session")?;

    Ok(result.into_iter().filter(|message| message.content.is_text()).collect())
  }

  pub async fn mark_all_clean_for_session(session_id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    db.query("UPDATE $session_id.messages.* SET memory_dirty = false").bind(("session_id", session_id.inner())).await?;

    Ok(())
  }

  pub async fn update(id: SurrealId, patch: MessagePatchV2) -> Result<MessageRecord> {
    let db = SurrealConnection::db().await;
    let mut message_model: MessageModelV2 = Self::get(id.clone()).await?.into();
    message_model.updated_at = Utc::now();

    if let Some(content) = patch.content {
      message_model.content = content;
    }

    if let Some(reasoning_effort) = patch.reasoning_effort {
      message_model.reasoning_effort = Some(reasoning_effort);
    }

    if let Some(visibility) = patch.visibility {
      message_model.visibility = visibility;
    }

    if let Some(token_usage) = patch.token_usage {
      message_model.token_usage = Some(token_usage);
    }

    if let Some(memory_dirty) = patch.memory_dirty {
      message_model.memory_dirty = Some(memory_dirty);
    }

    message_model.updated_at = Utc::now();

    let _: Option<Record> = db.update(id.inner()).merge(message_model).await?;

    Self::get(id).await
  }

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete message"))?;

    Ok(())
  }

  pub async fn truncate_to(session_id: SurrealId, to_history_id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;

    let query = r#"
        DELETE FROM messages
        WHERE session_id = $session_id
        AND id >= $to_history_id
      "#;

    db.query(query)
      .bind(("session_id", session_id.inner()))
      .bind(("to_history_id", to_history_id.inner()))
      .await
      .context("history::truncate_to::delete_history")?;

    tracing::info!("Truncate to fast, delete history");

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use common::shared::prelude::MessageText;
  use common::shared::prelude::PathList;
  use serde_json::json;

  use super::*;
  use crate::models_v2::ProjectModelV2;
  use crate::models_v2::ProjectRepositoryV2;
  use crate::models_v2::SessionModelV2;
  use crate::models_v2::SessionRecord;
  use crate::models_v2::SessionRepositoryV2;

  async fn create_session() -> SessionRecord {
    let project = ProjectRepositoryV2::create(ProjectModelV2::new("project".into(), PathList::default(), None))
      .await
      .expect("project should be created");
    SessionRepositoryV2::create(SessionModelV2::default(), project.id).await.expect("session should be created")
  }

  #[test]
  fn message_model_deserializes_legacy_rows_without_memory_dirty() {
    let value = json!({
      "rel_id": "",
      "turn_id": Uuid::nil().to_string(),
      "step_id": Uuid::nil().to_string(),
      "role": "user",
      "content": {
        "type": "text",
        "text": "legacy",
        "signature": null
      },
      "visibility": "none",
      "created_at": Utc::now(),
      "updated_at": Utc::now()
    });

    let model: MessageModelV2 = serde_json::from_value(value).expect("legacy message model should deserialize");

    assert_eq!(model.memory_dirty, None);
  }

  #[test]
  fn with_memory_dirty_sets_marker() {
    let model = MessageModelV2::default().with_memory_dirty(true);

    assert_eq!(model.memory_dirty, Some(true));
  }

  #[tokio::test]
  async fn list_dirty_for_session_returns_only_dirty_messages_in_created_order() {
    let session = create_session().await;

    let dirty_first = MessageRepositoryV2::create(
      MessageModelV2::assistant_full_message(MessageText { text: "dirty-first".into(), signature: None }.into())
        .with_memory_dirty(true),
      session.id.clone(),
    )
    .await
    .expect("first dirty message should be created");
    MessageRepositoryV2::create(
      MessageModelV2::assistant_full_message(MessageText { text: "clean-middle".into(), signature: None }.into()),
      session.id.clone(),
    )
    .await
    .expect("clean message should be created");
    let dirty_second = MessageRepositoryV2::create(
      MessageModelV2::assistant_full_message(MessageText { text: "dirty-second".into(), signature: None }.into())
        .with_memory_dirty(true),
      session.id.clone(),
    )
    .await
    .expect("second dirty message should be created");

    let dirty_messages =
      MessageRepositoryV2::list_dirty_for_session(session.id).await.expect("dirty messages should load");

    assert_eq!(
      dirty_messages.iter().map(|message| message.id.clone()).collect::<Vec<_>>(),
      vec![dirty_first.id, dirty_second.id]
    );
    assert!(dirty_messages.iter().all(|message| message.memory_dirty() == Some(true)));
  }
}
