pub type SseItem = serde_json::Value;
#[derive(Clone, Debug, Default)]
pub struct SseDecoder {
  event_name: Option<String>,
  saw_data:   bool,
  buf:        String,
}

impl SseDecoder {
  pub fn push_line(&mut self, line: &str) -> Option<SseItem> {
    if line.trim().is_empty() {
      if self.buf.is_empty() {
        return None;
      }

      let data = std::mem::take(&mut self.buf);
      self.event_name = None;
      self.saw_data = false;

      let trimmed = data.trim();

      if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Some(value);
      } else {
        tracing::error!("Failed to parse JSON: {}", trimmed);
      }

      return None;
    }

    if let Some(rest) = line.strip_prefix("event:") {
      self.event_name = Some(rest.trim().to_string());
      return None;
    }

    if let Some(rest) = line.strip_prefix("data:") {
      self.saw_data = true;
      self.buf.push_str(rest.trim_start());
      return None;
    }

    if self.saw_data {
      self.buf.push_str(line);
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decodes_anthropic_style_blocks_with_split_json() {
    let mut d = SseDecoder::default();
    let input = [
      "event: message_start",
      "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_01\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude\",\"content\":[],\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":125,\"cache_creation_inp",
      "ut_tokens\":0,\"cache_read_input_tokens\":0,\"cache_creation\":{\"ephemeral_5m_input_tokens\":0,\"ephemeral_1h_input_tokens\":0},\"output_tokens\":4,\"service_tier\":\"standard\"}}           }",
      "",
      "",
      "event: content_block_start",
      "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}      }",
      "",
      "",
      "event: content_block_delta",
      "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"boop\"}           }",
      "",
      "",
      "event: content_block_stop",
      "data: {\"type\":\"content_block_stop\",\"index\":0 }",
      "",
      "",
      "event: message_delta",
      "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":125,\"cache_creation_input_tokens\":0,\"cache_read_input_tokens\":0,\"output_tokens\":5} }",
      "",
      "event: message_stop",
      "data: {\"type\":\"message_stop\"  }",
      "",
      "",
    ];

    let mut types = Vec::new();
    for line in input {
      if let Some(item) = d.push_line(line) {
        types.push(item.get("type").and_then(|t| t.as_str()).unwrap().to_string());
      }
    }

    assert_eq!(
      types,
      vec![
        "message_start",
        "content_block_start",
        "content_block_delta",
        "content_block_stop",
        "message_delta",
        "message_stop",
      ]
    );
  }
}
