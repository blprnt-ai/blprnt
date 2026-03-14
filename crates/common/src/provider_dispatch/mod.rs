use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast::Sender;

use crate::agent::ToolId;
use crate::errors::ProviderError;

#[derive(Debug)]
pub struct ProviderDispatch {
  pub tx: Sender<ProviderEvent>,
}

impl ProviderDispatch {
  pub fn new(tx: Sender<ProviderEvent>) -> Arc<Self> {
    Arc::new(Self { tx })
  }

  pub fn send(&self, event: ProviderEvent) -> Result<()> {
    let _ = self.tx.send(event);

    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ProviderEvent {
  Start(String),
  Stop(String),

  Ping,

  // Reasoning
  ReasoningStarted { rel_id: String },
  ReasoningDelta { rel_id: String, delta: String },
  Reasoning { rel_id: String, reasoning: String, signature: Option<String> },
  ReasoningDone { rel_id: String },

  // Response
  ResponseStarted { rel_id: String },
  ResponseDelta { rel_id: String, delta: String },
  Response { rel_id: String, content: String, signature: Option<String> },
  ResponseDone { rel_id: String },

  // Tool use
  ToolCall { tool_id: ToolId, tool_use_id: String, args: String, signature: Option<String> },

  // Misc
  Status(String),
  TokenUsage(u32),

  Error(ProviderError),
}
