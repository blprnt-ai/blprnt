use anyhow::Result;

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
  #[error("memory store io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("invalid path: {0}")]
  InvalidPath(String),

  #[error("project not found: {0}")]
  ProjectNotFound(String),

  #[error("project lookup failed: {0}")]
  ProjectLookupFailed(String),

  #[error("qmd installation failed")]
  QmdInstallationFailed,

  #[error("qmd collection initialization failed")]
  QmdCollectionInitializationFailed,

  #[error("qmd operation failed: {0}")]
  QmdOperationFailed(String),
}

pub type MemoryResult<T> = Result<T, MemoryError>;
