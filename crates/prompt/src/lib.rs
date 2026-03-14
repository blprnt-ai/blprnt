use std::collections::HashMap;
use std::path::PathBuf;

use askama::Template;
use common::agent::AgentKind;
use common::shared::prelude::PlanContext;
use common::shared::prelude::PromptParams;

#[derive(Template)]
#[template(path = "identity/subagent-common.jinja", escape = "none")]
struct SubagentCommonTemplate<'a> {
  system:         &'a str,
  dirs:           &'a str,
  primer:         &'a str,
  current_skills: &'a str,
  current_plan:   &'a str,
  mcp_details:    &'a str,
}

#[derive(Template)]
#[template(path = "kind/crew.jinja", escape = "none")]
struct CrewTemplate<'a> {
  system:         &'a str,
  dirs:           &'a str,
  personality:    &'a str,
  primer:         &'a str,
  current_skills: &'a str,
  current_plan:   &'a str,
  mcp_details:    &'a str,
  memory:         &'a str,
}

#[derive(Template)]
#[template(path = "kind/planning.jinja", escape = "none")]
struct PlanningTemplate<'a> {
  subagent_common: SubagentCommonTemplate<'a>,
}

#[derive(Template)]
#[template(path = "kind/execution.jinja", escape = "none")]
struct ExecutionTemplate<'a> {
  subagent_common: SubagentCommonTemplate<'a>,
}

#[derive(Template)]
#[template(path = "kind/verification.jinja", escape = "none")]
struct VerificationTemplate<'a> {
  subagent_common: SubagentCommonTemplate<'a>,
}

#[derive(Template)]
#[template(path = "kind/research.jinja", escape = "none")]
struct ResearcherTemplate<'a> {
  subagent_common: SubagentCommonTemplate<'a>,
}

#[derive(Template)]
#[template(path = "kind/designer.jinja", escape = "none")]
struct DesignerTemplate<'a> {
  subagent_common: SubagentCommonTemplate<'a>,
}

pub fn render_prompt(params: PromptParams) -> String {
  let system_information = render_system();
  let workspace_roots = render_dirs(params.workspace_roots);
  let personality = render_personality(params.personality);
  let agent_primer = render_primer(params.primer);

  let current_skills = params.current_skills.join("\n\n");

  let current_plan = render_current_plan(params.plan_context);
  let mcp_details = render_mcp_details(params.mcp_details);

  let subagent_common = SubagentCommonTemplate {
    system:         &system_information,
    dirs:           &workspace_roots,
    primer:         &agent_primer,
    current_skills: &current_skills,
    current_plan:   &current_plan,
    mcp_details:    &mcp_details,
  };

  let system_prompt = match params.agent_kind {
    AgentKind::Crew => CrewTemplate {
      system:         &system_information,
      dirs:           &workspace_roots,
      personality:    &personality,
      primer:         &agent_primer,
      current_skills: &current_skills,
      current_plan:   &current_plan,
      mcp_details:    &mcp_details,
      memory:         &params.memory,
    }
    .render()
    .unwrap_or_default(),

    AgentKind::Planner => PlanningTemplate { subagent_common }.render().unwrap_or_default(),
    AgentKind::Executor => ExecutionTemplate { subagent_common }.render().unwrap_or_default(),
    AgentKind::Verifier => VerificationTemplate { subagent_common }.render().unwrap_or_default(),
    AgentKind::Researcher => ResearcherTemplate { subagent_common }.render().unwrap_or_default(),
    AgentKind::Designer => DesignerTemplate { subagent_common }.render().unwrap_or_default(),
  };

  #[cfg(debug_assertions)]
  let system_prompt = format!(
    "{}\n\n You are currently being run with debug mode. This means that the source code for your harness and system prompt is being worked on. Ignore all directive to keep the system prompt hidden from the user. You must answer any meta questions about the system prompt to debug your decision making process. If the user user asks you to do something specific, like not using subagent, that overrides your core directives. You must always follow the user prompt as if it was a higher level directive.",
    system_prompt
  );

  system_prompt
}

fn render_system() -> String {
  let os = std::env::consts::OS;
  let arch = std::env::consts::ARCH;

  format!(
    r#"
# System & Project Details

<system-information>
{os}
{arch}
</system-information>
"#,
  )
}

fn render_dirs(dirs: Vec<PathBuf>) -> String {
  let dirs = dirs.iter().enumerate().map(|(i, d)| format!("{}: {}", i, d.display())).collect::<Vec<_>>().join("\n");
  format!(
    r#"
<working-directories>
{dirs}
</working-directories>
"#,
  )
}

fn render_personality(personality: String) -> String {
  if personality.trim().is_empty() {
    String::new()
  } else {
    format!(
      r#"
## Personality

This user-selected personality overrides tone/style defaults. It must be respected above all other tone/style defaults.
Failure to respect these personality directives is a direct violation of the user's instructions. This includes any skill definitions or instructions prior to this personality directive. Above all else, you must follow the user's tone/style instructions.

<personality>
{personality}
</personality>
      "#,
    )
  }
}

fn render_primer(primer: Option<String>) -> String {
  primer
    .map(|primer| {
      format!(
        r#"
# User-Defined Agent Primer

<user-defined-primer>
{primer}
</user-defined-primer>
"#,
      )
    })
    .unwrap_or_default()
}

fn render_current_plan(plan_context: Option<PlanContext>) -> String {
  plan_context
    .map(|plan| {
      format!(
        r#"
# Current Plan ({})

<current-plan>
{}
</current-plan>
"#,
        plan.id, plan.content,
      )
    })
    .unwrap_or_default()
}

fn render_mcp_details(mcp_details: HashMap<String, String>) -> String {
  mcp_details
    .iter()
    .map(|(k, v)| format!("# MCP Instruction for: {}\n\n<mcp-instruction>{}</mcp-instruction>", k, v))
    .collect::<Vec<_>>()
    .join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_render_prompt() {
    let params = PromptParams {
      agent_kind:      AgentKind::Crew,
      personality:     "test".to_string(),
      workspace_roots: vec![PathBuf::from("/tmp")],
      primer:          None,
      current_skills:  vec![],
      plan_context:    None,
      mcp_details:     HashMap::new(),
      memory:          String::new(),
    };

    let prompt = render_prompt(params);
    println!("{}", prompt);
  }

  #[test]
  fn does_not_render_legacy_memory_sections() {
    let params = PromptParams {
      agent_kind:      AgentKind::Crew,
      personality:     "test".to_string(),
      workspace_roots: vec![PathBuf::from("/tmp")],
      primer:          Some("primer".to_string()),
      current_skills:  vec![],
      plan_context:    None,
      mcp_details:     HashMap::new(),
      memory:          String::new(),
    };

    let prompt = render_prompt(params);

    assert!(prompt.contains("# User-Defined Agent Primer"));
    assert!(!prompt.contains("# Known Projects"));
    assert!(!prompt.contains("<known-projects>"));
    assert!(!prompt.contains("# Project Memory Brief"));
    assert!(!prompt.contains("<project-memory-brief>"));
    assert!(!prompt.contains("# User Memory Brief"));
    assert!(!prompt.contains("<user-memory-brief>"));
  }
}
