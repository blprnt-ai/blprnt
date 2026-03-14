pub mod control;
pub mod llm;
pub mod prompt;
pub mod signal;
pub mod slack;

use crate::session_dispatch::events::control::ControlEvent;
use crate::session_dispatch::events::llm::LlmEvent;
use crate::session_dispatch::events::prompt::PromptEvent;
use crate::session_dispatch::events::signal::SignalEvent;
use crate::session_dispatch::events::slack::SlackEvent;

#[derive(Clone, Debug, serde::Serialize, specta::Type)]
#[serde(tag = "eventType", rename_all = "camelCase")]
pub enum SessionDispatchEvent {
  Control(ControlEvent),
  Llm(LlmEvent),
  Signal(SignalEvent),
  Prompt(PromptEvent),
  Slack(SlackEvent),
}

impl From<ControlEvent> for SessionDispatchEvent {
  fn from(event: ControlEvent) -> Self {
    SessionDispatchEvent::Control(event)
  }
}

impl From<LlmEvent> for SessionDispatchEvent {
  fn from(event: LlmEvent) -> Self {
    SessionDispatchEvent::Llm(event)
  }
}

impl From<SignalEvent> for SessionDispatchEvent {
  fn from(event: SignalEvent) -> Self {
    SessionDispatchEvent::Signal(event)
  }
}

impl From<PromptEvent> for SessionDispatchEvent {
  fn from(event: PromptEvent) -> Self {
    SessionDispatchEvent::Prompt(event)
  }
}

impl From<SlackEvent> for SessionDispatchEvent {
  fn from(event: SlackEvent) -> Self {
    SessionDispatchEvent::Slack(event)
  }
}
