use std::fs;
use std::path::Path;
use std::path::PathBuf;

use persistence::Uuid;
use persistence::prelude::RunTrigger;

const BLPRNT_SYSTEM_PROMPT_STUB: &str = include_str!("prompts/blprnt-system-prompt.md");

#[derive(Clone, Debug)]
pub struct PromptAssemblyInput {
  pub agent_home:       PathBuf,
  pub project_home:     Option<PathBuf>,
  pub employee_id:      String,
  pub api_url:          String,
  pub operating_system: String,
  pub heartbeat_prompt: String,
  pub trigger:          RunTrigger,
  pub issue_id:         Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct BuiltPrompt {
  pub system_prompt: String,
  pub user_prompt:   String,
}

impl PromptAssemblyInput {
  pub fn build(self) -> BuiltPrompt {
    let mut system_sections = vec![
      BLPRNT_SYSTEM_PROMPT_STUB.trim().to_string(),
      format!(
        "## Runtime Metadata\nOperating system: {}\nEmployee ID: {}\nAPI URL: {}\nAGENT_HOME: {}{}",
        self.operating_system,
        self.employee_id,
        self.api_url,
        self.agent_home.display(),
        self.project_home.as_ref().map(|path| format!("\nPROJECT_HOME: {}", path.display())).unwrap_or_default()
      ),
    ];

    if let Some(heartbeat) = read_optional_markdown(self.agent_home.join("HEARTBEAT.md")) {
      system_sections.push(format!("## HEARTBEAT.md\n{heartbeat}"));
    }

    if let Some(agents) = read_optional_markdown(self.agent_home.join("AGENTS.md")) {
      system_sections.push(format!("## AGENTS.md\n{agents}"));
    }

    if !self.heartbeat_prompt.trim().is_empty() {
      system_sections.push(format!("## Employee Runtime Prompt\n{}", self.heartbeat_prompt.trim()));
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

fn build_user_prompt(input: &PromptAssemblyInput) -> String {
  let mut sections = vec!["Use the blprnt API to continue your blprnt work.".to_string()];

  match &input.trigger {
    RunTrigger::Manual => sections.push("Trigger: manual".to_string()),
    RunTrigger::Timer => sections.push("Trigger: timer".to_string()),
    RunTrigger::IssueAssignment { .. } => {
      sections.push("Trigger: issue_assignment".to_string());

      let mut issue_lines = Vec::new();
      if let Some(issue_id) = &input.issue_id {
        issue_lines.push(format!("Issue ID: {issue_id}"));
      }

      if !issue_lines.is_empty() {
        sections.push(issue_lines.join("\n"));
      }
    }
  }

  sections.join("\n\n")
}
