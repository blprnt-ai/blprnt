use std::path::PathBuf;

use common::agent::AgentKind;
use common::sandbox_flags::SandboxFlags;
use common::shared::prelude::SurrealId;

#[derive(Clone, Debug)]
pub struct ToolUseContext {
  pub project_id:           SurrealId,
  pub agent_kind:           AgentKind,
  pub working_directories:  Vec<PathBuf>,
  pub sandbox_flags:        SandboxFlags,
  pub sandbox_key:          String,
  pub is_subagent:          bool,
  pub memory_tools_enabled: bool,
  pub session_id:           SurrealId,
  pub parent_id:            Option<SurrealId>,
  pub current_skills:       Vec<String>,
}

impl ToolUseContext {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    session_id: SurrealId,
    parent_id: Option<SurrealId>,
    project_id: SurrealId,
    agent_kind: AgentKind,
    working_directories: Vec<PathBuf>,
    current_skills: Vec<String>,
    sandbox_flags: SandboxFlags,
    sandbox_key: String,
    is_subagent: bool,
  ) -> Self {
    Self::new_with_memory_tools_enabled(
      session_id,
      parent_id,
      project_id,
      agent_kind,
      working_directories,
      current_skills,
      sandbox_flags,
      sandbox_key,
      is_subagent,
      true,
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub fn new_with_memory_tools_enabled(
    session_id: SurrealId,
    parent_id: Option<SurrealId>,
    project_id: SurrealId,
    agent_kind: AgentKind,
    working_directories: Vec<PathBuf>,
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
      sandbox_flags: sandbox_flags,
      sandbox_key: sandbox_key,
      is_subagent: is_subagent,
      memory_tools_enabled,
      current_skills: current_skills,
      session_id: session_id,
      parent_id: parent_id,
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
