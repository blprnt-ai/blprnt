mod types;
use shared::errors::DatabaseConflict;
use shared::errors::DatabaseEntity;
use shared::errors::DatabaseOperation;
use surrealdb_types::Value;
pub use types::*;

mod issue_actions;
mod issue_attachments;
mod issue_comments;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
pub use issue_actions::*;
pub use issue_attachments::*;
pub use issue_comments::*;
use shared::errors::DatabaseError;
use shared::errors::DatabaseResult;
use surrealdb_types::RecordId;
use surrealdb_types::SurrealValue;
use surrealdb_types::Uuid;

use crate::connection::DbConnection;
use crate::connection::SurrealConnection;
use crate::prelude::DbId;
use crate::prelude::EmployeeId;
use crate::prelude::ProjectId;
use crate::prelude::Record;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueModel {
  pub issue_number:   i32,
  pub identifier:     String,
  pub title:          String,
  pub description:    String,
  pub labels:         Option<Vec<IssueLabel>>,
  pub status:         IssueStatus,
  pub project:        Option<ProjectId>,
  pub parent_id:      Option<IssueId>,
  pub creator:        Option<EmployeeId>,
  pub assignee:       Option<EmployeeId>,
  pub blocked_by:     Option<IssueId>,
  pub checked_out_by: Option<EmployeeId>,
  pub priority:       IssuePriority,
  pub created_at:     DateTime<Utc>,
  pub updated_at:     DateTime<Utc>,
}

impl Default for IssueModel {
  fn default() -> Self {
    Self {
      issue_number:   0,
      identifier:     String::from("ISSUE"),
      title:          String::new(),
      description:    String::new(),
      labels:         None,
      status:         IssueStatus::Backlog,
      project:        None,
      parent_id:      None,
      creator:        None,
      assignee:       None,
      blocked_by:     None,
      checked_out_by: None,
      priority:       IssuePriority::Medium,
      created_at:     Utc::now(),
      updated_at:     Utc::now(),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct IssueRecord {
  pub id:             IssueId,
  pub issue_number:   i32,
  pub identifier:     String,
  pub title:          String,
  pub description:    String,
  pub labels:         Option<Vec<IssueLabel>>,
  pub status:         IssueStatus,
  pub project:        Option<ProjectId>,
  pub parent_id:      Option<IssueId>,
  pub creator:        Option<EmployeeId>,
  pub assignee:       Option<EmployeeId>,
  pub blocked_by:     Option<IssueId>,
  pub checked_out_by: Option<EmployeeId>,
  pub priority:       IssuePriority,
  pub created_at:     DateTime<Utc>,
  pub updated_at:     DateTime<Utc>,
}

impl From<IssueRecord> for IssueModel {
  fn from(record: IssueRecord) -> Self {
    Self {
      issue_number:   record.issue_number,
      identifier:     record.identifier,
      title:          record.title,
      description:    record.description,
      labels:         record.labels,
      status:         record.status,
      project:        record.project,
      parent_id:      record.parent_id,
      creator:        record.creator,
      assignee:       record.assignee,
      blocked_by:     record.blocked_by,
      checked_out_by: record.checked_out_by,
      priority:       record.priority,
      created_at:     record.created_at,
      updated_at:     record.updated_at,
    }
  }
}

impl IssueModel {
  pub async fn migrate(db: &DbConnection) -> Result<()> {
    IssueCommentModel::migrate(db).await?;
    IssueActionModel::migrate(db).await?;
    IssueAttachmentModel::migrate(db).await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS comments ON TABLE {ISSUES_TABLE} COMPUTED <~{ISSUE_COMMENTS_TABLE};"))
      .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS actions ON TABLE {ISSUES_TABLE} COMPUTED <~{ISSUE_ACTIONS_TABLE};"))
      .await?;

    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS attachments ON TABLE {ISSUES_TABLE} COMPUTED <~{ISSUE_ATTACHMENTS_TABLE};"
    ))
    .await?;

    db.query(format!(
      "DEFINE FIELD IF NOT EXISTS parent_id ON TABLE {ISSUES_TABLE} TYPE option<record<{ISSUES_TABLE}>> REFERENCE ON DELETE UNSET;"
    ))
    .await?;

    db.query(format!("DEFINE FIELD IF NOT EXISTS children ON TABLE {ISSUES_TABLE} COMPUTED <~{ISSUES_TABLE};")).await?;

    Ok(())
  }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS)]
#[ts(export)]
pub struct IssuePatch {
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub title:       Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(as = "Option<ProjectId>", optional = nullable)]
  pub labels:      Option<Option<Vec<IssueLabel>>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub status:      Option<IssueStatus>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(as = "Option<ProjectId>", optional = nullable)]
  pub project:     Option<Option<ProjectId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(as = "Option<EmployeeId>", optional = nullable)]
  pub assignee:    Option<Option<EmployeeId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(as = "Option<IssueId>", optional = nullable)]
  pub blocked_by:  Option<Option<IssueId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub priority:    Option<IssuePriority>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub updated_at:  Option<DateTime<Utc>>,
}

pub struct IssueRepository;

impl IssueRepository {
  pub async fn create(mut model: IssueModel) -> DatabaseResult<IssueRecord> {
    let db = SurrealConnection::db().await;
    let issue_number: i32 = db
      .query(format!("math::max(SELECT VALUE issue_number FROM {ISSUES_TABLE})"))
      .await
      .ok()
      .map(|mut r| r.take(0).ok())
      .flatten()
      .unwrap_or(Value::Number(0.into()))
      .as_i32()
      .unwrap_or(0);

    model.issue_number = issue_number + 1;

    let record_id = RecordId::new(ISSUES_TABLE, Uuid::new_v7());
    let record: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::Issue })?;

    Self::get(record.id.into()).await
  }

  pub async fn assign(id: IssueId, employee: EmployeeId) -> DatabaseResult<IssueRecord> {
    let patch = IssuePatch { assignee: Some(Some(employee)), ..Default::default() };

    Self::update(id, patch).await
  }

  pub async fn unassign(id: IssueId) -> DatabaseResult<IssueRecord> {
    let patch = IssuePatch { assignee: Some(None), ..Default::default() };

    Self::update(id, patch).await
  }

  pub async fn checkout(id: IssueId, employee: EmployeeId) -> DatabaseResult<IssueRecord> {
    let db = SurrealConnection::db().await;

    let mut issue_record = Self::get(id.clone()).await?;

    if issue_record.checked_out_by.is_some() && issue_record.checked_out_by.unwrap() != employee {
      return Err(DatabaseError::Conflict {
        entity: DatabaseEntity::Issue,
        reason: DatabaseConflict::AlreadyCheckedOut,
      });
    }

    issue_record.checked_out_by = Some(employee.into());
    let issue_model: IssueModel = issue_record.into();

    let _: Record = db
      .update(id.clone().inner())
      .merge(issue_model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;

    Self::get(id).await
  }

  pub async fn release(id: IssueId, employee: EmployeeId) -> DatabaseResult<IssueRecord> {
    let db = SurrealConnection::db().await;

    let mut issue_model = Self::get(id.clone()).await?;

    if issue_model.checked_out_by.is_some() && issue_model.checked_out_by.unwrap() != employee {
      return Err(DatabaseError::Conflict {
        entity: DatabaseEntity::Issue,
        reason: DatabaseConflict::AlreadyCheckedOut,
      });
    }

    issue_model.checked_out_by = None;

    let _: Record = db
      .update(id.clone().inner())
      .merge(issue_model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;

    Self::get(id).await
  }

  pub async fn add_comment(model: IssueCommentModel) -> DatabaseResult<IssueCommentRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_COMMENTS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueComment,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::IssueComment })?;

    Self::get_comment(record_id.into()).await
  }

  pub async fn add_action(model: IssueActionModel) -> DatabaseResult<IssueActionRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_ACTIONS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAction,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::IssueAction })?;

    Self::get_action(record_id.into()).await
  }

  pub async fn add_attachment(model: IssueAttachmentModel) -> DatabaseResult<IssueAttachmentRecord> {
    let db = SurrealConnection::db().await;
    let record_id = RecordId::new(ISSUE_ATTACHMENTS_TABLE, Uuid::new_v7());
    let _: Record = db
      .create(record_id.clone())
      .content(model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAttachment,
        operation: DatabaseOperation::Create,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFoundAfterCreate { entity: DatabaseEntity::IssueAttachment })?;

    Self::get_attachment(record_id.into()).await
  }

  pub async fn get(id: IssueId) -> DatabaseResult<IssueRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;
    Ok(record)
  }

  pub async fn find_by_display_identifier(identifier: &str) -> DatabaseResult<Option<IssueRecord>> {
    let Some((prefix, number)) = identifier.trim().rsplit_once('-') else {
      return Ok(None);
    };
    let Ok(issue_number) = number.parse::<i32>() else {
      return Ok(None);
    };

    let db = SurrealConnection::db().await;
    db.query(format!(
      "SELECT * FROM {ISSUES_TABLE} WHERE identifier = $identifier AND issue_number = $issue_number LIMIT 1"
    ))
    .bind(("identifier", prefix.to_string()))
    .bind(("issue_number", issue_number))
    .await
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Issue,
      operation: DatabaseOperation::Get,
      source:    e.into(),
    })?
    .take(0)
    .map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Issue,
      operation: DatabaseOperation::Get,
      source:    e.into(),
    })
  }

  pub async fn get_comment(id: IssueCommentId) -> DatabaseResult<IssueCommentRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueCommentRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueComment,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::IssueComment })?;
    Ok(record)
  }

  pub async fn get_action(id: IssueActionId) -> DatabaseResult<IssueActionRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueActionRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAction,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::IssueAction })?;
    Ok(record)
  }

  pub async fn get_attachment(id: IssueAttachmentId) -> DatabaseResult<IssueAttachmentRecord> {
    let db = SurrealConnection::db().await;
    let record: IssueAttachmentRecord = db
      .select(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAttachment,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::IssueAttachment })?;
    Ok(record)
  }

  pub async fn list(params: ListIssuesParams) -> DatabaseResult<Vec<IssueRecord>> {
    let db = SurrealConnection::db().await;

    let mut query = format!("SELECT * FROM {ISSUES_TABLE}");
    let mut clauses = Vec::new();

    if let Some(expected_statuses) = &params.expected_statuses {
      clauses.push(format!(
        "status IN [{}]",
        expected_statuses.iter().map(|s| format!("'{}'", s)).collect::<Vec<String>>().join(", ")
      ));
    }

    if params.assignee.is_some() {
      clauses.push("assignee = $assignee".to_string());
    }

    if !clauses.is_empty() {
      query.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
    }

    if let Some(page) = params.page {
      query.push_str(&format!(" LIMIT {}", page * params.page_size.unwrap_or(10)));
    }

    let sort_by_key = params.sort_by.unwrap_or(ListIssuesSortBy::Priority);
    let sort_by_order = params.sort_order.unwrap_or(ListIssuesSortOrder::Desc);

    let sort_by_key = if matches!(sort_by_key, ListIssuesSortBy::Priority) {
      "priority NUMERIC".to_string()
    } else {
      sort_by_key.to_string()
    };

    query.push_str(&format!(" ORDER BY {} {}", sort_by_key, sort_by_order.to_string().to_ascii_uppercase()));

    let mut query = db.query(query);
    if let Some(assignee) = params.assignee {
      query = query.bind(("assignee", EmployeeId::from(assignee).inner()));
    }

    let mut records: Vec<IssueRecord> = query
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;

    if let Some(label) = params.label {
      let normalized = label.trim().to_ascii_lowercase();
      records.retain(|issue| {
        issue
          .labels
          .clone()
          .unwrap_or_default()
          .iter()
          .any(|candidate| candidate.name.trim().to_ascii_lowercase() == normalized)
      });
    }

    Ok(records)
  }

  pub async fn list_children(issue: IssueId) -> DatabaseResult<Vec<IssueRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueRecord> = db
      .query("SELECT * FROM $issue_id.children.*")
      .bind(("issue_id", issue.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;
    Ok(records)
  }

  pub async fn list_comments(issue: IssueId) -> DatabaseResult<Vec<IssueCommentRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueCommentRecord> = db
      .query("SELECT * FROM $issue_id.comments.*")
      .bind(("issue_id", issue.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueComment,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueComment,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;
    Ok(records)
  }

  pub async fn list_actions(issue: IssueId) -> DatabaseResult<Vec<IssueActionRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueActionRecord> = db
      .query("SELECT * FROM $issue_id.actions.*")
      .bind(("issue_id", issue.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAction,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAction,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;
    Ok(records)
  }

  pub async fn list_attachments(issue: IssueId) -> DatabaseResult<Vec<IssueAttachmentRecord>> {
    let db = SurrealConnection::db().await;
    let records: Vec<IssueAttachmentRecord> = db
      .query("SELECT * FROM $issue_id.attachments.*")
      .bind(("issue_id", issue.inner()))
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAttachment,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?
      .take(0)
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::IssueAttachment,
        operation: DatabaseOperation::List,
        source:    e.into(),
      })?;
    Ok(records)
  }

  pub async fn update(id: IssueId, patch: IssuePatch) -> DatabaseResult<IssueRecord> {
    let db = SurrealConnection::db().await;
    let txn = db.clone().begin().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Issue,
      operation: DatabaseOperation::BeginTransaction,
      source:    e.into(),
    })?;
    let mut issue_model: IssueRecord = txn
      .select(id.clone().inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Get,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;

    if let Some(title) = patch.title {
      issue_model.title = title;
    }

    if let Some(description) = patch.description {
      issue_model.description = description;
    }

    if let Some(labels) = patch.labels {
      issue_model.labels = labels;
    }

    if let Some(status) = patch.status {
      issue_model.status = status;
    }

    if let Some(project) = patch.project {
      issue_model.project = project;
    }

    if let Some(assignee) = patch.assignee {
      if issue_model.assignee != assignee {
        issue_model.checked_out_by = None;
      }
      issue_model.assignee = assignee;
    }

    if let Some(blocked_by) = patch.blocked_by {
      issue_model.blocked_by = blocked_by;
    }

    if let Some(priority) = patch.priority {
      issue_model.priority = priority;
    }

    issue_model.updated_at = Utc::now();

    let _: Record = txn
      .update(id.clone().inner())
      .merge(issue_model)
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Update,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;

    txn.commit().await.map_err(|e| DatabaseError::Operation {
      entity:    DatabaseEntity::Issue,
      operation: DatabaseOperation::CommitTransaction,
      source:    e.into(),
    })?;

    Self::get(id).await
  }

  pub async fn delete(id: IssueId) -> DatabaseResult<()> {
    let db = SurrealConnection::db().await;
    let _: Record = db
      .delete(id.inner())
      .await
      .map_err(|e| DatabaseError::Operation {
        entity:    DatabaseEntity::Issue,
        operation: DatabaseOperation::Delete,
        source:    e.into(),
      })?
      .ok_or(DatabaseError::NotFound { entity: DatabaseEntity::Issue })?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;
  use std::sync::Mutex;

  use ts_rs::TS;

  use super::IssueModel;
  use super::IssuePatch;
  use super::IssuePriority;
  use super::IssueRepository;
  use super::IssueStatus;
  use crate::prelude::ProjectModel;
  use crate::prelude::ProjectRepository;

  static TEST_LOCK: Mutex<()> = Mutex::new(());
  static TEST_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread().enable_all().build().expect("failed to create test runtime")
  });

  #[test]
  fn issue_patch_binding_matches_sparse_patch_shape() {
    let binding = IssuePatch::decl(&ts_rs::Config::default());

    assert!(binding.contains("title?: string"), "{binding}");
    assert!(binding.contains("description?: string"), "{binding}");
    assert!(binding.contains("labels?: string | null"), "{binding}");
    assert!(binding.contains("status?: IssueStatus"), "{binding}");
    assert!(binding.contains("project?: string | null"), "{binding}");
    assert!(binding.contains("assignee?: string | null"), "{binding}");
    assert!(binding.contains("blocked_by?: string | null"), "{binding}");
    assert!(binding.contains("priority?: IssuePriority"), "{binding}");
    assert!(binding.contains("updated_at?: string"), "{binding}");
  }

  #[test]
  fn issue_repository_update_clears_nullable_project() {
    let _lock = TEST_LOCK.lock().unwrap();

    TEST_RUNTIME.block_on(async {
      let project = ProjectRepository::create(ProjectModel::new("Runtime Project".to_string(), String::new(), vec![]))
        .await
        .unwrap();
      let issue = IssueRepository::create(IssueModel {
        title: "Controller lifecycle".to_string(),
        description: "Needs nullable project regression coverage.".to_string(),
        status: IssueStatus::Todo,
        project: Some(project.id.clone()),
        priority: IssuePriority::Medium,
        ..Default::default()
      })
      .await
      .unwrap();

      let updated = IssueRepository::update(issue.id.clone(), IssuePatch { project: Some(None), ..Default::default() })
        .await
        .unwrap();

      assert!(updated.project.is_none());
      assert!(IssueRepository::get(issue.id).await.unwrap().project.is_none());
    });
  }
}
