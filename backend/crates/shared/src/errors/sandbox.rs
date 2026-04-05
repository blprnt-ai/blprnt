#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
  #[error("unknown sandbox root: {0}")]
  UnknownRoot(String),

  #[error("workspace {path} is not in sandbox {name}")]
  WorkspaceNotInSandbox { name: String, path: String },

  #[error("root {path} is not in sandbox {name}")]
  RootNotInSandbox { name: String, path: String },

  #[error("failed to open sandbox directory {path}: {error}")]
  FailedToOpenDirectory { path: String, error: String },

  #[error("failed to create file {path}: {error}")]
  FailedToCreateFile { path: String, error: String },

  #[error("failed to open file {path}: {error}")]
  FailedToOpenFile { path: String, error: String },

  #[error("failed to open write-only file {path}: {error}")]
  FailedToOpenWriteOnlyFile { path: String, error: String },

  #[error("failed to remove file {path}: {error}")]
  FailedToRemoveFile { path: String, error: String },

  #[error("failed to create parent directories {path}: {error}")]
  FailedToCreateParentDirectories { path: String, error: String },

  #[error("invalid file path {path}: {error}")]
  InvalidFilePath { path: String, error: String },
}
