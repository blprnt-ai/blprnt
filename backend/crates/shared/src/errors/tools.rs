use crate::agent::AgentKind;
use crate::agent::ToolId;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
  #[error("invalid tool args for {tool_id}: {error}")]
  InvalidArgs { tool_id: ToolId, error: String },

  #[error("failed to parse tool args for {tool_id}: {error}")]
  FailedToParseArgs { tool_id: ToolId, error: String },

  #[error("unknown tool: {0}")]
  UnknownTool(String),

  #[error("failed to rename: {0}")]
  RenameFailed(String),

  #[error("failed to find symbols: {0}")]
  SymbolsFailed(String),

  #[error("tool use error: {0}")]
  General(String),

  #[error("file read line start {line_start} beyond file end ({file_end} lines)")]
  FileReadLineStartBeyondFileEnd { line_start: usize, file_end: usize },

  #[error("file read line start {line_start} >= line end {line_end}")]
  FileReadLineStartGreaterThanLineEnd { line_start: usize, line_end: usize },

  #[error("no todo items provided")]
  NoTodoItemsProvided,

  #[error("access denied: agent type {agent_kind:?} is not allowed to use tool '{tool_id}'")]
  AccessDenied { agent_kind: AgentKind, tool_id: ToolId },

  #[error("workspace root index {index} is out of bounds (valid: 0 to {max})")]
  InvalidWorkspaceRoot { index: usize, max: usize },

  #[error("failed to read file {path}: {error}")]
  FileReadFailed { path: String, error: String },

  #[error("failed to write file {path}: {error}")]
  FileWriteFailed { path: String, error: String },

  #[error("failed to parse line for {path}: {error}")]
  PatchParseFailed { path: String, error: String },

  #[error("failed to apply patch for {path}: {error}")]
  PatchApplyFailed { path: String, error: String },

  #[error("invalid regex pattern: {0}")]
  InvalidRegex(String),

  #[error("failed to spawn process: {0}")]
  SpawnFailed(String),

  #[error("command timed out")]
  CommandTimeout,

  #[error("failed to read process output: {0}")]
  ProcessOutputFailed(String),
}
