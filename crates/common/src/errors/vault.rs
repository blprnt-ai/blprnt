#[derive(Debug, thiserror::Error)]
pub enum VaultError {
  #[error("failed to set stronghold secret for {item}: {error}")]
  FailedToSetSecret { item: String, error: String },

  #[error("failed to delete stronghold secret for {item}: {error}")]
  FailedToDeleteSecret { item: String, error: String },

  #[error("failed to commit stronghold secret: {error}")]
  FailedToCommitSecret { error: String },

  #[error("failed to get stronghold client: {error}")]
  FailedToGetClient { error: String },
}
