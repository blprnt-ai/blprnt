use chrono::DateTime;
use chrono::Utc;
use events::IssueEventKind;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueActionRecord;
use persistence::prelude::IssueAttachment;
use persistence::prelude::IssueAttachmentRecord;
use persistence::prelude::IssueCommentRecord;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRecord;
use persistence::prelude::IssueStatus;
use persistence::prelude::ProjectRecord;
use persistence::prelude::ProviderRecord;
use persistence::prelude::RunRecord;
use persistence::prelude::RunStatus;
use persistence::prelude::RunSummaryRecord;
use persistence::prelude::RunTrigger;
use persistence::prelude::TurnRecord;
use persistence::prelude::TurnStep;
use shared::agent::Provider;

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct IssueDto {
  pub id:             Uuid,
  pub identifier:     String,
  pub title:          String,
  pub description:    String,
  pub status:         IssueStatus,
  pub project:        Option<Uuid>,
  pub parent_id:      Option<Uuid>,
  pub creator:        Option<Uuid>,
  pub assignee:       Option<Uuid>,
  pub blocked_by:     Option<Uuid>,
  pub checked_out_by: Option<Uuid>,
  pub priority:       IssuePriority,
  pub created_at:     DateTime<Utc>,
  pub updated_at:     DateTime<Utc>,
  pub comments:       Vec<IssueCommentDto>,
  pub attachments:    Vec<IssueAttachmentDto>,
  pub actions:        Vec<IssueActionDto>,
}

impl From<IssueRecord> for IssueDto {
  fn from(record: IssueRecord) -> Self {
    Self {
      id:             record.id.uuid(),
      identifier:     format!("{}-{}", record.identifier, record.issue_number),
      title:          record.title,
      description:    record.description,
      status:         record.status,
      project:        record.project.map(|p| p.uuid()),
      parent_id:      record.parent_id.map(|p| p.uuid()),
      creator:        record.creator.map(|c| c.uuid()),
      assignee:       record.assignee.map(|a| a.uuid()),
      blocked_by:     record.blocked_by.map(|b| b.uuid()),
      checked_out_by: record.checked_out_by.map(|c| c.uuid()),
      priority:       record.priority,
      created_at:     record.created_at,
      updated_at:     record.updated_at,
      comments:       vec![],
      attachments:    vec![],
      actions:        vec![],
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct IssueCommentDto {
  pub id:         Uuid,
  pub comment:    String,
  pub creator:    Uuid,
  pub run_id:     Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueCommentRecord> for IssueCommentDto {
  fn from(record: IssueCommentRecord) -> Self {
    Self {
      id:         record.id.uuid(),
      comment:    record.comment,
      creator:    record.creator.uuid(),
      run_id:     record.run_id.map(|r| r.uuid()),
      created_at: record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct IssueAttachmentDto {
  pub id:         Uuid,
  pub attachment: IssueAttachment,
  pub creator:    Uuid,
  pub run_id:     Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentDto {
  fn from(record: IssueAttachmentRecord) -> Self {
    Self {
      id:         record.id.uuid(),
      attachment: record.attachment,
      creator:    record.creator.uuid(),
      run_id:     record.run_id.map(|r| r.uuid()),
      created_at: record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct IssueActionDto {
  pub id:          Uuid,
  pub action_kind: IssueActionKind,
  pub creator:     Uuid,
  pub run_id:      Option<Uuid>,
  pub created_at:  DateTime<Utc>,
}

impl From<IssueActionRecord> for IssueActionDto {
  fn from(record: IssueActionRecord) -> Self {
    Self {
      id:          record.id.uuid(),
      action_kind: record.action_kind,
      creator:     record.creator.uuid(),
      run_id:      record.run_id.map(|r| r.uuid()),
      created_at:  record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum IssueEventKindDto {
  Created,
  Updated,
  CommentAdded,
  AttachmentAdded,
  Assigned,
  Unassigned,
  CheckedOut,
  Released,
}

impl From<IssueEventKind> for IssueEventKindDto {
  fn from(kind: IssueEventKind) -> Self {
    match kind {
      IssueEventKind::Created => Self::Created,
      IssueEventKind::Updated => Self::Updated,
      IssueEventKind::CommentAdded => Self::CommentAdded,
      IssueEventKind::AttachmentAdded => Self::AttachmentAdded,
      IssueEventKind::Assigned => Self::Assigned,
      IssueEventKind::Unassigned => Self::Unassigned,
      IssueEventKind::CheckedOut => Self::CheckedOut,
      IssueEventKind::Released => Self::Released,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct IssueStreamSnapshotDto {
  pub issues: Vec<IssueDto>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IssueStreamMessageDto {
  Snapshot { snapshot: IssueStreamSnapshotDto },
  Upsert { kind: IssueEventKindDto, issue: IssueDto },
}

#[derive(Debug, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ProjectDto {
  pub id:                  Uuid,
  pub description:         String,
  pub name:                String,
  pub working_directories: Vec<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl From<ProjectRecord> for ProjectDto {
  fn from(record: ProjectRecord) -> Self {
    Self {
      id:                  record.id.uuid(),
      description:         record.description,
      name:                record.name,
      working_directories: record.working_directories,
      created_at:          record.created_at,
      updated_at:          record.updated_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RunDto {
  pub id:           Uuid,
  pub employee_id:  Uuid,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub created_at:   DateTime<Utc>,
  pub turns:        Vec<TurnDto>,
  pub started_at:   Option<DateTime<Utc>>,
  pub completed_at: Option<DateTime<Utc>>,
}

impl From<RunRecord> for RunDto {
  fn from(record: RunRecord) -> Self {
    Self {
      id:           record.id.uuid(),
      employee_id:  record.employee_id.uuid(),
      status:       record.status,
      trigger:      record.trigger,
      turns:        record.turns.into_iter().map(|t| t.into()).collect(),
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RunSummaryDto {
  pub id:           Uuid,
  pub employee_id:  Uuid,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub created_at:   DateTime<Utc>,
  pub started_at:   Option<DateTime<Utc>>,
  pub completed_at: Option<DateTime<Utc>>,
}

impl From<RunSummaryRecord> for RunSummaryDto {
  fn from(record: RunSummaryRecord) -> Self {
    Self {
      id:           record.id.uuid(),
      employee_id:  record.employee_id.uuid(),
      status:       record.status,
      trigger:      record.trigger,
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RunSummaryPageDto {
  pub items:       Vec<RunSummaryDto>,
  pub page:        u32,
  pub per_page:    u32,
  pub total:       u64,
  pub total_pages: u32,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RunStreamSnapshotDto {
  pub recent_runs:         Vec<RunSummaryDto>,
  pub running_runs:        Vec<RunSummaryDto>,
  pub running_run_details: Vec<RunDto>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunStreamMessageDto {
  Snapshot { snapshot: RunStreamSnapshotDto },
  SummaryUpsert { run: RunSummaryDto },
  DetailUpsert { run: RunDto },
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TurnDto {
  pub id:         Uuid,
  pub steps:      Vec<TurnStep>,
  pub run_id:     Uuid,
  pub reasoning_effort: Option<persistence::prelude::ReasoningEffort>,
  pub created_at: DateTime<Utc>,
}

impl From<TurnRecord> for TurnDto {
  fn from(record: TurnRecord) -> Self {
    Self {
      id:         record.id.uuid(),
      steps:      record.steps,
      run_id:     record.run_id.uuid(),
      reasoning_effort: record.reasoning_effort,
      created_at: record.created_at,
    }
  }
}

#[derive(Debug, serde::Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ProviderDto {
  pub id:         Uuid,
  pub provider:   Provider,
  pub base_url:   Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl From<ProviderRecord> for ProviderDto {
  fn from(record: ProviderRecord) -> Self {
    Self {
      id:         record.id.uuid(),
      provider:   record.provider,
      base_url:   record.base_url,
      created_at: record.created_at,
      updated_at: record.updated_at,
    }
  }
}
