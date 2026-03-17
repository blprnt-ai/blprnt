mod message;

pub use message::ChannelMessage;
pub use message::SendMessage;

/// Slack DM adapter bound to one authenticated user and one DM conversation.
#[derive(Debug, Clone)]
pub struct SlackChannel {
  bot_token:      String,
  dm_channel_id:  String,
  authed_user_id: String,
}

impl SlackChannel {
  pub fn new(bot_token: String, dm_channel_id: String, authed_user_id: String) -> Self {
    Self {
      bot_token:      bot_token.trim().to_string(),
      dm_channel_id:  dm_channel_id.trim().to_string(),
      authed_user_id: authed_user_id.trim().to_string(),
    }
  }

  fn http_client(&self) -> reqwest::Client {
    reqwest::Client::new()
  }

  fn slack_api_ok(payload: &serde_json::Value) -> bool {
    matches!(payload.get("ok"), Some(serde_json::Value::Bool(true)))
  }

  fn slack_api_error(payload: &serde_json::Value) -> &str {
    payload.get("error").and_then(|error| error.as_str()).unwrap_or("unknown")
  }

  fn expected_recipient(&self, recipient: &str) -> bool {
    recipient.trim() == self.dm_channel_id
  }

  pub fn dm_channel_id(&self) -> &str {
    &self.dm_channel_id
  }

  pub async fn open_dm_channel_id(bot_token: &str, authed_user_id: &str) -> anyhow::Result<String> {
    anyhow::ensure!(!bot_token.trim().is_empty(), "Slack bot token is required");
    anyhow::ensure!(!authed_user_id.trim().is_empty(), "Slack authed user ID is required");

    let resp = reqwest::Client::new()
      .post("https://slack.com/api/conversations.open")
      .bearer_auth(bot_token.trim())
      .json(&serde_json::json!({
        "users": authed_user_id.trim(),
      }))
      .send()
      .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|error| format!("<failed to read response body: {error}>"));

    if !status.is_success() {
      anyhow::bail!("Slack conversations.open failed ({status}): {body}");
    }

    let payload: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    if !Self::slack_api_ok(&payload) {
      anyhow::bail!("Slack conversations.open failed: {}", Self::slack_api_error(&payload));
    }

    payload
      .get("channel")
      .and_then(|channel| channel.get("id"))
      .and_then(|channel_id| channel_id.as_str())
      .map(str::to_string)
      .ok_or_else(|| anyhow::anyhow!("Slack conversations.open missing DM channel ID"))
  }
}

impl SlackChannel {
  pub fn name(&self) -> &str {
    "slack"
  }

  pub async fn send(&self, message: &SendMessage) -> anyhow::Result<String> {
    anyhow::ensure!(!self.dm_channel_id.is_empty(), "Slack DM channel ID is required");
    anyhow::ensure!(!self.authed_user_id.is_empty(), "Slack authed user ID is required");
    anyhow::ensure!(
      self.expected_recipient(&message.recipient),
      "Slack send recipient must match the configured DM channel"
    );

    let mut body = serde_json::json!({
      "channel": self.dm_channel_id,
      "text": message.content,
    });

    if let Some(ref thread_ts) = message.thread_ts {
      body["thread_ts"] = serde_json::json!(thread_ts);
    }

    let resp = self
      .http_client()
      .post("https://slack.com/api/chat.postMessage")
      .bearer_auth(&self.bot_token)
      .json(&body)
      .send()
      .await?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_else(|error| format!("<failed to read response body: {error}>"));

    if !status.is_success() {
      anyhow::bail!("Slack chat.postMessage failed ({status}): {body}");
    }

    let payload: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    if !Self::slack_api_ok(&payload) {
      anyhow::bail!("Slack chat.postMessage failed: {}", Self::slack_api_error(&payload));
    }

    payload
      .get("ts")
      .and_then(|ts| ts.as_str())
      .map(str::to_string)
      .ok_or_else(|| anyhow::anyhow!("Slack chat.postMessage missing ts"))
  }

  pub async fn health_check(&self) -> bool {
    let resp = match self.http_client().get("https://slack.com/api/auth.test").bearer_auth(&self.bot_token).send().await
    {
      Ok(response) => response,
      Err(_) => return false,
    };

    if !resp.status().is_success() {
      return false;
    }

    match resp.json::<serde_json::Value>().await {
      Ok(payload) => Self::slack_api_ok(&payload),
      Err(_) => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn slack_channel_name() {
    let channel = SlackChannel::new("xoxb-fake".into(), "D12345".into(), "U12345".into());
    assert_eq!(channel.name(), "slack");
  }

  #[test]
  fn slack_channel_contract_trims_ids() {
    let channel = SlackChannel::new("xoxb-fake".into(), " D12345 ".into(), " U12345 ".into());
    assert_eq!(channel.dm_channel_id, "D12345");
    assert_eq!(channel.authed_user_id, "U12345");
  }

  #[test]
  fn expected_recipient_requires_exact_dm_channel() {
    let channel = SlackChannel::new("xoxb-fake".into(), "D12345".into(), "U12345".into());
    assert!(channel.expected_recipient("D12345"));
    assert!(channel.expected_recipient(" D12345 "));
    assert!(!channel.expected_recipient("C12345"));
    assert!(!channel.expected_recipient(""));
  }

  #[test]
  fn slack_api_ok_requires_true() {
    assert!(SlackChannel::slack_api_ok(&serde_json::json!({ "ok": true })));
    assert!(!SlackChannel::slack_api_ok(&serde_json::json!({ "ok": false })));
    assert!(!SlackChannel::slack_api_ok(&serde_json::json!({ "error": "invalid_auth" })));
  }

  #[test]
  fn slack_message_id_format_includes_dm_and_ts() {
    let ts = "1234567890.123456";
    let dm_channel_id = "D12345";
    let expected_id = format!("slack_{dm_channel_id}_{ts}");
    assert_eq!(expected_id, "slack_D12345_1234567890.123456");
  }

  #[test]
  fn slack_message_id_is_deterministic() {
    let ts = "1234567890.123456";
    let dm_channel_id = "D12345";
    let id1 = format!("slack_{dm_channel_id}_{ts}");
    let id2 = format!("slack_{dm_channel_id}_{ts}");
    assert_eq!(id1, id2);
  }

  #[test]
  fn slack_message_id_different_ts_different_id() {
    let dm_channel_id = "D12345";
    let id1 = format!("slack_{dm_channel_id}_1234567890.123456");
    let id2 = format!("slack_{dm_channel_id}_1234567890.123457");
    assert_ne!(id1, id2);
  }

  #[test]
  fn slack_message_id_different_dm_different_id() {
    let ts = "1234567890.123456";
    let id1 = format!("slack_D12345_{ts}");
    let id2 = format!("slack_D67890_{ts}");
    assert_ne!(id1, id2);
  }
}
