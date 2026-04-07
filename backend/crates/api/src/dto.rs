use chrono::DateTime;
use chrono::Utc;
use events::IssueEventKind;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::IssueActionKind;
use persistence::prelude::IssueActionRecord;
use persistence::prelude::IssueAttachment;
use persistence::prelude::IssueAttachmentKind;
use persistence::prelude::IssueAttachmentRecord;
use persistence::prelude::IssueCommentMention;
use persistence::prelude::IssueCommentRecord;
use persistence::prelude::IssueLabel;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRecord;
use persistence::prelude::IssueStatus;
use persistence::prelude::ProjectRecord;
use persistence::prelude::ProviderRecord;
use persistence::prelude::RunRecord;
use persistence::prelude::RunStatus;
use persistence::prelude::RunSummaryRecord;
use persistence::prelude::RunTrigger;
use persistence::prelude::TelegramConfigRecord;
use persistence::prelude::TelegramCorrelationKind;
use persistence::prelude::TelegramLinkCodeRecord;
use persistence::prelude::TelegramLinkRecord;
use persistence::prelude::TelegramLinkStatus;
use persistence::prelude::TelegramMessageCorrelationRecord;
use persistence::prelude::TelegramMessageDirection;
use persistence::prelude::TelegramNotificationPreferences;
use persistence::prelude::TelegramParseMode;
use persistence::prelude::TurnRecord;
use persistence::prelude::TurnStep;
use persistence::prelude::McpServerRecord;
use persistence::prelude::RunEnabledMcpServerRecord;
use shared::agent::Provider;
use shared::tools::McpServerAuthState;

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueDto {
  pub id:             Uuid,
  pub identifier:     String,
  pub title:          String,
  pub description:    String,
  pub labels:         Vec<IssueLabel>,
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum MyWorkReasonDto {
  Assigned,
  Mentioned,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MyWorkItemDto {
  pub issue_id:         Uuid,
  pub issue_identifier: String,
  pub title:            String,
  pub project_id:       Option<Uuid>,
  pub project_name:     Option<String>,
  pub status:           IssueStatus,
  pub priority:         IssuePriority,
  pub reason:           MyWorkReasonDto,
  pub relevant_at:      DateTime<Utc>,
  pub comment_id:       Option<Uuid>,
  pub comment_snippet:  Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MyWorkResponseDto {
  pub assigned:  Vec<MyWorkItemDto>,
  pub mentioned: Vec<MyWorkItemDto>,
}

impl From<IssueRecord> for IssueDto {
  fn from(record: IssueRecord) -> Self {
    Self {
      id:             record.id.uuid(),
      identifier:     format!("{}-{}", record.identifier, record.issue_number),
      title:          record.title,
      description:    record.description,
      labels:         record.labels.clone().unwrap_or_default(),
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueCommentMentionDto {
  pub employee_id: Uuid,
  pub label:       String,
}

impl From<IssueCommentMention> for IssueCommentMentionDto {
  fn from(record: IssueCommentMention) -> Self {
    Self { employee_id: record.employee_id.uuid(), label: record.label }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueCommentDto {
  pub id:         Uuid,
  pub comment:    String,
  pub mentions:   Vec<IssueCommentMentionDto>,
  pub creator:    Uuid,
  pub run_id:     Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueCommentRecord> for IssueCommentDto {
  fn from(record: IssueCommentRecord) -> Self {
    let mentions = record.mentions.clone().unwrap_or_default().into_iter().map(Into::into).collect();

    Self {
      id:         record.id.uuid(),
      comment:    record.comment,
      mentions:   mentions,
      creator:    record.creator.uuid(),
      run_id:     record.run_id.map(|r| r.uuid()),
      created_at: record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueAttachmentDto {
  pub id:              Uuid,
  pub name:            String,
  pub attachment_kind: IssueAttachmentKind,
  pub mime_kind:       String,
  pub size:            u64,
  pub run_id:          Option<Uuid>,
  pub created_at:      DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentDto {
  fn from(record: IssueAttachmentRecord) -> Self {
    Self {
      id:              record.id.uuid(),
      name:            record.attachment.name,
      attachment_kind: record.attachment.attachment_kind,
      mime_kind:       record.attachment.mime_kind,
      size:            record.attachment.size,
      run_id:          record.run_id.map(|r| r.uuid()),
      created_at:      record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueAttachmentDetailDto {
  pub id:         Uuid,
  pub attachment: IssueAttachment,
  pub creator:    Uuid,
  pub run_id:     Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

impl From<IssueAttachmentRecord> for IssueAttachmentDetailDto {
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IssueStreamSnapshotDto {
  pub issues: Vec<IssueDto>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IssueStreamMessageDto {
  Snapshot { snapshot: IssueStreamSnapshotDto },
  Upsert { kind: IssueEventKindDto, issue: IssueDto },
}

#[derive(Debug, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RunDto {
  pub id:           Uuid,
  pub employee_id:  Uuid,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub enabled_mcp_servers: Vec<RunEnabledMcpServerDto>,
  pub usage:        Option<persistence::prelude::UsageMetrics>,
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
      enabled_mcp_servers: vec![],
      usage:        record.usage,
      turns:        record.turns.into_iter().map(|t| t.into()).collect(),
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RunSummaryDto {
  pub id:           Uuid,
  pub employee_id:  Uuid,
  pub status:       RunStatus,
  pub trigger:      RunTrigger,
  pub enabled_mcp_servers: Vec<RunEnabledMcpServerDto>,
  pub usage:        Option<persistence::prelude::UsageMetrics>,
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
      enabled_mcp_servers: vec![],
      usage:        record.usage,
      created_at:   record.created_at,
      started_at:   record.started_at,
      completed_at: record.completed_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RunSummaryPageDto {
  pub items:       Vec<RunSummaryDto>,
  pub page:        u32,
  pub per_page:    u32,
  pub total:       u64,
  pub total_pages: u32,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RunStreamSnapshotDto {
  pub recent_runs:         Vec<RunSummaryDto>,
  pub running_runs:        Vec<RunSummaryDto>,
  pub running_run_details: Vec<RunDto>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunStreamMessageDto {
  Snapshot { snapshot: RunStreamSnapshotDto },
  SummaryUpsert { run: RunSummaryDto },
  DetailUpsert { run: RunDto },
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TurnDto {
  pub id:               Uuid,
  pub steps:            Vec<TurnStep>,
  pub run_id:           Uuid,
  pub reasoning_effort: Option<persistence::prelude::ReasoningEffort>,
  pub usage:            persistence::prelude::UsageMetrics,
  pub created_at:       DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct McpServerDto {
  pub id:           Uuid,
  pub project_id:   Uuid,
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

impl From<McpServerRecord> for McpServerDto {
  fn from(record: McpServerRecord) -> Self {
    Self {
      id: record.id.uuid(),
      project_id: record.project_id.uuid(),
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RunEnabledMcpServerDto {
  pub id:         Uuid,
  pub run_id:     Uuid,
  pub server_id:  Uuid,
  pub enabled_at: DateTime<Utc>,
}

impl From<RunEnabledMcpServerRecord> for RunEnabledMcpServerDto {
  fn from(record: RunEnabledMcpServerRecord) -> Self {
    Self {
      id: record.id.uuid(),
      run_id: record.run_id.uuid(),
      server_id: record.server_id.uuid(),
      enabled_at: record.enabled_at,
    }
  }
}

impl From<TurnRecord> for TurnDto {
  fn from(record: TurnRecord) -> Self {
    Self {
      id:               record.id.uuid(),
      steps:            record.steps,
      run_id:           record.run_id.uuid(),
      reasoning_effort: record.reasoning_effort,
      usage:            record.usage,
      created_at:       record.created_at,
    }
  }
}

#[derive(Debug, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
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

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TelegramConfigDto {
  pub id:           Uuid,
  pub bot_username: Option<String>,
  pub parse_mode:   Option<TelegramParseMode>,
  pub enabled:      bool,
  pub created_at:   DateTime<Utc>,
  pub updated_at:   DateTime<Utc>,
}

impl From<TelegramConfigRecord> for TelegramConfigDto {
  fn from(record: TelegramConfigRecord) -> Self {
    Self {
      id:           record.id.uuid(),
      bot_username: record.bot_username,
      parse_mode:   record.parse_mode,
      enabled:      record.enabled,
      created_at:   record.created_at,
      updated_at:   record.updated_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TelegramLinkDto {
  pub id:                       Uuid,
  pub employee_id:              Uuid,
  pub telegram_user_id:         i64,
  pub telegram_chat_id:         i64,
  pub status:                   TelegramLinkStatus,
  pub notification_preferences: TelegramNotificationPreferences,
  pub created_at:               DateTime<Utc>,
  pub updated_at:               DateTime<Utc>,
  pub last_seen_at:             Option<DateTime<Utc>>,
}

impl From<TelegramLinkRecord> for TelegramLinkDto {
  fn from(record: TelegramLinkRecord) -> Self {
    Self {
      id:                       record.id.uuid(),
      employee_id:              record.employee_id.uuid(),
      telegram_user_id:         record.telegram_user_id,
      telegram_chat_id:         record.telegram_chat_id,
      status:                   record.status,
      notification_preferences: record.notification_preferences,
      created_at:               record.created_at,
      updated_at:               record.updated_at,
      last_seen_at:             record.last_seen_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TelegramLinkCodeDto {
  pub id:              Uuid,
  pub employee_id:     Uuid,
  pub code_last4:      String,
  pub expires_at:      DateTime<Utc>,
  pub claimed_at:      Option<DateTime<Utc>>,
  pub claimed_chat_id: Option<i64>,
  pub claimed_user_id: Option<i64>,
  pub created_at:      DateTime<Utc>,
}

impl From<TelegramLinkCodeRecord> for TelegramLinkCodeDto {
  fn from(record: TelegramLinkCodeRecord) -> Self {
    Self {
      id:              record.id.uuid(),
      employee_id:     record.employee_id.uuid(),
      code_last4:      record.code_last4,
      expires_at:      record.expires_at,
      claimed_at:      record.claimed_at,
      claimed_chat_id: record.claimed_chat_id,
      claimed_user_id: record.claimed_user_id,
      created_at:      record.created_at,
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TelegramMessageCorrelationDto {
  pub id:                  Uuid,
  pub telegram_chat_id:    i64,
  pub telegram_message_id: i64,
  pub direction:           TelegramMessageDirection,
  pub kind:                TelegramCorrelationKind,
  pub issue_id:            Option<Uuid>,
  pub run_id:              Option<Uuid>,
  pub employee_id:         Option<Uuid>,
  pub text_preview:        Option<String>,
  pub created_at:          DateTime<Utc>,
  pub updated_at:          DateTime<Utc>,
}

impl From<TelegramMessageCorrelationRecord> for TelegramMessageCorrelationDto {
  fn from(record: TelegramMessageCorrelationRecord) -> Self {
    Self {
      id:                  record.id.uuid(),
      telegram_chat_id:    record.telegram_chat_id,
      telegram_message_id: record.telegram_message_id,
      direction:           record.direction,
      kind:                record.kind,
      issue_id:            record.issue_id.map(|id| id.uuid()),
      run_id:              record.run_id.map(|id| id.uuid()),
      employee_id:         record.employee_id.map(|id| id.uuid()),
      text_preview:        record.text_preview,
      created_at:          record.created_at,
      updated_at:          record.updated_at,
    }
  }
}
