use std::fmt::Display;
use std::sync::OnceLock;

use tauri::AppHandle;
use tauri::Emitter;
use tauri::Wry;

use crate::blprnt_dispatch::SessionEvent;
use crate::errors::ErrorEvent;
use crate::shared::prelude::McpServerStatus;

static BLPRNT_HANDLE: OnceLock<AppHandle<Wry>> = OnceLock::new();

type BackendReady = ();

#[derive(Clone, Debug, serde::Serialize)]
pub struct ReportBugMenuClicked;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EmittedEvent {
  BackendReady(BackendReady),
  ReportBugMenuClicked(ReportBugMenuClicked),
  SessionEvent(Box<SessionEvent>),
  Error(ErrorEvent),
  TunnelMessage(TunnelMessage),
  McpServerStatus(McpServerStatus),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TunnelMessage {
  SlackOauthCallback,
  PaymentSuccess,
  Authentication(TunnelMessageAuthentication),
  Raw(TunnelMessageRaw),
  #[serde(other)]
  Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TunnelMessageAuthentication {
  pub token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct TunnelMessageRaw {
  pub raw: String,
}

impl From<TunnelMessage> for EmittedEvent {
  fn from(message: TunnelMessage) -> Self {
    EmittedEvent::TunnelMessage(message)
  }
}

impl From<SessionEvent> for EmittedEvent {
  fn from(event: SessionEvent) -> Self {
    EmittedEvent::SessionEvent(Box::new(event))
  }
}

impl From<ErrorEvent> for EmittedEvent {
  fn from(error: ErrorEvent) -> Self {
    EmittedEvent::Error(error)
  }
}

impl From<BackendReady> for EmittedEvent {
  fn from(event: BackendReady) -> Self {
    EmittedEvent::BackendReady(event)
  }
}

impl From<ReportBugMenuClicked> for EmittedEvent {
  fn from(event: ReportBugMenuClicked) -> Self {
    EmittedEvent::ReportBugMenuClicked(event)
  }
}

impl From<McpServerStatus> for EmittedEvent {
  fn from(event: McpServerStatus) -> Self {
    EmittedEvent::McpServerStatus(event)
  }
}

pub struct Blprnt;

impl Blprnt {
  pub fn init(handle: &AppHandle<Wry>) {
    tracing::debug!("Initializing Blprnt");
    let _ = BLPRNT_HANDLE.set(handle.clone());
  }

  pub fn handle() -> AppHandle<Wry> {
    BLPRNT_HANDLE.get().expect("AppHandle not initialized").clone()
  }

  pub fn emit(kind: BlprntEventKind, payload: EmittedEvent) {
    let _ = Self::handle().emit(&kind.to_string(), payload);
  }

  /// Emit a global error event to the frontend.
  /// Use this for errors that occur outside of a session context.
  pub fn emit_error<E: Into<ErrorEvent>>(error: E) {
    Self::emit(BlprntEventKind::Error, EmittedEvent::Error(error.into()));
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum BlprntEventKind {
  BackendReady,
  ReportBugMenuClicked,
  SessionEvent,
  Error,
  #[serde(rename = "oauthCallback")]
  OAuthCallback,
  TunnelMessage,
  McpServerStatus,
}

impl Display for BlprntEventKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::BackendReady => "backendReady",
        Self::ReportBugMenuClicked => "reportBugMenuClicked",
        Self::SessionEvent => "sessionEvent",
        Self::Error => "error",
        Self::OAuthCallback => "oauthCallback",
        Self::TunnelMessage => "tunnelMessage",
        Self::McpServerStatus => "mcpServerStatus",
      }
    )
  }
}
