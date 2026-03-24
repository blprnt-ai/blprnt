use std::path::PathBuf;

use persistence::prelude::ProjectId;
use shared::agent::AgentKind;
use shared::sandbox_flags::SandboxFlags;
use shared::tools::config::ToolRuntimeConfig;

#[derive(Clone, Debug)]
pub struct ToolUseContext {
  pub project_id:           Option<ProjectId>,
  pub agent_kind:           AgentKind,
  pub working_directories:  Vec<PathBuf>,
  pub runtime_config:       ToolRuntimeConfig,
  pub sandbox_flags:        SandboxFlags,
  pub sandbox_key:          String,
  pub is_subagent:          bool,
  pub memory_tools_enabled: bool,
  pub current_skills:       Vec<String>,
}

impl ToolUseContext {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    project_id: Option<ProjectId>,
    agent_kind: AgentKind,
    working_directories: Vec<PathBuf>,
    runtime_config: ToolRuntimeConfig,
    current_skills: Vec<String>,
    sandbox_flags: SandboxFlags,
    sandbox_key: String,
    is_subagent: bool,
  ) -> Self {
    Self::new_with_memory_tools_enabled(
      project_id,
      agent_kind,
      working_directories,
      runtime_config,
      current_skills,
      sandbox_flags,
      sandbox_key,
      is_subagent,
      true,
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new_with_memory_tools_enabled(
    project_id: Option<ProjectId>,
    agent_kind: AgentKind,
    working_directories: Vec<PathBuf>,
    runtime_config: ToolRuntimeConfig,
    current_skills: Vec<String>,
    sandbox_flags: SandboxFlags,
    sandbox_key: String,
    is_subagent: bool,
    memory_tools_enabled: bool,
  ) -> Self {
    Self {
      project_id: project_id,
      agent_kind: agent_kind,
      working_directories: working_directories,
      runtime_config,
      sandbox_flags: sandbox_flags,
      sandbox_key: sandbox_key,
      is_subagent: is_subagent,
      memory_tools_enabled,
      current_skills: current_skills,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResultsData {
  pub pattern: String,
  pub matches: Vec<SearchMatch>,
  pub total:   usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchMatch {
  pub file_path:   PathBuf,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line_number: Option<usize>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line:        Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub column:      Option<usize>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeResultData {
  pub query:    String,
  pub response: serde_json::Value,
  pub sources:  Vec<String>,
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use shared::tools::config::ToolRuntimeConfig;

  #[test]
  fn runtime_config_exports_expected_environment_variables() {
    let runtime = ToolRuntimeConfig {
      agent_home:   Some(PathBuf::from("/tmp/agent-home")),
      project_home: Some(PathBuf::from("/tmp/project-home")),
      employee_id:  Some("employee-123".to_string()),
      api_url:      Some("http://127.0.0.1:3100".to_string()),
    };

    let env = runtime.env_overrides();

    assert_eq!(env.get("AGENT_HOME").map(String::as_str), Some("/tmp/agent-home"));
    assert_eq!(env.get("PROJECT_HOME").map(String::as_str), Some("/tmp/project-home"));
    assert_eq!(env.get("BLPRNT_EMPLOYEE_ID").map(String::as_str), Some("employee-123"));
    assert_eq!(env.get("BLPRNT_API_URL").map(String::as_str), Some("http://127.0.0.1:3100"));
  }
}
