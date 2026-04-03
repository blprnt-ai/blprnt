use anyhow::Context;
use chrono::Duration;
use chrono::Utc;
use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::EmployeeRecord;
use persistence::prelude::IssueId;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueRepository;
use persistence::prelude::IssueStatus;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectRepository;
use persistence::prelude::RunId;
use persistence::prelude::RunRepository;
use persistence::prelude::RunStatus;
use persistence::prelude::RunTrigger;
use persistence::prelude::TelegramConfigRepository;
use persistence::prelude::TelegramCorrelationKind;
use persistence::prelude::TelegramIssueWatchRepository;
use persistence::prelude::TelegramLinkCodeModel;
use persistence::prelude::TelegramLinkCodeRepository;
use persistence::prelude::TelegramLinkRepository;
use persistence::prelude::TelegramLinkStatus;
use persistence::prelude::TelegramMessageCorrelationModel;
use persistence::prelude::TelegramMessageCorrelationPatch;
use persistence::prelude::TelegramMessageCorrelationRepository;
use persistence::prelude::TelegramMessageDirection;
use persistence::prelude::TurnStepContent;
use reqwest::Client;
use serde::Deserialize;
use sha2::Digest;
use sha2::Sha256;
use vault::get_stronghold_secret;

use crate::dto::IssueCommentDto;
use crate::dto::IssueDto;
use crate::dto::RunDto;
use crate::routes::errors::ApiError;
use crate::routes::v1::issues::AddCommentPayload;
use crate::routes::v1::issues::CreateIssuePayload;
use crate::routes::v1::issues::add_comment;
use crate::routes::v1::issues::create_issue;
use crate::routes::v1::issues::load_issue_dto;
use crate::routes::v1::runs::AppendRunMessagePayload;
use crate::routes::v1::runs::TriggerRunPayload;
use crate::routes::v1::runs::append_message;
use crate::routes::v1::runs::trigger_run;
use crate::state::RequestAuth;
use crate::state::RequestExtension;

const TELEGRAM_BOT_TOKEN_NAMESPACE: Uuid = Uuid::from_u128(0x6f9c98c8e3cb4e3ca2fe9d34f9c66aa1);
const TELEGRAM_WEBHOOK_SECRET_NAMESPACE: Uuid = Uuid::from_u128(0x53ed113f6d684a3b9af3cc2588904376);

pub fn telegram_bot_token_key(config_id: Uuid) -> Uuid {
  Uuid::new_v5(&TELEGRAM_BOT_TOKEN_NAMESPACE, config_id.as_bytes())
}

pub fn telegram_webhook_secret_key(config_id: Uuid) -> Uuid {
  Uuid::new_v5(&TELEGRAM_WEBHOOK_SECRET_NAMESPACE, config_id.as_bytes())
}

pub fn hash_link_code(code: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(code.as_bytes());
  hex::encode(hasher.finalize())
}

pub async fn create_link_code(
  employee_id: EmployeeId,
) -> anyhow::Result<(String, persistence::prelude::TelegramLinkCodeRecord)> {
  let raw = Uuid::new_v4().simple().to_string()[..8].to_uppercase();
  let record = TelegramLinkCodeRepository::create(TelegramLinkCodeModel {
    employee_id,
    code_hash: hash_link_code(&raw),
    code_last4: raw.chars().rev().take(4).collect::<String>().chars().rev().collect(),
    expires_at: Utc::now() + Duration::minutes(15),
    claimed_at: None,
    claimed_chat_id: None,
    claimed_user_id: None,
    created_at: Utc::now(),
  })
  .await?;

  Ok((raw, record))
}

pub async fn verify_webhook_secret(candidate: Option<&str>) -> anyhow::Result<bool> {
  let Some(config) = TelegramConfigRepository::get_active().await? else {
    return Ok(false);
  };

  let secret = get_stronghold_secret(vault::Vault::Key, telegram_webhook_secret_key(config.id.uuid())).await;
  Ok(matches!((secret.as_deref(), candidate), (Some(expected), Some(actual)) if expected == actual))
}

pub async fn send_message(
  chat_id: i64,
  text: &str,
  reply_to_message_id: Option<i64>,
  kind: TelegramCorrelationKind,
  issue_id: Option<IssueId>,
  run_id: Option<RunId>,
  employee_id: Option<EmployeeId>,
) -> anyhow::Result<TelegramSendMessageResponse> {
  let config = TelegramConfigRepository::get_active().await?.context("telegram config not found")?;
  let token = get_stronghold_secret(vault::Vault::Key, telegram_bot_token_key(config.id.uuid()))
    .await
    .context("telegram bot token missing")?;
  let client = Client::new();
  let base_url =
    std::env::var("BLPRNT_TELEGRAM_API_BASE_URL").unwrap_or_else(|_| "https://api.telegram.org".to_string());
  let url = format!("{}/bot{token}/sendMessage", base_url.trim_end_matches('/'));

  let response = client
    .post(url)
    .json(&serde_json::json!({
      "chat_id": chat_id,
      "text": text,
      "reply_to_message_id": reply_to_message_id,
    }))
    .send()
    .await?
    .error_for_status()?
    .json::<TelegramSendMessageResponse>()
    .await?;

  TelegramMessageCorrelationRepository::create(TelegramMessageCorrelationModel {
    telegram_chat_id: chat_id,
    telegram_message_id: response.result.message_id,
    direction: TelegramMessageDirection::Outbound,
    kind,
    issue_id,
    run_id,
    employee_id,
    text_preview: Some(text.chars().take(120).collect()),
    created_at: Utc::now(),
    updated_at: Utc::now(),
  })
  .await?;

  Ok(response)
}

pub async fn link_from_code(
  code: &str,
  chat_id: i64,
  user_id: i64,
) -> anyhow::Result<Option<persistence::prelude::TelegramLinkRecord>> {
  let code_hash = hash_link_code(code);
  let Some(link_code) = TelegramLinkCodeRepository::find_claimable_by_hash(&code_hash).await? else {
    return Ok(None);
  };

  TelegramLinkCodeRepository::claim(link_code.id.clone(), chat_id, user_id).await?;
  let link = TelegramLinkRepository::upsert_link(link_code.employee_id, user_id, chat_id).await?;
  Ok(Some(link))
}

pub async fn handle_linked_message(
  employee: EmployeeRecord,
  chat_id: i64,
  message_id: i64,
  text: Option<&str>,
  reply_issue_id: Option<IssueId>,
  reply_run_id: Option<RunId>,
) -> anyhow::Result<()> {
  let text = text.unwrap_or("").trim();
  if text.is_empty() {
    return Ok(());
  }

  if let Some(rest) = text.strip_prefix("/issue new ") {
    let (title, description) = parse_issue_create(rest)?;
    let project_id = infer_default_project_id().await?;
    let issue = create_issue(
      axum::Extension(RequestExtension {
        employee:   employee.clone(),
        project_id: project_id.clone(),
        run_id:     None,
        auth:       RequestAuth::Header,
      }),
      axum::Json(CreateIssuePayload {
        title,
        description,
        status: IssueStatus::Todo,
        priority: IssuePriority::Medium,
        project: project_id.map(|id| id.uuid()),
        parent: None,
        assignee: None,
      }),
    )
    .await
    .map_err(api_error_to_anyhow)?
    .0;

    send_message(
      chat_id,
      &format!("Created {} — {}", issue.identifier, issue.title),
      Some(message_id),
      TelegramCorrelationKind::Issue,
      Some(issue.id.into()),
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  if let Some(prompt) = text.strip_prefix("/run ") {
    let prompt = prompt.trim();
    anyhow::ensure!(!prompt.is_empty(), "Use /run <prompt>");
    ensure_owner_linked(&employee)?;

    let employee_id = employee.id.clone();
    let run = start_conversation_run(employee.clone(), prompt.to_string()).await?;
    let summary = format_run_launch_summary(&run);
    let sent = send_message(
      chat_id,
      &summary,
      Some(message_id),
      TelegramCorrelationKind::Run,
      issue_id_from_run_trigger(&run),
      Some(run.id.into()),
      Some(employee_id.clone()),
    )
    .await?;

    annotate_run_thread(chat_id, sent.result.message_id, run.id.into(), employee_id, issue_id_from_run_trigger(&run))
      .await?;
    return Ok(());
  }

  if let Some(identifier) = text.strip_prefix("/issue ") {
    if let Some(issue) = IssueRepository::find_by_display_identifier(identifier.trim()).await? {
      let dto: IssueDto = load_issue_dto(issue.id.clone(), employee.is_owner()).await?;
      send_message(
        chat_id,
        &format_issue_summary(&dto),
        Some(message_id),
        TelegramCorrelationKind::Issue,
        Some(issue.id),
        None,
        Some(employee.id),
      )
      .await?;
    } else {
      send_message(
        chat_id,
        "Issue not found.",
        Some(message_id),
        TelegramCorrelationKind::Unknown,
        None,
        None,
        Some(employee.id),
      )
      .await?;
    }
    return Ok(());
  }

  if let Some(rest) = text.strip_prefix("/comment ") {
    let (identifier, comment_text) = parse_identifier_and_text(rest)?;
    let issue = IssueRepository::find_by_display_identifier(identifier).await?.context("Issue not found")?;
    create_issue_comment(employee.clone(), issue.id.clone(), comment_text.to_string()).await?;
    let display_id = format!("{}-{}", issue.identifier, issue.issue_number);
    send_message(
      chat_id,
      &format!("Comment added to {display_id}."),
      Some(message_id),
      TelegramCorrelationKind::Issue,
      Some(issue.id),
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  if let Some(identifier) = text.strip_prefix("/watch ") {
    let issue = IssueRepository::find_by_display_identifier(identifier.trim()).await?.context("Issue not found")?;
    TelegramIssueWatchRepository::watch(employee.id.clone(), issue.id.clone()).await?;
    let display_id = format!("{}-{}", issue.identifier, issue.issue_number);
    send_message(
      chat_id,
      &format!("Watching {display_id}."),
      Some(message_id),
      TelegramCorrelationKind::Issue,
      Some(issue.id),
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  if let Some(identifier) = text.strip_prefix("/unwatch ") {
    let issue = IssueRepository::find_by_display_identifier(identifier.trim()).await?.context("Issue not found")?;
    let removed = TelegramIssueWatchRepository::unwatch(employee.id.clone(), issue.id.clone()).await?;
    let display_id = format!("{}-{}", issue.identifier, issue.issue_number);
    let response_text =
      if removed { format!("Unwatched {display_id}.") } else { format!("Was not watching {display_id}.") };
    send_message(
      chat_id,
      &response_text,
      Some(message_id),
      TelegramCorrelationKind::Issue,
      Some(issue.id),
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  if text == "/start" {
    send_message(
      chat_id,
      "Linked. Use /issue ISSUE-1, /issue new Title -- details, /comment ISSUE-1 text, /watch ISSUE-1, or /run prompt.",
      Some(message_id),
      TelegramCorrelationKind::Unknown,
      None,
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  if let Some(run_id) = reply_run_id {
    ensure_owner_linked(&employee)?;
    let employee_id = employee.id.clone();
    let run = continue_run(employee.clone(), run_id.clone(), text.to_string()).await?;
    let summary = format_run_continue_summary(&run);
    let sent = send_message(
      chat_id,
      &summary,
      Some(message_id),
      TelegramCorrelationKind::Run,
      issue_id_from_run_trigger(&run),
      Some(run.id.into()),
      Some(employee_id.clone()),
    )
    .await?;

    annotate_run_thread(chat_id, sent.result.message_id, run.id.into(), employee_id, issue_id_from_run_trigger(&run))
      .await?;
    return Ok(());
  }

  if let Some(issue_id) = reply_issue_id {
    create_issue_comment(employee.clone(), issue_id.clone(), text.to_string()).await?;
    send_message(
      chat_id,
      "Comment added.",
      Some(message_id),
      TelegramCorrelationKind::Issue,
      Some(issue_id),
      None,
      Some(employee.id),
    )
    .await?;
    return Ok(());
  }

  send_message(
    chat_id,
    "Unknown command. Use /issue, /comment, /watch, /run, or reply to an issue/run message.",
    Some(message_id),
    TelegramCorrelationKind::Unknown,
    None,
    None,
    Some(employee.id),
  )
  .await?;

  Ok(())
}

pub async fn notify_run_terminal_status(run_id: RunId) -> anyhow::Result<()> {
  let run = RunRepository::get(run_id.clone()).await?;
  let (status_emoji, status_label) = match &run.status {
    RunStatus::Completed => ("✅", "completed"),
    RunStatus::Failed(_) => ("❌", "failed"),
    _ => return Ok(()),
  };

  let summary = compact_run_terminal_summary(&run, status_emoji, status_label);
  let issue_id = issue_id_from_run_trigger_record(&run.trigger);
  let prior_outbound = TelegramMessageCorrelationRepository::list_outbound_for_run(run.id.clone()).await?;

  let mut notified_chats = std::collections::HashSet::new();

  for correlation in prior_outbound {
    let Some(employee_id) = correlation.employee_id.clone() else {
      continue;
    };

    let links = TelegramLinkRepository::list_for_employee(employee_id.clone()).await?;
    for link in links
      .into_iter()
      .filter(|link| link.status == TelegramLinkStatus::Linked && link.notification_preferences.run_notifications)
    {
      if !notified_chats.insert(link.telegram_chat_id) {
        continue;
      }

      let reply_to_message_id = find_latest_run_thread_message_id_for_chat(&run.id, link.telegram_chat_id).await?;
      let _ = send_message(
        link.telegram_chat_id,
        &summary,
        reply_to_message_id,
        TelegramCorrelationKind::Notification,
        issue_id.clone(),
        Some(run.id.clone()),
        Some(employee_id.clone()),
      )
      .await?;
    }
  }

  if let Some(issue_id) = issue_id.clone() {
    let watches = TelegramIssueWatchRepository::list_for_issue(issue_id.clone()).await?;
    for watch in watches {
      let links = TelegramLinkRepository::list_for_employee(watch.employee_id.clone()).await?;
      for link in links.into_iter().filter(|link| {
        link.status == TelegramLinkStatus::Linked
          && link.notification_preferences.run_notifications
          && link.notification_preferences.issue_notifications
      }) {
        if !notified_chats.insert(link.telegram_chat_id) {
          continue;
        }

        let reply_to_message_id = find_latest_run_thread_message_id_for_chat(&run.id, link.telegram_chat_id).await?;
        let _ = send_message(
          link.telegram_chat_id,
          &summary,
          reply_to_message_id,
          TelegramCorrelationKind::Notification,
          Some(issue_id.clone()),
          Some(run.id.clone()),
          Some(watch.employee_id.clone()),
        )
        .await?;
      }
    }
  }

  Ok(())
}

pub async fn correlate_inbound_message(
  chat_id: i64,
  message_id: i64,
  employee_id: Option<EmployeeId>,
  text: Option<String>,
  kind: TelegramCorrelationKind,
  issue_id: Option<IssueId>,
  run_id: Option<RunId>,
) -> anyhow::Result<persistence::prelude::TelegramMessageCorrelationRecord> {
  let record = TelegramMessageCorrelationRepository::create(TelegramMessageCorrelationModel {
    telegram_chat_id: chat_id,
    telegram_message_id: message_id,
    direction: TelegramMessageDirection::Inbound,
    kind,
    issue_id,
    run_id,
    employee_id,
    text_preview: text.map(|value| value.chars().take(120).collect()),
    created_at: Utc::now(),
    updated_at: Utc::now(),
  })
  .await?;
  Ok(record)
}

async fn create_issue_comment(
  employee: EmployeeRecord,
  issue_id: IssueId,
  comment: String,
) -> anyhow::Result<IssueCommentDto> {
  Ok(
    add_comment(
      axum::Extension(RequestExtension { employee, project_id: None, run_id: None, auth: RequestAuth::Header }),
      axum::extract::Path(issue_id.uuid()),
      axum::Json(AddCommentPayload { comment, reopen_issue: None, mentions: vec![] }),
    )
    .await
    .map_err(api_error_to_anyhow)?
    .0,
  )
}

async fn infer_default_project_id() -> anyhow::Result<Option<ProjectId>> {
  let projects = ProjectRepository::list().await?;
  Ok((projects.len() == 1).then(|| projects[0].id.clone()))
}

fn parse_issue_create(input: &str) -> anyhow::Result<(String, String)> {
  let Some((title, description)) = input.split_once(" -- ") else {
    anyhow::bail!("Use /issue new <title> -- <description>");
  };
  let title = title.trim();
  let description = description.trim();
  anyhow::ensure!(!title.is_empty() && !description.is_empty(), "Issue title and description are required");
  Ok((title.to_string(), description.to_string()))
}

fn parse_identifier_and_text(input: &str) -> anyhow::Result<(&str, &str)> {
  let Some((identifier, text)) = input.trim().split_once(' ') else {
    anyhow::bail!("Use /comment <identifier> <text>");
  };
  anyhow::ensure!(
    !identifier.trim().is_empty() && !text.trim().is_empty(),
    "Issue identifier and comment text are required"
  );
  Ok((identifier.trim(), text.trim()))
}

fn format_issue_summary(issue: &IssueDto) -> String {
  let assignee = issue.assignee.map(|id| id.to_string()).unwrap_or_else(|| "unassigned".to_string());
  format!(
    "{} — {}\nstatus: {}\npriority: {}\nassignee: {}",
    issue.identifier, issue.title, issue.status, issue.priority, assignee
  )
}

fn api_error_to_anyhow(error: ApiError) -> anyhow::Error {
  anyhow::anyhow!("{}", error.message)
}

fn ensure_owner_linked(employee: &EmployeeRecord) -> anyhow::Result<()> {
  anyhow::ensure!(employee.is_owner(), "Telegram run access is currently limited to linked owners");
  Ok(())
}

async fn start_conversation_run(employee: EmployeeRecord, prompt: String) -> anyhow::Result<RunDto> {
  let response = trigger_run(
    axum::Extension(RequestExtension {
      employee: employee.clone(),
      project_id: None,
      run_id: None,
      auth: RequestAuth::Header,
    }),
    axum::Json(TriggerRunPayload {
      employee_id:      employee.id.uuid(),
      trigger:          Some(RunTrigger::Conversation),
      prompt:           Some(prompt),
      reasoning_effort: None,
    }),
  )
  .await
  .map_err(api_error_to_anyhow)?;

  Ok(response.0)
}

async fn continue_run(employee: EmployeeRecord, run_id: RunId, prompt: String) -> anyhow::Result<RunDto> {
  let response = append_message(
    axum::extract::Path(run_id.uuid()),
    axum::Extension(RequestExtension { employee, project_id: None, run_id: None, auth: RequestAuth::Header }),
    axum::Json(AppendRunMessagePayload { prompt, reasoning_effort: None }),
  )
  .await
  .map_err(|error| match error.status {
    axum::http::StatusCode::BAD_REQUEST => {
      anyhow::anyhow!(extract_api_detail(&error).unwrap_or_else(|| error.message.clone()))
    }
    axum::http::StatusCode::FORBIDDEN => anyhow::anyhow!("Telegram run access is currently limited to linked owners"),
    _ => api_error_to_anyhow(error),
  })?;

  Ok(response.0)
}

async fn annotate_run_thread(
  chat_id: i64,
  telegram_message_id: i64,
  run_id: RunId,
  employee_id: EmployeeId,
  issue_id: Option<IssueId>,
) -> anyhow::Result<()> {
  let correlation = TelegramMessageCorrelationRepository::find_by_chat_message(chat_id, telegram_message_id).await?;
  let Some(correlation) = correlation else {
    anyhow::bail!("run correlation missing after telegram send");
  };

  TelegramMessageCorrelationRepository::update(
    correlation.id,
    TelegramMessageCorrelationPatch {
      run_id: Some(Some(run_id)),
      issue_id: Some(issue_id),
      employee_id: Some(Some(employee_id)),
      updated_at: Some(Utc::now()),
      ..Default::default()
    },
  )
  .await?;

  Ok(())
}

fn format_run_launch_summary(run: &RunDto) -> String {
  match issue_id_from_run_trigger(run) {
    Some(issue_id) => format!("Started run {} for issue {}.", short_run_id(run.id), issue_id.uuid()),
    None => format!("Started run {}.", short_run_id(run.id)),
  }
}

fn format_run_continue_summary(run: &RunDto) -> String {
  format!("Continued run {}.", short_run_id(run.id))
}

fn compact_run_terminal_summary(run: &persistence::prelude::RunRecord, emoji: &str, label: &str) -> String {
  let context = issue_id_from_run_trigger_record(&run.trigger)
    .map(|issue_id| format!(" issue {}", issue_id.uuid()))
    .unwrap_or_default();
  let tail = latest_run_response_preview(run).unwrap_or_else(|| match &run.status {
    RunStatus::Failed(reason) => truncate_single_line(reason, 120),
    _ => String::new(),
  });

  if tail.is_empty() {
    format!("{emoji} Run {}{context} {label}.", short_run_id(run.id.uuid()))
  } else {
    format!("{emoji} Run {}{context} {label}. {}", short_run_id(run.id.uuid()), tail)
  }
}

fn latest_run_response_preview(run: &persistence::prelude::RunRecord) -> Option<String> {
  for turn in run.turns.iter().rev() {
    for step in turn.steps.iter().rev() {
      for content in step.response.contents.iter().rev() {
        if let TurnStepContent::Text(text) = content
          && !text.text.trim().is_empty()
        {
          return Some(truncate_single_line(&text.text, 120));
        }
      }
    }
  }

  None
}

async fn find_latest_run_thread_message_id_for_chat(run_id: &RunId, chat_id: i64) -> anyhow::Result<Option<i64>> {
  let outbound = TelegramMessageCorrelationRepository::list_outbound_for_run(run_id.clone()).await?;
  Ok(outbound.into_iter().find(|record| record.telegram_chat_id == chat_id).map(|record| record.telegram_message_id))
}

fn issue_id_from_run_trigger(run: &RunDto) -> Option<IssueId> {
  match &run.trigger {
    RunTrigger::IssueAssignment { issue_id } | RunTrigger::IssueMention { issue_id, .. } => Some(issue_id.clone()),
    RunTrigger::Manual | RunTrigger::Conversation | RunTrigger::Timer => None,
  }
}

fn issue_id_from_run_trigger_record(trigger: &RunTrigger) -> Option<IssueId> {
  match trigger {
    RunTrigger::IssueAssignment { issue_id } | RunTrigger::IssueMention { issue_id, .. } => Some(issue_id.clone()),
    RunTrigger::Manual | RunTrigger::Conversation | RunTrigger::Timer => None,
  }
}

fn short_run_id(run_id: Uuid) -> String {
  run_id.to_string().chars().take(8).collect()
}

fn truncate_single_line(text: &str, max_chars: usize) -> String {
  let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
  let mut truncated = normalized.chars().take(max_chars).collect::<String>();
  if normalized.chars().count() > max_chars {
    truncated.push('…');
  }
  truncated
}

fn extract_api_detail(error: &ApiError) -> Option<String> {
  error.details.as_ref().and_then(|value| match value {
    serde_json::Value::String(text) => Some(text.clone()),
    _ => None,
  })
}

#[derive(Debug, Deserialize)]
pub struct TelegramSendMessageResponse {
  pub result: TelegramMessage,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
  pub message_id: i64,
}
