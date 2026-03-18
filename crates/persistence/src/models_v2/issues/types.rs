use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use common::shared::prelude::DbId;
use common::shared::prelude::SurrealId;
use macros::SurrealEnumValue;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::prelude::EmployeeId;

pub const ISSUES_TABLE: &str = "issues";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueId(pub SurrealId);

impl DbId for IssueId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<Uuid> for IssueId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUES_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
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

#[derive(
  Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue,
)]
pub enum IssuePriority {
  Low = 0,
  Medium = 1,
  High = 2,
  Critical = 3,
}

impl Display for IssuePriority {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IssuePriority::Low => write!(f, "0"),
      IssuePriority::Medium => write!(f, "1"),
      IssuePriority::High => write!(f, "2"),
      IssuePriority::Critical => write!(f, "3"),
    }
  }
}

impl FromStr for IssuePriority {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "0" => Ok(IssuePriority::Low),
      "1" => Ok(IssuePriority::Medium),
      "2" => Ok(IssuePriority::High),
      "3" => Ok(IssuePriority::Critical),
      _ => Ok(IssuePriority::Low),
    }
  }
}

pub const ISSUE_ACTIONS_TABLE: &str = "issue_actions";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueActionId(SurrealId);

impl DbId for IssueActionId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<Uuid> for IssueActionId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_ACTIONS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueActionId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum IssueActionKind {
  Create,
  AddComment,
  AddAttachment,
  CheckOut,
  Release,
  Unassign,
  Assign { employee: EmployeeId },
  StatusChange { from: IssueStatus, to: IssueStatus },
  Update,
}

pub const ISSUE_ATTACHMENTS_TABLE: &str = "issue_attachments";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachmentId(SurrealId);

impl DbId for IssueAttachmentId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<Uuid> for IssueAttachmentId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_ATTACHMENTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueAttachmentId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum IssueAttachmentKind {
  #[default]
  Image,
  File,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueAttachment {
  pub name:            String,
  pub attachment_kind: IssueAttachmentKind,
  pub attachment:      String,
  pub mime_kind:       String,
  pub size:            u64,
}

pub const ISSUE_COMMENTS_TABLE: &str = "issue_comments";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct IssueCommentId(SurrealId);

impl DbId for IssueCommentId {
  fn id(&self) -> SurrealId {
    self.0.clone()
  }
}

impl From<Uuid> for IssueCommentId {
  fn from(uuid: Uuid) -> Self {
    Self(RecordId::new(ISSUE_COMMENTS_TABLE, uuid).into())
  }
}

impl From<RecordId> for IssueCommentId {
  fn from(id: RecordId) -> Self {
    Self(SurrealId::from(id))
  }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListIssuesSortBy {
  #[default]
  Priority,
  CreatedAt,
  UpdatedAt,
  Title,
  Status,
}

impl Display for ListIssuesSortBy {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ListIssuesSortBy::Priority => write!(f, "priority"),
      ListIssuesSortBy::CreatedAt => write!(f, "created_at"),
      ListIssuesSortBy::UpdatedAt => write!(f, "updated_at"),
      ListIssuesSortBy::Title => write!(f, "title"),
      ListIssuesSortBy::Status => write!(f, "status"),
    }
  }
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListIssuesSortOrder {
  Asc,
  #[default]
  Desc,
}

impl Display for ListIssuesSortOrder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ListIssuesSortOrder::Asc => write!(f, "asc"),
      ListIssuesSortOrder::Desc => write!(f, "desc"),
    }
  }
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct ListIssuesParams {
  pub expected_statuses: Option<Vec<IssueStatus>>,
  pub page:              Option<i32>,
  pub page_size:         Option<i32>,
  pub sort_by:           Option<ListIssuesSortBy>,
  pub sort_order:        Option<ListIssuesSortOrder>,
}
