#[derive(Debug, thiserror::Error)]
pub enum SessionError {
  #[error("failed to create session: {0}")]
  FailedToCreate(String),

  #[error("failed to open session: {0}")]
  FailedToOpen(String),

  #[error("user interrupted")]
  UserInterrupted,
}
