use crate::session_dispatch::events::SessionDispatchEvent;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SlackInput {
  pub session_id:       String,
  pub text:             String,
  pub slack_user_id:    String,
  pub slack_channel_id: String,
  pub thread_ts:        Option<String>,
  pub message_ts:       Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SlackEvent {
  Input(SlackInput),
}

impl From<SlackInput> for SessionDispatchEvent {
  fn from(value: SlackInput) -> Self {
    SessionDispatchEvent::Slack(SlackEvent::Input(value))
  }
}
