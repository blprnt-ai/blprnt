#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
  #[error("failed to set stronghold secret for {item}: {error}")]
  FailedToSetSecret { item: String, error: String },

  #[error("failed to delete stronghold secret for {item}: {error}")]
  FailedToDeleteSecret { item: String, error: String },
}
