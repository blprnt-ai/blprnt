use crate::agent::AgentKind;
use crate::api::LlmModelResponse;
use crate::tools::WorkingDirectories;

pub struct ToolsSchemaConfig {
  pub agent_kind:           AgentKind,
  pub working_directories:  WorkingDirectories,
  pub is_subagent:          bool,
  pub memory_tools_enabled: bool,
  pub enabled_models:       Vec<LlmModelResponse>,
}
