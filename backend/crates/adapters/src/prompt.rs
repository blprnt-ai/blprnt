use std::fs;
use std::path::Path;
use std::path::PathBuf;

use persistence::Uuid;
use persistence::prelude::DbId;
use persistence::prelude::IssuePriority;
use persistence::prelude::IssueStatus;
use persistence::prelude::RunTrigger;
use shared::tools::McpServerAuthState;
use skills::SkillRef;

const BLPRNT_SYSTEM_PROMPT_STUB: &str = include_str!("prompts/blprnt-system-prompt.md");

#[derive(Clone, Debug)]
pub struct PromptAssemblyInput {
  pub agent_home:            PathBuf,
  pub project_home:          Option<PathBuf>,
  pub project_workdirs:      Vec<PathBuf>,
  pub employee_id:           String,
  pub api_url:               String,
  pub operating_system:      String,
  pub heartbeat_prompt:      String,
  pub available_skills:      Vec<SkillRef>,
  pub injected_skill_stack:  Vec<InjectedSkillPrompt>,
  pub trigger:               RunTrigger,
  pub dreaming_date:         Option<String>,
  pub daily_memory_content:  Option<String>,
  pub prior_memory_content:  Option<String>,
  pub issue_id:              Option<Uuid>,
  pub issue_identifier:      Option<String>,
  pub issue_title:           Option<String>,
  pub issue_description:     Option<String>,
  pub issue_status:          Option<IssueStatus>,
  pub issue_priority:        Option<IssuePriority>,
  pub trigger_comment:       Option<String>,
  pub trigger_commenter:     Option<String>,
  pub available_mcp_servers: Vec<PromptMcpServerCatalogEntry>,
}

#[derive(Clone, Debug)]
pub struct PromptMcpServerCatalogEntry {
  pub server_id:    String,
  pub display_name: String,
  pub description:  String,
  pub auth_state:   McpServerAuthState,
}

#[derive(Clone, Debug)]
pub struct BuiltPrompt {
  pub system_prompt: String,
  pub user_prompt:   String,
}

#[derive(Clone, Debug)]
pub struct InjectedSkillPrompt {
  pub name:     String,
  pub path:     String,
  pub contents: String,
}

impl PromptAssemblyInput {
  pub fn build(self) -> BuiltPrompt {
    if matches!(self.trigger, RunTrigger::Dreaming) {
      return build_dreaming_prompt(&self);
    }

    let mut system_sections = vec![
      BLPRNT_SYSTEM_PROMPT_STUB.trim().to_string(),
      format!(
        "## Runtime Metadata\nOperating system: {}\nEmployee ID: {}\nAPI URL: {}\nAGENT_HOME: {}",
        self.operating_system,
        self.employee_id,
        self.api_url,
        self.agent_home.display(),
      ),
    ];

    if let Some(project_home) = &self.project_home {
      system_sections.push(format!(
        "## Project Directories\nPROJECT_HOME: {}\nUse PROJECT_HOME for blprnt-managed project metadata only, not for primary source work.\nPROJECT_HOME is writable as a whole for blprnt-managed files.\nPROJECT_HOME/memory stores project memory files.\nPROJECT_HOME/plans stores plan documents and is the correct place for project plan files.",
        project_home.display()
      ));
    }

    if !self.project_workdirs.is_empty() {
      let lines =
        self.project_workdirs.iter().map(|path| format!("- {}", path.display())).collect::<Vec<_>>().join("\n");
      system_sections.push(format!(
        "## Project Working Directories\nThese are the actual project source/work directories. Use them for code changes and normal project file work.\n{lines}"
      ));
    }

    for file_name in ["HEARTBEAT.md", "SOUL.md", "AGENTS.md", "TOOLS.md"] {
      if let Some(contents) = read_optional_markdown(self.agent_home.join(file_name)) {
        system_sections.push(format!("## {file_name}\n{contents}"));
      }
    }

    for (workdir, contents) in read_project_agents_markdown(&self.project_workdirs) {
      system_sections.push(format!("## Project AGENTS.md ({})\n{}", workdir.join("AGENTS.md").display(), contents));
    }

    if let Some(memory) = read_optional_markdown(self.agent_home.join("MEMORY.md")) {
      system_sections.push(format!("## MEMORY.md\n{memory}"));
    }

    if !self.heartbeat_prompt.trim().is_empty() {
      system_sections.push(format!("## Employee Runtime Prompt\n{}", self.heartbeat_prompt.trim()));
    }

    if !self.available_skills.is_empty() {
      let lines = self
        .available_skills
        .iter()
        .map(|skill| format!("- {}\n  - {}\n  - {}", skill.name, skill.path, skill.description))
        .collect::<Vec<_>>()
        .join("\n");
      system_sections.push(format!("## Available Runtime Skills\n{lines}"));
    }

    for skill in &self.injected_skill_stack {
      system_sections.push(format!(
        "## Employee Skill Stack: {} ({})\n{}",
        skill.name,
        skill.path,
        skill.contents.trim()
      ));
    }

    if !self.available_mcp_servers.is_empty() {
      let lines = self
        .available_mcp_servers
        .iter()
        .map(|server| {
          format!(
            "- {} ({}) — {} [{}]",
            server.display_name,
            server.server_id,
            server.description,
            format_mcp_auth_state(&server.auth_state)
          )
        })
        .collect::<Vec<_>>()
        .join("\n");
      system_sections.push(format!(
        "## Available MCP Servers\nThese servers are configured and available to enable for this run. They are not callable until you explicitly enable one with `enable_mcp_server`.\n{lines}"
      ));
    }

    BuiltPrompt { system_prompt: system_sections.join("\n\n"), user_prompt: build_user_prompt(&self) }
  }
}

fn read_optional_markdown(path: impl AsRef<Path>) -> Option<String> {
  let path = path.as_ref();
  let content = fs::read_to_string(path).ok()?;
  let trimmed = content.trim();
  (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn read_project_agents_markdown(project_workdirs: &[PathBuf]) -> Vec<(PathBuf, String)> {
  project_workdirs
    .iter()
    .filter_map(|workdir| {
      let contents = read_optional_markdown(workdir.join("AGENTS.md"))?;
      Some((workdir.clone(), contents))
    })
    .collect()
}

fn build_user_prompt(input: &PromptAssemblyInput) -> String {
  let mut sections = vec!["Use the blprnt API to continue your blprnt work.".to_string()];

  match &input.trigger {
    RunTrigger::Manual => sections.push("Trigger: manual".to_string()),
    RunTrigger::Conversation => sections.push("Trigger: conversation".to_string()),
    RunTrigger::Timer => sections.push("Trigger: timer".to_string()),
    RunTrigger::Dreaming => sections.push("Trigger: dreaming".to_string()),
    RunTrigger::IssueAssignment { .. } | RunTrigger::IssueMention { .. } => {
      sections.push(match &input.trigger {
        RunTrigger::IssueAssignment { .. } => "Trigger: issue_assignment".to_string(),
        RunTrigger::IssueMention { .. } => "Trigger: issue_mention".to_string(),
        _ => unreachable!(),
      });

      let mut issue_lines = Vec::new();
      if let Some(issue_id) = &input.issue_id {
        issue_lines.push(format!("Issue ID: {issue_id}"));
      }
      if let Some(identifier) = input.issue_identifier.as_deref() {
        issue_lines.push(format!("Issue Identifier: {identifier}"));
      }
      if let Some(title) = input.issue_title.as_deref() {
        issue_lines.push(format!("Issue Title: {title}"));
      }
      if let Some(status) = &input.issue_status {
        issue_lines.push(format!("Issue Status: {}", format_issue_status(status)));
      }
      if let Some(priority) = &input.issue_priority {
        issue_lines.push(format!("Issue Priority: {}", format_issue_priority(priority)));
      }
      if let Some(description) = input.issue_description.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        issue_lines.push(format!("Issue Description:\n{description}"));
      }
      if let RunTrigger::IssueMention { comment_id, .. } = &input.trigger {
        issue_lines.push(format!("Triggering Comment ID: {}", comment_id.uuid()));
        if let Some(commenter) = input.trigger_commenter.as_deref().filter(|value| !value.is_empty()) {
          issue_lines.push(format!("Comment Author: {commenter}"));
        }
        if let Some(comment) = input.trigger_comment.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
          issue_lines.push(format!("Triggering Comment:\n{comment}"));
        }
      }

      if !issue_lines.is_empty() {
        sections.push(issue_lines.join("\n"));
      }
    }
  }

  sections.join("\n\n")
}

fn build_dreaming_prompt(input: &PromptAssemblyInput) -> BuiltPrompt {
  let date = input.dreaming_date.as_deref().unwrap_or("unknown");
  let daily_memory = input.daily_memory_content.as_deref().unwrap_or("").trim();
  let prior_memory = input.prior_memory_content.as_deref().unwrap_or("").trim();

  BuiltPrompt {
    system_prompt: [
      "You are synthesizing AGENT_HOME/MEMORY.md for a blprnt employee.",
      "Output only concise markdown bullet items.",
      "Each item must use exactly this structure:",
      "- statement: <concise durable takeaway>",
      "  type: <preference|constraint|workflow|insight|relationship>",
      "  freshness: <active|decaying|stale>",
      "  last_reinforced: <YYYY-MM-DD>",
      "Reinforce existing items instead of duplicating them.",
      "Keep the result concise and cap it at 25 items.",
      "If an item was not reinforced today but still matters, you may keep it with lower freshness.",
      "If nothing deserves to be kept, return an empty response.",
    ]
    .join("\n"),
    user_prompt:   format!(
      "Trigger: dreaming\n\nEmployee ID: {}\nCurrent Date: {}\n\nToday's daily memory:\n```md\n{}\n```\n\nPrior MEMORY.md:\n```md\n{}\n```",
      input.employee_id, date, daily_memory, prior_memory,
    ),
  }
}

fn format_issue_status(status: &IssueStatus) -> &'static str {
  match status {
    IssueStatus::Backlog => "backlog",
    IssueStatus::Todo => "todo",
    IssueStatus::InProgress => "in_progress",
    IssueStatus::Blocked => "blocked",
    IssueStatus::Done => "done",
    IssueStatus::Cancelled => "cancelled",
    IssueStatus::Archived => "archived",
  }
}

fn format_issue_priority(priority: &IssuePriority) -> &'static str {
  match priority {
    IssuePriority::Low => "low",
    IssuePriority::Medium => "medium",
    IssuePriority::High => "high",
    IssuePriority::Critical => "critical",
  }
}

fn format_mcp_auth_state(state: &McpServerAuthState) -> &'static str {
  match state {
    McpServerAuthState::NotConnected => "not_connected",
    McpServerAuthState::AuthRequired => "auth_required",
    McpServerAuthState::Connected => "connected",
    McpServerAuthState::ReconnectRequired => "reconnect_required",
  }
}
