use anyhow::Result;

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
  #[error("memory store io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("invalid path: {0}")]
  InvalidPath(String),

  #[error("qmd installation failed")]
  QmdInstallationFailed,

  #[error("qmd collection initialization failed")]
  QmdCollectionInitializationFailed,
}

pub type MemoryResult<T> = Result<T, MemoryError>;
