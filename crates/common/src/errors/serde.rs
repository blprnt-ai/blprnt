#[derive(Debug, thiserror::Error)]
pub enum SerdeError {
  #[error("failed to deserialize from plain: {0}")]
  FailedToDeserializeFromPlain(String),

  #[error("failed to deserialize from JSON: {0}")]
  FailedToDeserializeFromJson(String),

  #[error("invalid surreal id: {0}")]
  InvalidSurrealId(String),
}

impl From<serde_plain::Error> for SerdeError {
  fn from(error: serde_plain::Error) -> Self {
    SerdeError::FailedToDeserializeFromPlain(error.to_string())
  }
}

impl From<serde_json::Error> for SerdeError {
  fn from(error: serde_json::Error) -> Self {
    SerdeError::FailedToDeserializeFromJson(error.to_string())
  }
}
