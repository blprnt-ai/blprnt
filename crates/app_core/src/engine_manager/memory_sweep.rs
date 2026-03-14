use std::time::Duration;

use chrono::Local;
use chrono::NaiveDate;
use common::memory::ManagedMemoryStore;
use common::memory::MemoryPathInfo;
use common::memory::MemorySummaryContract;
use common::memory::QmdMemorySearchService;
use common::memory::local_today;
use common::paths::BlprntPath;
use common::shared::prelude::MessageContent;
use common::shared::prelude::SurrealId;
use engine_v2::prelude::same_provider_one_off_text_for_session;
use persistence::prelude::MessagePatchV2;
use persistence::prelude::MessageRecord;
use persistence::prelude::MessageRepositoryV2;
use persistence::prelude::SessionRepositoryV2;
use serde::Serialize;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

const MEMORY_SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 30);
const MEMORY_SWEEP_EXTRACTION_SYSTEM: &str = concat!(
  "You are extracting high-signal durable memory from one blprnt conversation window for long-term reuse.\n",
  "\n",
  "If the conversation only contains exploration, tool usage, or procedural steps without a durable conclusion, return an empty string.",
  "\n",
  "This is not a transcript, activity log, tool trace, or diary. The goal is signal, not coverage.\n",
  "Write plain markdown only, suitable for direct append to a daily memory log.\n",
  "Prefer a small number of substantive sections under clear headings when there is enough material.\n",
  "\n",
  "Before writing, apply this test silently: would a future reader make a worse decision, misunderstand the project, or lose an important lesson if this were omitted? If not, omit it.\n",
  "\n",
  "Capture only material with lasting value, such as:\n",
  "- durable facts about the user, project, architecture, workflow, or constraints\n",
  "- decisions and why they were made\n",
  "- broad work completed and what changed\n",
  "- important bugs, root causes, and fixes\n",
  "- failed or rejected approaches only when they taught a durable lesson or explain the chosen direction\n",
  "- meaningful validation tied to a result\n",
  "- unresolved issues or follow-ups that future work should remember\n",
  "\n",
  "Episodic memory is allowed, but only at the broad-picture level. Summarize the meaningful arc of the work, not the interaction-by-interaction trace.\n",
  "Good episodic memory: completed a migration, discovered a root cause, established a new constraint, validated a refactor, or changed direction for a clear reason.\n",
  "Bad episodic memory: narrating every command, prompt, retry, click, or micro-step.\n",
  "\n",
  "Do not include fine-grained tool or agent mechanics unless they are themselves the durable lesson. Usually omit details such as:\n",
  "- opening the terminal or editor\n",
  "- searching, scrolling, inspecting files, or reading output\n",
  "- asking for confirmation or following routine prompts\n",
  "- sending input like \"y\" to an installer\n",
  "- polling a session or retrying a command\n",
  "- reporting that a command ran when the only outcome was that it ran\n",
  "- plan metadata or todo bookkeeping\n",
  "\n",
  "Preserve chronology only when it clarifies a meaningful arc such as attempt -> discovery -> fix -> consequence. Collapse minor steps into one sentence or omit them entirely.\n",
  "Prefer synthesis over exhaustiveness. A good memory helps a future reader recover context quickly; a bad memory merely replays the conversation.\n",
  "\n",
  "Include concrete names when they carry durable value: files, crates, tools, commands, features, bugs, errors, APIs, and user preferences. Mention them only when they help preserve a real fact, decision, fix, or verification.\n",
  "If you mention validation, tie it to what was validated; do not emit raw test-running diaries.\n",
  "\n",
  "Useful rule: if removing a detail would not change future understanding of the project, user, or problem, omit it.\n",
  "\n",
  "Good: \"I completed the memory-system migration from the old database-backed setup to markdown daily logs, removed the outdated update plumbing, and validated the new search/write flow with targeted tests.\"\n",
  "Bad: \"I opened the terminal, ran a command, answered a prompt, and kept polling the session.\"\n",
  "\n",
  "Do not invent details. Do not speculate. Do not summarize beyond what the messages support.\n",
  "Do not return JSON, metadata blocks, XML, YAML frontmatter, or code fences.\n",
  "If there is no durable fact, decision, lesson, or broad outcome worth preserving, return an empty string.\n",
  "\n",
  "Use 1st person pronouns when describing the agent's actions. For example: \"I completed a system refactor\" instead of \"The agent completed a system refactor\". Using \"we\" is also acceptable.\n",
);

#[derive(Clone)]
pub(super) struct MemorySweepCoordinator {
  project_id: String,
}

#[derive(Clone, Debug)]
struct DirtySessionWindow {
  session_id: SurrealId,
  messages:   Vec<MessageRecord>,
}

#[derive(Clone, Debug)]
struct DirtyDayWindow {
  session_id: SurrealId,
  date:       NaiveDate,
  messages:   Vec<MessageRecord>,
}

#[derive(Debug, Serialize)]
struct SweepPromptMessage {
  id:         String,
  created_at: String,
  role:       Value,
  visibility: Value,
  content:    String,
}

impl MemorySweepCoordinator {
  pub(super) fn new(project_id: String) -> Self {
    Self { project_id }
  }

  pub(super) async fn ensure_today_dir(&self) -> anyhow::Result<()> {
    let today = local_today();
    let created_today =
      ManagedMemoryStore::new(BlprntPath::memories_root().join(&self.project_id)).ensure_dir_for_date(today)?;

    if created_today {
      tracing::info!("Created today's memory directory");
      self.refresh_rolling_summary().await?;
    }

    Ok(())
  }

  pub(super) async fn run_boot_catch_up(&self) {
    if let Err(error) = self.run_once("boot_catch_up").await {
      tracing::warn!("Memory boot catch-up failed: {error}");
    } else {
      tracing::info!("Memory boot catch-up completed successfully");
    }
  }

  pub(super) async fn run_periodic(&self, cancel_token: CancellationToken) {
    let mut interval = tokio::time::interval(MEMORY_SWEEP_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval.tick().await;

    loop {
      tokio::select! {
        _ = cancel_token.cancelled() => break,
        _ = interval.tick() => {
          if let Err(error) = self.run_once("periodic_sweep").await {
            tracing::warn!("Memory periodic sweep failed: {error}");
          }
        }
      }
    }
  }

  async fn run_once(&self, trigger: &str) -> anyhow::Result<()> {
    let _ = self.ensure_today_dir().await;

    let dirty_session_ids = MessageRepositoryV2::list_dirty_session_ids().await?;

    let mut wrote_memories = false;
    for session_id in dirty_session_ids {
      let Some(window) = self.discover_dirty_window(session_id.clone()).await? else {
        tracing::warn!("No dirty window found for session {}", session_id);
        continue;
      };

      let outcome = self.process_window(trigger, window).await?;
      if outcome.wrote_memories {
        wrote_memories = true;
      }
    }

    if wrote_memories {
      QmdMemorySearchService::new(self.project_id.clone()).refresh().await?;
    }

    Ok(())
  }

  async fn discover_dirty_window(&self, session_id: SurrealId) -> anyhow::Result<Option<DirtySessionWindow>> {
    let messages = MessageRepositoryV2::list_dirty_for_session(session_id.clone()).await?;
    if messages.is_empty() {
      return Ok(None);
    }

    Ok(Some(DirtySessionWindow { session_id, messages }))
  }

  async fn process_window(&self, trigger: &str, window: DirtySessionWindow) -> anyhow::Result<WindowProcessingOutcome> {
    let session_model = SessionRepositoryV2::get(window.session_id.clone()).await?;
    let mut outcome = WindowProcessingOutcome::default();
    for day_window in split_window_by_local_day(window) {
      let session_id = day_window.session_id.clone();
      let day_outcome = self.process_day_window(trigger, &session_model, day_window).await?;
      let _ = MessageRepositoryV2::mark_all_clean_for_session(session_id).await;
      outcome.merge(day_outcome);
    }

    Ok(outcome)
  }

  async fn process_day_window(
    &self,
    trigger: &str,
    session_model: &persistence::prelude::SessionRecord,
    day_window: DirtyDayWindow,
  ) -> anyhow::Result<WindowProcessingOutcome> {
    let prompt = build_extraction_prompt(trigger, &day_window);

    let Some(prompt) = prompt else {
      return Ok(WindowProcessingOutcome::default());
    };

    let Some(response_text) = same_provider_one_off_text_for_session(
      session_model,
      prompt,
      MEMORY_SWEEP_EXTRACTION_SYSTEM.to_string(),
      CancellationToken::new(),
    )
    .await?
    else {
      return Ok(WindowProcessingOutcome::default());
    };

    let markdown = normalize_markdown_response(&response_text);
    if markdown.is_empty() {
      return Ok(WindowProcessingOutcome::default());
    }

    ManagedMemoryStore::new(BlprntPath::memories_root().join(session_model.project.key().to_string()))
      .append_entry_for_date(day_window.date, &markdown)?;

    for message in &day_window.messages {
      MessageRepositoryV2::update(
        message.id.clone(),
        MessagePatchV2 { memory_dirty: Some(false), ..Default::default() },
      )
      .await?;
    }

    tracing::info!(
      trigger,
      session_id = %day_window.session_id,
      date = %day_window.date,
      dirty_message_count = day_window.messages.len(),
      extracted_char_count = markdown.len(),
      "Memory sweep processed dirty day window"
    );

    Ok(WindowProcessingOutcome { wrote_memories: true, project_id: session_model.project.key().to_string() })
  }

  async fn refresh_rolling_summary(&self) -> anyhow::Result<()> {
    let store = ManagedMemoryStore::new(BlprntPath::memories_root().join(&self.project_id));
    store.ensure_rolling_summary_file()?;

    let today = local_today();
    let Some(prior_path) = ManagedMemoryStore::newest_prior_daily_path_info(store.root(), today)? else {
      return Ok(());
    };

    let Some(session_model) = self.select_summary_refresh_session().await else {
      tracing::warn!("Memory summary refresh skipped: no session available for one-off model selection");
      return Ok(());
    };

    let existing_summary = store.read_rolling_summary_markdown().unwrap_or_default();
    let prior_daily = ManagedMemoryStore::read_all_daily_logs(&prior_path.absolute_path).await?;

    tracing::info!("Refreshing rolling memory summary");
    tracing::info!("Prior daily: {prior_daily}");

    if prior_daily.is_empty() {
      return Ok(());
    }

    let prompt = build_summary_refresh_prompt(&existing_summary, &prior_path, &prior_daily);

    tracing::info!("Prompt: {prompt}");

    let Some(response_text) = same_provider_one_off_text_for_session(
      &session_model,
      prompt,
      summary_refresh_system_prompt(),
      CancellationToken::new(),
    )
    .await?
    else {
      tracing::warn!("Memory summary refresh skipped: no same-provider small model available");
      return Ok(());
    };

    tracing::info!("Response: {response_text}");

    store.rewrite_rolling_summary_markdown(&response_text)?;
    QmdMemorySearchService::new(self.project_id.clone()).refresh().await?;
    Ok(())
  }

  async fn select_summary_refresh_session(&self) -> Option<persistence::prelude::SessionRecord> {
    SessionRepositoryV2::last_used_session_id().await
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct WindowProcessingOutcome {
  wrote_memories: bool,
  project_id:     String,
}

impl WindowProcessingOutcome {
  fn merge(&mut self, other: Self) {
    self.wrote_memories |= other.wrote_memories;
  }
}

fn split_window_by_local_day(window: DirtySessionWindow) -> Vec<DirtyDayWindow> {
  let mut windows = Vec::new();
  let mut current_date = None;
  let mut current_messages = Vec::new();

  for message in window.messages {
    let message_date = message.created_at.with_timezone(&Local).date_naive();
    if current_date.is_some_and(|date| date != message_date) {
      windows.push(DirtyDayWindow {
        session_id: window.session_id.clone(),
        date:       current_date.expect("current date missing while splitting dirty windows"),
        messages:   std::mem::take(&mut current_messages),
      });
    }

    current_date = Some(message_date);
    current_messages.push(message);
  }

  if let Some(date) = current_date {
    windows.push(DirtyDayWindow { session_id: window.session_id, date, messages: current_messages });
  }

  windows
}

fn build_extraction_prompt(trigger: &str, day_window: &DirtyDayWindow) -> Option<String> {
  let messages = day_window.messages.iter().filter_map(prompt_message).collect::<Vec<_>>();

  if messages.is_empty() {
    None
  } else {
    let messages_json = serde_json::to_string_pretty(&messages).ok()?;

    Some(format!(
      concat!(
        "Trigger: {trigger}\n",
        "Session: {session_id}\n",
        "Local day: {date}\n",
        "Dirty messages: {message_count}\n\n",
        "Extract durable memory from this raw dirty window. Return appendable markdown only.\n\n",
        "What to capture:\n",
        "- substantial work that was actually done\n",
        "- user goals, preferences, constraints, and corrections that will matter later\n",
        "- concrete implementation details that future work may need\n",
        "- outcomes, decisions, tradeoffs, failures, follow-ups, and verification results\n",
        "- ongoing context that would save time if another agent resumed the work later\n\n",
        "What to avoid:\n",
        "- tiny one-liners with no context\n",
        "- generic statements like 'worked on memory system'\n",
        "- empty praise, conversational filler, or assistant politeness\n",
        "- speculative claims or details not grounded in the messages\n",
        "- JSON, code fences, metadata fields, or rigid schemas\n\n",
        "Write style requirements:\n",
        "- use markdown headings\n",
        "- under each heading, write contextful prose or dense bullets\n",
        "- be specific about files, crates, commands, bugs, errors, and changed behavior when those appear\n",
        "- prefer reusable narrative summaries over terse notes\n",
        "- capture enough detail that a future session can reuse the memory without rereading the full conversation\n",
        "- if the window contains meaningful implementation work, decisions, or debugging, the output should usually be multiple sentences per heading, not a fragment\n\n",
        "Good example:\n",
        "## Memory system refactor\n",
        "We continued the simplification of blprnt's memory architecture away from the older type-heavy and project-scoped model. Our work focused on making daily logs plain markdown under `memories/daily/YYYY-MM-DD.md` and consolidating long-term memory into a single rolling summary at `memories/summary.md`. I removed stale `memory_update` plumbing from common tool surfaces, prompt text, bindings, and UI, then verified that `memory_search` was global-only and that QMD shaping used plain markdown content instead of semantic or episodic entry metadata.\n\n",
        "## Shell tool works arounds\n",
        "I discovered that the `/bin/bash -c` wrapper used by the shell tool does not work on Windows. I added a workaround to use the `powershell -c` wrapper instead.\n\n",
        "## Create React App issues\n",
        "After several failed attempts to run `npx create-react-app my-app`, I discovered that the installation process was not working. The issue was that the shell tool is non-interactive. Switching to the terminal tool fixed the issue.",
        "Bad example:\n",
        "## Memory\n",
        "Updated memory stuff.\n\n",
        "Another bad example:\n",
        "## Progress\n",
        "Worked on prompts.\n\n",
        "If nothing in the messages is durable or worth reusing later, return an empty string.\n\n",
        "Messages:\n{messages_json}"
      ),
      trigger = trigger,
      session_id = day_window.session_id,
      date = day_window.date.format("%Y-%m-%d"),
      message_count = day_window.messages.len(),
      messages_json = messages_json,
    ))
  }
}

fn prompt_message(message: &MessageRecord) -> Option<SweepPromptMessage> {
  summarize_message_content(message.content()).map(|c| SweepPromptMessage {
    id:         message.id.to_string(),
    created_at: message.created_at.with_timezone(&Local).to_rfc3339(),
    role:       serde_json::to_value(message.role()).unwrap_or_else(|_| Value::String("unknown".to_string())),
    visibility: serde_json::to_value(message.visibility()).unwrap_or_else(|_| Value::String("unknown".to_string())),

    content: c,
  })
}

fn summarize_message_content(content: &MessageContent) -> Option<String> {
  match content {
    MessageContent::Text(text) => Some(text.text.trim().to_string()),
    _ => None,
  }
}

fn normalize_markdown_response(response_text: &str) -> String {
  let trimmed = response_text.trim();
  if trimmed.is_empty() {
    return String::new();
  }

  for prefix in ["```markdown", "```md", "```MD", "```json", "```JSON", "```"] {
    if let Some(inner) = trimmed.strip_prefix(prefix).and_then(|value| value.strip_suffix("```")) {
      return inner.trim().to_string();
    }
  }

  trimmed.to_string()
}

fn build_summary_refresh_prompt(existing_summary: &str, prior_path: &MemoryPathInfo, prior_daily: &str) -> String {
  format!(
    concat!(
      "Refresh the rolling memory summary.\n",
      "Use only the existing summary and the newest prior daily log.\n",
      "Exclude today's log.\n",
      "Soft token budget: {token_budget} tokens.\n",
      "Rewrite the maintained rolling summary to fit within that soft budget.\n",
      "Target: compact durable markdown summary.\n\n",
      "Existing summary:\n{existing_summary}\n\n",
      "Newest prior daily log ({prior_path}):\n{prior_daily}\n"
    ),
    token_budget = MemorySummaryContract::SOFT_TOKEN_BUDGET,
    existing_summary = existing_summary.trim(),
    prior_path = prior_path.relative_display_path(),
    prior_daily = prior_daily.trim(),
  )
}

fn summary_refresh_system_prompt() -> String {
  format!(
    concat!(
      "Refresh the rolling blprnt memory summary. ",
      "Return plain markdown only. Keep it compact and durable. ",
      "Preserve important long-term facts, constraints, preferences, and ongoing context. ",
      "Rewrite and condense the maintained summary to stay within a soft budget of about {token_budget} tokens. ",
      "Attempt to keep the existing summary verbatim, if token budget allows. Only compact previous summary if it will exceed the budget by a large margin. ",
      "It is a soft token budget, not a hard limit. The goal is to keep the summary compact and durable, not to fit within the budget exactly."
    ),
    token_budget = MemorySummaryContract::SOFT_TOKEN_BUDGET,
  )
}

#[cfg(test)]
mod tests {

  use chrono::TimeZone;
  use common::shared::prelude::HistoryVisibility;
  use common::shared::prelude::MessageRole;
  use common::shared::prelude::MessageText;
  use surrealdb::types::RecordId;
  use surrealdb::types::Uuid;

  use super::*;

  fn message(
    session_id: &SurrealId,
    index: u32,
    created_at: chrono::DateTime<chrono::Utc>,
    text: &str,
  ) -> MessageRecord {
    MessageRecord {
      id: SurrealId::from(RecordId::new("messages", Uuid::new_v7())),
      rel_id: format!("rel-{index}"),
      turn_id: Uuid::new_v7(),
      step_id: Uuid::new_v7(),
      role: MessageRole::Assistant,
      content: MessageText { text: text.to_string(), signature: None }.into(),
      token_usage: None,
      visibility: HistoryVisibility::Full,
      reasoning_effort: None,
      memory_dirty: Some(true),
      created_at,
      updated_at: created_at,
      session_id: session_id.clone(),
      parent_id: None,
    }
  }

  #[test]
  fn split_window_by_local_day_groups_messages_without_reordering() {
    let session_id = SurrealId::from(RecordId::new("sessions", Uuid::new_v7()));
    let window = DirtySessionWindow {
      session_id: session_id.clone(),
      messages:   vec![
        message(&session_id, 1, chrono::Utc.with_ymd_and_hms(2026, 3, 6, 10, 0, 0).single().expect("valid ts"), "a"),
        message(&session_id, 2, chrono::Utc.with_ymd_and_hms(2026, 3, 6, 18, 0, 0).single().expect("valid ts"), "b"),
        message(&session_id, 3, chrono::Utc.with_ymd_and_hms(2026, 3, 7, 10, 0, 0).single().expect("valid ts"), "c"),
      ],
    };

    let windows = split_window_by_local_day(window);

    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].messages.iter().map(|item| item.rel_id.as_str()).collect::<Vec<_>>(), vec!["rel-1", "rel-2"]);
    assert_eq!(windows[1].messages.iter().map(|item| item.rel_id.as_str()).collect::<Vec<_>>(), vec!["rel-3"]);
  }

  #[test]
  fn normalize_markdown_response_accepts_fenced_markdown_and_rejects_blank_payloads() {
    let parsed = normalize_markdown_response("```markdown\n## Preferences\nKeep it small.\n```");

    assert_eq!(parsed, "## Preferences\nKeep it small.");
    assert!(normalize_markdown_response("  ").is_empty());
  }

  #[test]
  fn build_summary_refresh_prompt_uses_existing_summary_and_prior_daily_only() {
    let path = MemoryPathInfo::for_date(
      std::path::PathBuf::from("memories"),
      chrono::NaiveDate::from_ymd_opt(2026, 3, 7).expect("valid date"),
    );

    let prompt = build_summary_refresh_prompt("## Summary\nExisting", &path, "## Daily\nPrior log");

    assert!(prompt.contains("Existing summary:\n## Summary\nExisting"));
    assert!(prompt.contains("Newest prior daily log (memories/daily/2026-03-07.md):\n## Daily\nPrior log"));
    assert!(prompt.contains("Exclude today's log."));
    assert!(prompt.contains("Soft token budget: 1200 tokens."));
  }

  #[test]
  fn summary_refresh_system_prompt_includes_explicit_soft_budget() {
    let prompt = summary_refresh_system_prompt();

    assert!(prompt.contains("soft budget of about 1200 tokens"));
  }
}
