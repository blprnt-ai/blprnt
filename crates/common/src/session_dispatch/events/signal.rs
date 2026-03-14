use serde::ser::SerializeStruct;
use surrealdb::types::ToSql;

use crate::errors::ErrorEvent;
use crate::shared::prelude::HistoryMessageSource;
use crate::shared::prelude::MessageContent;
use crate::shared::prelude::Signal;
use crate::shared::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SignalEvent {
  Error(SignalPayload),
  Info(SignalPayload),
  Warning(SignalPayload),
}

impl SignalEvent {
  pub fn with_id(self, id: SurrealId) -> Self {
    match self {
      SignalEvent::Error(payload) => SignalEvent::Error(payload.with_id(id)),
      SignalEvent::Warning(payload) => SignalEvent::Warning(payload.with_id(id)),
      SignalEvent::Info(payload) => SignalEvent::Info(payload.with_id(id)),
    }
  }
}

impl From<SignalEvent> for MessageContent {
  fn from(event: SignalEvent) -> Self {
    match event {
      SignalEvent::Error(payload) => MessageContent::Error(Signal {
        message: payload.message,
        error:   payload.error.clone(),
        source:  Some(payload.source),
        id:      payload.id,
      }),
      SignalEvent::Warning(payload) => MessageContent::Warning(Signal {
        message: payload.message,
        error:   payload.error.clone(),
        source:  Some(payload.source),
        id:      payload.id,
      }),
      SignalEvent::Info(payload) => MessageContent::Info(Signal {
        message: payload.message,
        error:   payload.error.clone(),
        source:  Some(payload.source),
        id:      payload.id,
      }),
    }
  }
}

impl SignalEvent {
  pub fn source(&self) -> HistoryMessageSource {
    match self {
      SignalEvent::Error(payload) => payload.source.clone(),
      SignalEvent::Warning(payload) => payload.source.clone(),
      SignalEvent::Info(payload) => payload.source.clone(),
    }
  }
}

pub enum SignalKind {
  Error,
  Warning,
  Info,
}

#[derive(Clone, Debug, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SignalPayload {
  pub id:      Option<SurrealId>,
  pub message: String,
  pub source:  HistoryMessageSource,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error:   Option<ErrorEvent>,
}

impl SignalPayload {
  fn into_signal_event(
    kind: SignalKind,
    message: String,
    source: HistoryMessageSource,
    error: Option<ErrorEvent>,
  ) -> SignalEvent {
    let payload = SignalPayload { id: None, message, source, error };

    match kind {
      SignalKind::Error => SignalEvent::Error(payload),
      SignalKind::Warning => SignalEvent::Warning(payload),
      SignalKind::Info => SignalEvent::Info(payload),
    }
  }

  pub fn error(message: String) -> SignalEvent {
    SignalPayload::into_signal_event(SignalKind::Error, message, HistoryMessageSource::Blprnt, None)
  }

  /// Create an error signal from a structured error.
  /// The error is converted to ErrorEvent and included in the payload.
  pub fn error_from<E: Into<ErrorEvent>>(error: E) -> SignalEvent {
    let event: ErrorEvent = error.into();
    SignalPayload::into_signal_event(
      SignalKind::Error,
      event.message.clone(),
      HistoryMessageSource::Blprnt,
      Some(event),
    )
  }

  pub fn error_from_with_message<E: Into<ErrorEvent>>(message: String, error: E) -> SignalEvent {
    let event: ErrorEvent = error.into();
    SignalPayload::into_signal_event(SignalKind::Error, message, HistoryMessageSource::Blprnt, Some(event))
  }

  pub fn warning(message: String) -> SignalEvent {
    SignalPayload::into_signal_event(SignalKind::Warning, message, HistoryMessageSource::Blprnt, None)
  }

  pub fn info(message: String) -> SignalEvent {
    SignalPayload::into_signal_event(SignalKind::Info, message, HistoryMessageSource::Blprnt, None)
  }

  pub fn with_id(self, id: SurrealId) -> Self {
    Self { id: Some(id), ..self }
  }
}

impl serde::Serialize for SignalPayload {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    let mut state = serializer.serialize_struct("SignalPayload", 4)?;
    state.serialize_field("id", &self.id.as_ref().map(|id| id.0.to_sql()))?;
    state.serialize_field("message", &self.message)?;
    state.serialize_field("source", &self.source)?;
    state.serialize_field("error", &self.error)?;
    state.end()
  }
}
