use surrealdb::types::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PromptEvent {
  Started(PromptStarted),
  Queued(PromptQueued),
  Deleted(PromptDeleted),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PromptStarted {
  pub id:            String,
  #[specta(type = String)]
  pub turn_id:       Uuid,
  pub prompt:        String,
  #[serde(rename = "queue_item_id")]
  pub queue_item_id: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PromptQueued {
  pub id:            String,
  #[serde(rename = "queue_item_id")]
  pub queue_item_id: String,
  pub prompt:        Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PromptDeleted {
  #[serde(rename = "queue_item_id")]
  pub queue_item_id: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn queued_event_serializes_queue_item_id() {
    let event = PromptEvent::Queued(PromptQueued {
      id:            "queued-id".to_string(),
      queue_item_id: "queue-id".to_string(),
      prompt:        Some("hello".to_string()),
    });

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value.get("queue_item_id").and_then(|value| value.as_str()), Some("queue-id"));
  }

  #[test]
  fn started_event_serializes_queue_item_id() {
    let event = PromptEvent::Started(PromptStarted {
      id:            "history-id".to_string(),
      turn_id:       Uuid::new_v7(),
      prompt:        "hello".to_string(),
      queue_item_id: Some("queue-id".to_string()),
    });

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value.get("queue_item_id").and_then(|value| value.as_str()), Some("queue-id"));
  }

  #[test]
  fn deleted_event_serializes_queue_item_id() {
    let event = PromptEvent::Deleted(PromptDeleted { queue_item_id: "queue-id".to_string() });

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value.get("queue_item_id").and_then(|value| value.as_str()), Some("queue-id"));
  }
}
