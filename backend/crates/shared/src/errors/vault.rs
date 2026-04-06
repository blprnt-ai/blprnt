#[derive(Debug, thiserror::Error)]
pub enum VaultError {
  #[error("failed to decode stronghold secret bytes as utf-8: {error}")]
  FailedToDecodeSecret { error: String },

  #[error("failed to set stronghold secret for {item}: {error}")]
  FailedToSetSecret { item: String, error: String },

  #[error("failed to delete stronghold secret for {item}: {error}")]
  FailedToDeleteSecret { item: String, error: String },

  #[error("failed to commit stronghold secret: {error}")]
  FailedToCommitSecret { error: String },

  #[error("failed to get stronghold client: {error}")]
  FailedToGetClient { error: String },

  #[error("failed to acquire vault state cache lock")]
  FailedToLockState,

  #[error("failed to obtain machine UID: {error}")]
  FailedToGetMachineUid { error: String },

  #[error("failed to derive stronghold key material: {error}")]
  FailedToDeriveKeyMaterial { error: String },

  #[error("failed to create stronghold key provider: {error}")]
  FailedToCreateKeyProvider { error: String },

  #[error("failed to create stronghold client: {error}")]
  FailedToCreateClient { error: String },
}
