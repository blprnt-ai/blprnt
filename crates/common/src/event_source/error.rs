use std::string::FromUtf8Error;

use eventsource_stream::EventStreamError;
use nom::error::Error as NomError;
use reqwest::Error as ReqwestError;
use reqwest::Response;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use reqwest_middleware::Error as ReqwestMiddlewareError;

use crate::errors::ProviderError;

#[derive(Debug, thiserror::Error)]
pub enum EventSourceError {
  #[error(transparent)]
  Utf8(FromUtf8Error),
  #[error(transparent)]
  Parser(NomError<String>),
  #[error(transparent)]
  Transport(ReqwestError),
  #[error(transparent)]
  MiddlewareTransport(ReqwestMiddlewareError),
  #[error("Invalid header value: {0:?}")]
  InvalidContentType(HeaderValue, Response),
  #[error("Invalid status code: {0}")]
  InvalidStatusCode(StatusCode, String, String),
  #[error("Stream ended")]
  StreamEnded,
}

impl From<EventStreamError<ReqwestMiddlewareError>> for EventSourceError {
  fn from(err: EventStreamError<ReqwestMiddlewareError>) -> Self {
    match err {
      EventStreamError::Utf8(err) => Self::Utf8(err),
      EventStreamError::Parser(err) => Self::Parser(err),
      EventStreamError::Transport(err) => Self::MiddlewareTransport(err),
    }
  }
}

impl From<EventStreamError<ReqwestError>> for EventSourceError {
  fn from(err: EventStreamError<ReqwestError>) -> Self {
    match err {
      EventStreamError::Utf8(err) => Self::Utf8(err),
      EventStreamError::Parser(err) => Self::Parser(err),
      EventStreamError::Transport(err) => Self::Transport(err),
    }
  }
}

impl From<EventSourceError> for ProviderError {
  fn from(err: EventSourceError) -> Self {
    match err {
      EventSourceError::Utf8(err) => ProviderError::Utf8(err.to_string()),
      EventSourceError::Parser(err) => ProviderError::Parser(err.to_string()),
      EventSourceError::Transport(err) => ProviderError::Transport(err.to_string()),
      EventSourceError::MiddlewareTransport(err) => ProviderError::MiddlewareTransport(err.to_string()),
      EventSourceError::InvalidContentType(_, response) => {
        ProviderError::InvalidContentType(response.status().to_string())
      }
      EventSourceError::InvalidStatusCode(code, status_text, body) => {
        ProviderError::InvalidStatusCode { code: code.to_string(), status_text, body }
      }
      EventSourceError::StreamEnded => ProviderError::StreamEnded,
    }
  }
}
