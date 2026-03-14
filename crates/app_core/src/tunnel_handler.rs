use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use common::blprnt::Blprnt;
use common::blprnt::BlprntEventKind;
use common::blprnt::TunnelMessage;
use common::blprnt::TunnelMessageRaw;
use common::slack::ChannelMessage;
use serde::Deserialize;
use serde_json::Value;
use tunnel_client::TunnelRequest;
use tunnel_client::TunnelResponse;

use crate::EngineManager;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SlackRelayPayload {
  EventCallback(SlackRelayEventCallback),
  Message(SlackRelayMessage),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct SlackRelayEventCallback {
  #[allow(dead_code)]
  #[serde(rename = "type")]
  kind: String,
  #[allow(dead_code)]
  event: SlackRelayMessage,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct SlackRelayMessage {
  #[allow(dead_code)]
  #[serde(rename = "type")]
  kind: String,
  #[allow(dead_code)]
  subtype: Option<String>,
  #[allow(dead_code)]
  text: Option<String>,
  #[allow(dead_code)]
  user: Option<String>,
  #[allow(dead_code)]
  channel: Option<String>,
  #[allow(dead_code)]
  ts: Option<String>,
  #[allow(dead_code)]
  thread_ts: Option<String>,
}

fn normalize_slack_relay_payload(payload: SlackRelayPayload) -> Option<ChannelMessage> {
  let message = match payload {
    SlackRelayPayload::EventCallback(callback) => {
      if callback.kind != "event_callback" {
        return None;
      }
      callback.event
    }
    SlackRelayPayload::Message(message) => message,
  };

  normalize_slack_relay_message(message)
}

fn normalize_slack_relay_message(message: SlackRelayMessage) -> Option<ChannelMessage> {
  if message.kind != "message" || message.subtype.is_some() {
    return None;
  }

  let text = message.text?.trim().to_string();
  let user = message.user?.trim().to_string();
  let channel = message.channel?.trim().to_string();
  let ts = message.ts?.trim().to_string();

  if text.is_empty() || user.is_empty() || channel.is_empty() || ts.is_empty() {
    return None;
  }

  let thread_ts = message.thread_ts.as_deref().map(str::trim).filter(|thread_ts| !thread_ts.is_empty()).map(str::to_string);

  Some(ChannelMessage {
    id:           format!("slack_{}_{}", channel, ts),
    sender:       user,
    reply_target: channel,
    content:      text,
    channel:      "slack".to_string(),
    timestamp:    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
    thread_ts:    thread_ts.or_else(|| Some(ts.to_string())),
  })
}

fn is_slack_oauth_request(request: &TunnelRequest) -> bool {
  request.method.eq_ignore_ascii_case("POST") && request.path == "webhook/slack/oauth"
}

fn is_slack_relay_request(request: &TunnelRequest) -> bool {
  request.method.eq_ignore_ascii_case("POST") && matches!(request.path.as_str(), "webhook/slack/event" | "webhook/slack/events")
}

pub fn handle_tunnel_request(manager: Arc<EngineManager>, request: TunnelRequest) -> TunnelResponse {
  // Do not log request bodies (may contain secrets/tokens).
  tracing::info!(method = %request.method, path = %request.path, "Tunnel request");

  if let Some(response) = maybe_handle_slack(manager, &request) {
    return response;
  }

  let raw = serde_json::from_slice::<Value>(&request.body);

  let body = serde_json::from_slice::<TunnelMessage>(&request.body);

  match body {
    Ok(message) => Blprnt::emit(BlprntEventKind::TunnelMessage, message.into()),
    _ => {
      Blprnt::emit(
        BlprntEventKind::TunnelMessage,
        TunnelMessage::Raw(TunnelMessageRaw { raw: raw.unwrap_or_default().to_string() }).into(),
      );
    }
  }

  TunnelResponse { id: request.id, status: 200, headers: vec![], body: vec![] }
}

pub fn maybe_handle_slack(manager: Arc<EngineManager>, request: &TunnelRequest) -> Option<TunnelResponse> {
  if is_slack_oauth_request(request) {
    tracing::info!("Slack OAuth callback received");
    match serde_json::from_slice::<Value>(&request.body) {
      Ok(payload) => {
        handle_slack_oauth_json(&manager, payload);
        Blprnt::emit(BlprntEventKind::TunnelMessage, TunnelMessage::SlackOauthCallback.into());
        Some(TunnelResponse { id: request.id, status: 200, headers: vec![], body: vec![] })
      }
      Err(err) => {
        tracing::warn!("Slack OAuth payload not valid JSON: {err}");
        Some(TunnelResponse { id: request.id, status: 200, headers: vec![], body: vec![] })
      }
    }
  } else if is_slack_relay_request(request) {
    tracing::info!(path = %request.path, "Slack inbound relay payload received");

    match serde_json::from_slice::<SlackRelayPayload>(&request.body) {
      Ok(payload) => {
        if let Some(normalized) = normalize_slack_relay_payload(payload) {
          let slack = manager.slack.clone();
          tokio::spawn(async move {
            slack.handle_inbound_message(normalized).await;
          });
        } else {
          tracing::debug!(path = %request.path, "Ignoring unsupported normalized Slack inbound relay payload");
        }
        let raw = serde_json::from_slice::<Value>(&request.body).unwrap_or_default().to_string();
        Blprnt::emit(
          BlprntEventKind::TunnelMessage,
          TunnelMessage::Raw(TunnelMessageRaw { raw }).into(),
        );
      }
      Err(err) => {
        tracing::warn!(path = %request.path, "Slack inbound relay payload failed typed deserialization: {err}");
      }
    }

    Some(TunnelResponse { id: request.id, status: 200, headers: vec![], body: vec![] })
  } else {
    None
  }
}

fn handle_slack_oauth_json(manager: &Arc<EngineManager>, payload: Value) {
  // Forward-compat: accept both success JSON from Slack oauth.v2.access and relay-generated error JSON.
  let ok = payload.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

  if !ok {
    let err = payload
      .get("error_description")
      .and_then(|v| v.as_str())
      .or_else(|| payload.get("error").and_then(|v| v.as_str()))
      .unwrap_or("slack_oauth_failed")
      .to_string();

    if let Err(e) = manager.slack.set_status(false, Some(err.clone())) {
      tracing::warn!("Failed to persist Slack error status: {e}");
    }

    if let Err(e) = manager.slack.set_oauth_state(None) {
      tracing::warn!("Failed to clear Slack OAuth state after error: {e}");
    }

    return;
  }

  // Minimal persistence: mark connected. Token + team info stored securely.
  if let Err(e) = manager.slack.persist_oauth_success(&payload) {
    tracing::warn!("Failed to persist Slack OAuth success: {e}");
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_request(path: &str, body: Value) -> TunnelRequest {
    TunnelRequest {
      id: uuid::Uuid::new_v4(),
      method: "POST".to_string(),
      path: path.to_string(),
      headers: vec![],
      body: serde_json::to_vec(&body).expect("request body should serialize"),
    }
  }

  #[test]
  fn normalizes_supported_message_payload() {
    let payload = SlackRelayPayload::Message(SlackRelayMessage {
      kind: "message".to_string(),
      subtype: None,
      text: Some(" hello from slack ".to_string()),
      user: Some(" U123 ".to_string()),
      channel: Some(" C123 ".to_string()),
      ts: Some(" 1712345.678 ".to_string()),
      thread_ts: Some(" 1712000.000 ".to_string()),
    });

    let normalized = normalize_slack_relay_payload(payload).expect("message payload should normalize");

    assert_eq!(normalized.id, "slack_C123_1712345.678");
    assert_eq!(normalized.sender, "U123");
    assert_eq!(normalized.reply_target, "C123");
    assert_eq!(normalized.content, "hello from slack");
    assert_eq!(normalized.channel, "slack");
    assert_eq!(normalized.thread_ts.as_deref(), Some("1712000.000"));
  }

  #[test]
  fn normalizes_event_callback_payload_and_defaults_thread_ts_to_message_ts() {
    let payload = SlackRelayPayload::EventCallback(SlackRelayEventCallback {
      kind: "event_callback".to_string(),
      event: SlackRelayMessage {
        kind: "message".to_string(),
        subtype: None,
        text: Some("reply".to_string()),
        user: Some("U999".to_string()),
        channel: Some("D999".to_string()),
        ts: Some("1712999.0001".to_string()),
        thread_ts: None,
      },
    });

    let normalized = normalize_slack_relay_payload(payload).expect("event callback should normalize");

    assert_eq!(normalized.thread_ts.as_deref(), Some("1712999.0001"));
    assert_eq!(normalized.content, "reply");
  }

  #[test]
  fn rejects_unsupported_payload_shapes_during_normalization() {
    let subtype_payload = SlackRelayPayload::Message(SlackRelayMessage {
      kind: "message".to_string(),
      subtype: Some("bot_message".to_string()),
      text: Some("ignored".to_string()),
      user: Some("U123".to_string()),
      channel: Some("C123".to_string()),
      ts: Some("1712345.678".to_string()),
      thread_ts: Some("1712000.000".to_string()),
    });
    let unsupported_callback = SlackRelayPayload::EventCallback(SlackRelayEventCallback {
      kind: "url_verification".to_string(),
      event: SlackRelayMessage {
        kind: "message".to_string(),
        subtype: None,
        text: Some("ignored".to_string()),
        user: Some("U123".to_string()),
        channel: Some("C123".to_string()),
        ts: Some("1712345.678".to_string()),
        thread_ts: None,
      },
    });
    let missing_required_fields = SlackRelayPayload::Message(SlackRelayMessage {
      kind: "message".to_string(),
      subtype: None,
      text: Some("   ".to_string()),
      user: Some("U123".to_string()),
      channel: Some("C123".to_string()),
      ts: Some("1712345.678".to_string()),
      thread_ts: None,
    });

    assert!(normalize_slack_relay_payload(subtype_payload).is_none());
    assert!(normalize_slack_relay_payload(unsupported_callback).is_none());
    assert!(normalize_slack_relay_payload(missing_required_fields).is_none());
  }

  #[test]
  fn preserves_slack_oauth_callback_route_matching() {
    let oauth_request = test_request("webhook/slack/oauth", serde_json::json!({ "ok": true }));
    let other_request = test_request("webhook/slack/event", serde_json::json!({ "ok": true }));

    assert!(is_slack_oauth_request(&oauth_request));
    assert!(!is_slack_oauth_request(&other_request));
  }

  #[test]
  fn matches_relay_listener_paths_and_rejects_other_paths() {
    let direct_event = test_request("webhook/slack/event", serde_json::json!({}));
    let plural_events = test_request("webhook/slack/events", serde_json::json!({}));
    let other = test_request("webhook/slack/oauth", serde_json::json!({}));

    assert!(is_slack_relay_request(&direct_event));
    assert!(is_slack_relay_request(&plural_events));
    assert!(!is_slack_relay_request(&other));
  }

  #[test]
  fn relay_payload_deserialization_supports_supported_and_unsupported_shapes() {
    let supported = serde_json::from_value::<SlackRelayPayload>(serde_json::json!({
      "type": "event_callback",
      "event": {
        "type": "message",
        "text": "thread reply",
        "user": "U123",
        "channel": "D123",
        "ts": "1712345.678",
        "thread_ts": "1712000.000"
      }
    }))
    .expect("supported relay payload should deserialize");
    let unsupported = serde_json::from_value::<SlackRelayPayload>(serde_json::json!({
      "type": "message",
      "subtype": "bot_message",
      "text": "ignored",
      "user": "U123",
      "channel": "D123",
      "ts": "1712345.678"
    }))
    .expect("unsupported relay payload shape should still deserialize for filtering");

    assert!(normalize_slack_relay_payload(supported).is_some());
    assert!(normalize_slack_relay_payload(unsupported).is_none());
  }
}
