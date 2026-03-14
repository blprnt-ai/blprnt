use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use surrealdb::types::Uuid;

use crate::shared::message::MessageContent;
use crate::shared::message::MessageImage64;
use crate::shared::message::MessageText;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum QueueItemKind {
  Text(String),
  Image64(String),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct DeleteQueuedPromptRequest {
  pub session_id:    String,
  pub queue_item_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum DeleteQueuedPromptOutcome {
  Deleted,
  AlreadyStarted,
  NotFound,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct QueueItem {
  pub queue_item_id: String,
  items:             Vec<QueueItemKind>,
}

impl FromStr for QueueItem {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    Ok(Self { queue_item_id: Uuid::new_v7().to_string(), items: vec![QueueItemKind::Text(s.to_string())] })
  }
}

impl QueueItemKind {
  pub fn from_image_url(url: String) -> anyhow::Result<Self> {
    Ok(Self::Image64(url))
  }
}

impl From<QueueItemKind> for QueueItem {
  fn from(kind: QueueItemKind) -> Self {
    QueueItem { queue_item_id: Uuid::new_v7().to_string(), items: vec![kind] }
  }
}

impl From<Vec<QueueItemKind>> for QueueItem {
  fn from(kinds: Vec<QueueItemKind>) -> Self {
    QueueItem { queue_item_id: Uuid::new_v7().to_string(), items: kinds }
  }
}

impl From<QueueItem> for Vec<MessageContent> {
  fn from(item: QueueItem) -> Self {
    item
      .items
      .iter()
      .filter_map(|k| match k {
        QueueItemKind::Text(text) => {
          if text.is_empty() {
            None
          } else {
            Some(MessageText { text: text.clone(), signature: None }.into())
          }
        }
        QueueItemKind::Image64(image64) => Some(MessageImage64::from_str(&image64.clone()).unwrap_or_default().into()),
      })
      .collect()
  }
}

impl QueueItem {
  pub fn queue_item_id(&self) -> &str {
    &self.queue_item_id
  }

  pub fn display(&self) -> String {
    self.items.iter().fold(String::new(), |mut acc, c| {
      if let QueueItemKind::Text(text) = c {
        acc.push_str(text);
      }

      acc
    })
  }

  pub fn push_image_url(&mut self, url: String) -> anyhow::Result<()> {
    self.items.push(QueueItemKind::from_image_url(url)?);

    Ok(())
  }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PauseReason {
  #[default]
  None,
  SoftEditingHead,
  HardErrorNeedsEdit,
  UserInterrupted,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
#[serde(rename_all = "snake_case")]
pub enum QueueMode {
  Queue,
  Inject,
}

impl QueueMode {
  pub fn label(&self) -> String {
    match self {
      QueueMode::Queue => "Queue".to_string(),
      QueueMode::Inject => "Inject".to_string(),
    }
  }
}

impl FromStr for QueueMode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    Ok(match s {
      "queue" => Self::Queue,
      "inject" => Self::Inject,
      _ => unreachable!(),
    })
  }
}

impl Display for QueueMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Queue => "queue",
        Self::Inject => "inject",
      }
    )
  }
}
