use std::fmt::Display;
use std::fmt::Formatter;

use axum::Json;
use axum::response::IntoResponse;
use http::StatusCode;

use crate::errors::ErrorEvent;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TauriError {
  pub message: String,
  pub error:   Option<ErrorEvent>,
  pub trace:   Option<Vec<String>>,
}

impl TauriError {
  pub fn new(message: impl Into<String>) -> Self {
    Self { message: message.into(), ..Default::default() }
  }
}

impl Display for TauriError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for TauriError {}

impl From<anyhow::Error> for TauriError {
  fn from(error: anyhow::Error) -> Self {
    let trace = error.chain().map(|e| e.to_string()).collect();

    Self { message: error.to_string(), error: Some(error.into()), trace: Some(trace) }
  }
}

impl From<String> for TauriError {
  fn from(message: String) -> Self {
    Self { message, ..Default::default() }
  }
}

impl From<&str> for TauriError {
  fn from(message: &str) -> Self {
    Self { message: message.to_string(), ..Default::default() }
  }
}

/// Type alias for Tauri command results
pub type TauriResult<T> = Result<T, TauriError>;

/// Extension trait for converting anyhow::Result to TauriResult
pub trait IntoTauriResult<T> {
  fn into_tauri(self) -> TauriResult<T>;
}

impl<T> IntoTauriResult<T> for anyhow::Result<T> {
  fn into_tauri(self) -> TauriResult<T> {
    self.map_err(Into::into)
  }
}

impl IntoResponse for TauriError {
  fn into_response(self) -> axum::response::Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
  }
}
