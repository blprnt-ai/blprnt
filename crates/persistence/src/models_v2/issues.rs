mod issue_actions;
mod issue_attachments;
mod issue_comments;

use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use common::shared::prelude::SurrealId;
pub use issue_actions::*;
pub use issue_attachments::*;
pub use issue_comments::*;
use macros::SurrealEnumValue;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::Record;

pub const ISSUES_TABLE: &str = "issues";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
pub enum IssueStatus {
  Backlog,
  Todo,
  InProgress,
  InReview,
  Blocked,
  Done,
  Cancelled,
  Archived,
}

impl Display for IssueStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IssueStatus::Backlog => write!(f, "backlog"),
      IssueStatus::Todo => write!(f, "todo"),
      IssueStatus::InProgress => write!(f, "in_progress"),
      IssueStatus::InReview => write!(f, "in_review"),
      IssueStatus::Blocked => write!(f, "blocked"),
      IssueStatus::Done => write!(f, "done"),
      IssueStatus::Cancelled => write!(f, "cancelled"),
      IssueStatus::Archived => write!(f, "archived"),
    }
  }
}

impl FromStr for IssueStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "backlog" => Ok(IssueStatus::Backlog),
      "todo" => Ok(IssueStatus::Todo),
      "in_progress" => Ok(IssueStatus::InProgress),
      "in_review" => Ok(IssueStatus::InReview),
      "blocked" => Ok(IssueStatus::Blocked),
      "done" => Ok(IssueStatus::Done),
      "cancelled" => Ok(IssueStatus::Cancelled),
      "archived" => Ok(IssueStatus::Archived),
      _ => Err(anyhow::anyhow!("Invalid issue status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
pub enum IssuePriority {
  Low,
  Medium,
  High,
  Critical,
}

impl Display for IssuePriority {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IssuePriority::Low => write!(f, "low"),
      IssuePriority::Medium => write!(f, "medium"),
      IssuePriority::High => write!(f, "high"),
      IssuePriority::Critical => write!(f, "critical"),
    }
  }
}

impl FromStr for IssuePriority {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "low" => Ok(IssuePriority::Low),
      "medium" => Ok(IssuePriority::Medium),
      "high" => Ok(IssuePriority::High),
      "critical" => Ok(IssuePriority::Critical),
      _ => Err(anyhow::anyhow!("Invalid issue priority: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueModel {
  pub issue_number: i32,
  pub identifier:   String,
  pub title:        String,
  pub description:  String,
  pub status:       IssueStatus,
  pub project:      Option<SurrealId>,
  pub parent:       Option<SurrealId>,
  pub creator:      Option<SurrealId>,
  pub assignee:     Option<SurrealId>,
  pub blocked_by:   Option<SurrealId>,
  pub priority:     IssuePriority,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:   DateTime<Utc>,
}

impl Default for IssueModel {
  fn default() -> Self {
    Self {
      issue_number: 0,
      identifier:   String::new(),
      title:        String::new(),
      description:  String::new(),
      status:       IssueStatus::Backlog,
      project:      None,
      parent:       None,
      creator:      None,
      assignee:     None,
      blocked_by:   None,
      priority:     IssuePriority::Medium,
      created_at:   Utc::now(),
      updated_at:   Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueRecord {
  pub id:           SurrealId,
  pub issue_number: i32,
  pub identifier:   String,
  pub title:        String,
  pub description:  String,
  pub status:       IssueStatus,
  pub project:      Option<SurrealId>,
  pub parent:       Option<SurrealId>,
  pub creator:      Option<SurrealId>,
  pub assignee:     Option<SurrealId>,
  pub blocked_by:   Option<SurrealId>,
  pub priority:     IssuePriority,
  #[specta(type = i32)]
  pub created_at:   DateTime<Utc>,
  #[specta(type = i32)]
  pub updated_at:   DateTime<Utc>,
}

impl From<IssueRecord> for IssueModel {
  fn from(record: IssueRecord) -> Self {
    Self {
      issue_number: record.issue_number,
      identifier:   record.identifier,
      title:        record.title,
      description:  record.description,
      status:       record.status,
      project:      record.project,
      parent:       record.parent,
      creator:      record.creator,
      assignee:     record.assignee,
      blocked_by:   record.blocked_by,
      priority:     record.priority,
      created_at:   record.created_at,
      updated_at:   record.updated_at,
    }
  }
}

impl IssueRecord {
  pub fn issue_number(&self) -> i32 {
    self.issue_number
  }

  pub fn identifier(&self) -> &String {
    &self.identifier
  }

  pub fn title(&self) -> &String {
    &self.title
  }

  pub fn description(&self) -> &String {
    &self.description
  }

  pub fn status(&self) -> &IssueStatus {
    &self.status
  }

  pub fn project(&self) -> &Option<SurrealId> {
    &self.project
  }

  pub fn parent(&self) -> &Option<SurrealId> {
    &self.parent
  }

  pub fn creator(&self) -> &Option<SurrealId> {
    &self.creator
  }

  pub fn assignee(&self) -> &Option<SurrealId> {
    &self.assignee
  }

  pub fn blocked_by(&self) -> &Option<SurrealId> {
    &self.blocked_by
  }

  pub fn priority(&self) -> &IssuePriority {
    &self.priority
  }

  pub fn created_at(&self) -> &DateTime<Utc> {
    &self.created_at
  }

  pub fn updated_at(&self) -> &DateTime<Utc> {
    &self.updated_at
  }
}

impl IssueModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    IssueCommentModel::migrate(db).await?;
    IssueActionModel::migrate(db).await?;
    IssueAttachmentModel::migrate(db).await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS comments ON TABLE issues COMPUTED <~issue_comments;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS actions ON TABLE issues COMPUTED <~issue_actions;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS attachments ON TABLE issues COMPUTED <~issue_attachments;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS parent ON TABLE issue TYPE option<record<issue>> REFERENCE ON DELETE UNSET;
      "#,
    )
    .await?;

    db.query(
      r#"
      DEFINE FIELD IF NOT EXISTS children ON TABLE issue COMPUTED <~issue;
      "#,
    )
    .await?;

    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssuePatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status:      Option<IssueStatus>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub project:     Option<Option<SurrealId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent:      Option<Option<SurrealId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub creator:     Option<Option<SurrealId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub assignee:    Option<Option<SurrealId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub blocked_by:  Option<Option<SurrealId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub priority:    Option<IssuePriority>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[specta(type = i32)]
  pub updated_at:  Option<DateTime<Utc>>,
}

pub struct IssueRepository;

impl IssueRepository {
  pub async fn create(model: IssueModel) -> Result<IssueRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUES_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create issue"))?;

    Self::get(record_id.into()).await
  }

  pub async fn add_comment(model: IssueCommentModel) -> Result<IssueCommentRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_COMMENTS_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create issue comment"))?;

    Self::get_comment(record_id.into()).await
  }

  pub async fn add_action(model: IssueActionModel) -> Result<IssueActionRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_ACTIONS_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create issue action"))?;

    Self::get_action(record_id.into()).await
  }

  pub async fn add_attachment(model: IssueAttachmentModel) -> Result<IssueAttachmentRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_ATTACHMENTS_TABLE, Uuid::new_v7());
    let _: Record =
      db.create(record_id.clone()).content(model).await?.ok_or(anyhow::anyhow!("Failed to create issue attachment"))?;

    Self::get_attachment(record_id.into()).await
  }

  pub async fn get(id: SurrealId) -> Result<IssueRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Issue not found"))?;
    Ok(record)
  }

  pub async fn get_comment(id: SurrealId) -> Result<IssueCommentRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueCommentRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Issue comment not found"))?;
    Ok(record)
  }

  pub async fn get_action(id: SurrealId) -> Result<IssueActionRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueActionRecord = db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Issue action not found"))?;
    Ok(record)
  }

  pub async fn get_attachment(id: SurrealId) -> Result<IssueAttachmentRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueAttachmentRecord =
      db.select(id.inner()).await?.ok_or(anyhow::anyhow!("Issue attachment not found"))?;
    Ok(record)
  }

  pub async fn list() -> Result<Vec<IssueRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueRecord> = db.query("SELECT * FROM issues").await?.take(0)?;
    Ok(records)
  }

  pub async fn list_children(issue: SurrealId) -> Result<Vec<IssueRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueRecord> =
      db.query("SELECT * FROM $issue_id.children.*").bind(("issue_id", issue.inner())).await?.take(0)?;
    Ok(records)
  }

  pub async fn list_comments(issue: SurrealId) -> Result<Vec<IssueCommentRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueCommentRecord> =
      db.query("SELECT * FROM $issue_id.comments.*").bind(("issue_id", issue.inner())).await?.take(0)?;
    Ok(records)
  }

  pub async fn list_actions(issue: SurrealId) -> Result<Vec<IssueActionRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueActionRecord> =
      db.query("SELECT * FROM $issue_id.actions.*").bind(("issue_id", issue.inner())).await?.take(0)?;
    Ok(records)
  }

  pub async fn list_attachments(issue: SurrealId) -> Result<Vec<IssueAttachmentRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueAttachmentRecord> =
      db.query("SELECT * FROM $issue_id.attachments.*").bind(("issue_id", issue.inner())).await?.take(0)?;
    Ok(records)
  }

  pub async fn update(id: SurrealId, patch: IssuePatch) -> Result<IssueRecord> {
    let db = SurrealConnection::db().await;

    let mut issue_model: IssueModel = Self::get(id.clone()).await?.into();
    issue_model.updated_at = Utc::now();

    if let Some(title) = patch.title {
      issue_model.title = title;
    }

    if let Some(description) = patch.description {
      issue_model.description = description;
    }

    if let Some(status) = patch.status {
      issue_model.status = status;
    }

    if let Some(project) = patch.project {
      issue_model.project = project;
    }

    if let Some(parent) = patch.parent {
      issue_model.parent = parent;
    }

    if let Some(creator) = patch.creator {
      issue_model.creator = creator;
    }

    if let Some(assignee) = patch.assignee {
      issue_model.assignee = assignee;
    }

    if let Some(blocked_by) = patch.blocked_by {
      issue_model.blocked_by = blocked_by;
    }

    if let Some(priority) = patch.priority {
      issue_model.priority = priority;
    }

    issue_model.updated_at = Utc::now();

    let record: IssueRecord =
      db.update(id.inner()).merge(issue_model).await?.ok_or(anyhow::anyhow!("Failed to update issue"))?;
    Ok(record)
  }

  pub async fn delete(id: SurrealId) -> Result<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db.delete(id.inner()).await?.ok_or(anyhow::anyhow!("Failed to delete issue"))?;
    Ok(())
  }
}
