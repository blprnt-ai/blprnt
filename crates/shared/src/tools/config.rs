use crate::agent::AgentKind;
use crate::tools::WorkingDirectories;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct LlmModel {
  pub name:               String,
  pub slug:               String,
  pub context_length:     i64,
  pub supports_reasoning: bool,
  pub provider_slug:      Option<String>,
  pub enabled:            bool,
}

pub struct ToolsSchemaConfig {
  pub agent_kind:           AgentKind,
  pub working_directories:  WorkingDirectories,
  pub is_subagent:          bool,
  pub memory_tools_enabled: bool,
  pub enabled_models:       Vec<LlmModel>,
}
