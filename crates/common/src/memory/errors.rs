use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagedMemoryStoreError {
  #[error("memory store io error: {0}")]
  Io(#[from] io::Error),
  #[error("memory content must not be empty")]
  EmptyContent,
}
