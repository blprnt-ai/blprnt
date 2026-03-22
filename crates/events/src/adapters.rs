use persistence::prelude::RunId;
use shared::agent::ToolId;
use shared::tools::ToolUseResponse;

use crate::bus::Events;

#[derive(Clone, Debug)]
pub enum AdapterEvent {
  // Life cycle
  RunStarted { run_id: RunId },
  RunCompleted { run_id: RunId },
  RunFailed { run_id: RunId, error: String },

  // Ouput
  Response { run_id: RunId, response: String },
  Thinking { run_id: RunId, thinking: String },
  ToolDone { run_id: RunId, tool_id: ToolId, result: ToolUseResponse },
}

lazy_static::lazy_static! {
  pub static ref ADAPTER_EVENTS: Events<AdapterEvent> = Events::new();
}
