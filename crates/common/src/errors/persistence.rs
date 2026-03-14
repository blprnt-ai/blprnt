#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
  #[error("database not found: {context}: {message}")]
  DatabaseNotFound { context: String, message: String },

  #[error("invalid arguments: {context}: {message}")]
  InvalidArguments { context: String, message: String },

  #[error("internal database error: {context}: {message}")]
  InternalDb { context: String, message: String },

  #[error("resource not found: {resource} ({id}): {context}")]
  NotFound { resource: String, id: String, context: String },

  #[error("cannot delete project with items: {0}")]
  CannotDeleteProjectWithItems(String),
}
