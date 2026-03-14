use serde_json::Value;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
  Ping,
  MessageStart { message: MessageStart },
  ContentBlockStart { index: u32, content_block: AnthropicContentBlock },
  ContentBlockDelta { index: u32, delta: Delta },
  ContentBlockStop { index: u32 },
  MessageDelta { usage: MessageUsage },
  MessageStop,
  Error { error: AnthropicError },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageStart {
  pub id: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
  Text { text: String },
  Thinking { thinking: String, signature: Option<String> },
  ToolUse { id: String, name: String, input: Value },
  RedactedThinking { data: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
  TextDelta { text: String },
  InputJsonDelta { partial_json: String },
  ThinkingDelta { thinking: String },
  SignatureDelta { signature: String },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageUsage {
  pub input_tokens:                u32,
  pub cache_creation_input_tokens: Option<u32>,
  pub output_tokens:               u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicError {
  pub details: Option<String>,
  #[serde(rename = "type")]
  pub kind:    String,
  pub message: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn text_parse_content_block_start() {
    let event_json = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
    let event = serde_json::from_str::<AnthropicStreamEvent>(event_json);
    assert!(event.is_ok());
  }

  #[test]
  fn text_parse_content_block_delta() {
    let event_json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"I'll rea"}}"#;
    let event = serde_json::from_str::<AnthropicStreamEvent>(event_json);
    assert!(event.is_ok());
  }

  #[test]
  fn text_parse_content_block_stop() {
    let event_json = r#"{"type":"content_block_stop","index":0}"#;
    let event = serde_json::from_str::<AnthropicStreamEvent>(event_json);
    assert!(event.is_ok());
  }

  #[test]
  fn text_parse_message_stop() {
    let event_json = r#"{"type":"message_stop"}"#;
    let event = serde_json::from_str::<AnthropicStreamEvent>(event_json);
    assert!(event.is_ok());
  }
}
