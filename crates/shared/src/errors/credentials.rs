#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
  #[error("codex credentials not found")]
  CodexCredentialsNotFound,

  #[error("claude credentials not found")]
  ClaudeCredentialsNotFound,
}
