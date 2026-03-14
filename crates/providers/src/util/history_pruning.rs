use std::collections::HashSet;

use anyhow::Result;
use common::api::LlmModelResponse;
use common::shared::prelude::MessageContent;
use common::shared::prelude::MessageToolResult;
use common::shared::prelude::MessageToolUse;
use common::tokenizer::Tokenizer;
use persistence::prelude::MessageRecord;

/// Only run heuristic pruning when estimated usage is at or above this fraction of context.
const PRUNE_THRESHOLD: f64 = 0.90;
/// Prune from the top until estimated usage is at or below this fraction of context (50% headroom).
const HEADROOM: f64 = 0.70;

/// Zone boundaries as fraction of context window (top 5%, middle 55%, bottom 30%).
const ZONE_TOP_END: f64 = 0.05;
const ZONE_MIDDLE_END: f64 = 0.50;

pub async fn prune_history(history: Vec<MessageRecord>, model: LlmModelResponse) -> Result<Vec<MessageRecord>> {
  if history.is_empty() {
    tracing::info!("history is empty");
    return Ok(history);
  }

  Ok(prune_heuristic(history, &model))
}

pub async fn apply_pruning<F, Fut>(
  history: Vec<MessageRecord>,
  model: LlmModelResponse,
  pruner: F,
) -> Vec<MessageRecord>
where
  F: Fn(Vec<MessageRecord>, LlmModelResponse) -> Fut,
  Fut: std::future::Future<Output = Result<Vec<MessageRecord>>>,
{
  let original = history.clone();
  match pruner(history, model).await {
    Ok(pruned) if !pruned.is_empty() => pruned,
    _ => original,
  }
}

fn find_tool_result(
  history: &[MessageRecord],
  from_index: usize,
  tool_use_id: &str,
  window_end: usize,
) -> Option<usize> {
  let limit = window_end.min(history.len().saturating_sub(1));
  for (index, message) in history.iter().enumerate().skip(from_index + 1) {
    if index > limit {
      break;
    }
    if let MessageContent::ToolResult(MessageToolResult { tool_use_id: id, .. }) = &message.content()
      && id == tool_use_id
    {
      return Some(index);
    }
  }

  None
}

fn find_tool_use(
  history: &[MessageRecord],
  from_index: usize,
  tool_use_id: &str,
  window_start: usize,
) -> Option<usize> {
  if history.is_empty() || from_index == 0 {
    return None;
  }

  let mut index = from_index.saturating_sub(1);
  loop {
    if index < window_start {
      break;
    }

    if let Some(message) = history.get(index)
      && let MessageContent::ToolUse(MessageToolUse { id, .. }) = &message.content()
      && id == tool_use_id
    {
      return Some(index);
    }

    if index == 0 {
      break;
    }

    index = index.saturating_sub(1);
  }

  None
}

fn estimate_message_tokens(msg: &MessageRecord) -> u32 {
  let s = serde_json::to_string(msg).unwrap_or_default();
  Tokenizer::count_string_tokens(&s)
}

/// Zone in the context window (by token position). Top = oldest 15%, Middle = next 55%, Bottom = newest 30%.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContextZone {
  Top,
  Middle,
  Bottom,
}

/// Token-based context pruning fallback when embeddings are unavailable.
///
/// Pruning order: (0) user image messages from the top first (no zones); then by zone (top 15%,
/// middle 55%, bottom 30% of context): (1) all messages in the middle zone, (2) assistant messages
/// in the top zone, (3) user messages in the middle zone, (4) assistant messages in the bottom
/// zone, until usage is at or below 50% of context. Only runs when usage is above the prune threshold (e.g. 80%).
fn prune_heuristic(history: Vec<MessageRecord>, model: &LlmModelResponse) -> Vec<MessageRecord> {
  // openrouter/auto picks the model at request time; we don't know context size, so don't prune.
  if model.slug == "openrouter/auto" {
    return history;
  }

  if history.is_empty() {
    return history;
  }

  let context_max = model.context_length.max(0) as u64;
  if context_max == 0 {
    return history;
  }

  let total_tokens: u64 = history.iter().map(estimate_message_tokens).map(u64::from).sum();
  let threshold_tokens = (PRUNE_THRESHOLD * context_max as f64) as u64;
  if total_tokens <= threshold_tokens {
    return history;
  }

  let target_tokens = ((1.0 - HEADROOM) * context_max as f64) as u64;
  let top_bound = (ZONE_TOP_END * context_max as f64) as u64;
  let middle_bound = (ZONE_MIDDLE_END * context_max as f64) as u64;

  // Assign each message to a zone by its start position in token space (cumulative from start of history).
  let mut cum = 0u64;
  let message_zones: Vec<(ContextZone, u64)> = history
    .iter()
    .map(|msg| {
      let tokens = u64::from(estimate_message_tokens(msg));
      let start = cum;
      cum += tokens;
      let zone = if start < top_bound {
        ContextZone::Top
      } else if start < middle_bound {
        ContextZone::Middle
      } else {
        ContextZone::Bottom
      };
      (zone, tokens)
    })
    .collect();

  let mut current_tokens = total_tokens;
  let mut dropped: HashSet<usize> = HashSet::new();

  let mut try_drop = |index: usize| {
    if current_tokens <= target_tokens || dropped.contains(&index) {
      return;
    }

    let tokens = message_zones[index].1;
    dropped.insert(index);
    current_tokens -= tokens;
  };

  // Phase 0: drop user image messages from the top first (no zones). Images are only in user content.
  for (index, msg) in history.iter().enumerate() {
    if msg.is_user() && matches!(msg.content(), MessageContent::Image64(_)) {
      try_drop(index);
    }
  }

  // Phase 1: drop all messages (assistant) in the middle zone.
  for (index, msg) in history.iter().enumerate() {
    if message_zones[index].0 == ContextZone::Middle && !msg.is_user() {
      try_drop(index);
    }
  }

  // Phase 2: if more headroom needed, drop assistant messages in the top zone.
  for (index, msg) in history.iter().enumerate() {
    if message_zones[index].0 == ContextZone::Top && !msg.is_user() {
      try_drop(index);
    }
  }

  // Phase 3: if more headroom needed, drop user messages in the middle zone.
  for (index, msg) in history.iter().enumerate() {
    if message_zones[index].0 == ContextZone::Middle && msg.is_user() {
      try_drop(index);
    }
  }

  // Phase 4: if more headroom needed, drop assistant messages in the bottom zone.
  for (index, msg) in history.iter().enumerate() {
    if message_zones[index].0 == ContextZone::Bottom && !msg.is_user() {
      try_drop(index);
    }
  }

  // Restore tool use/result pairs so we never leave an orphaned tool_use or tool_result.
  let len = history.len();
  let window_end = len.saturating_sub(1);
  let mut keep: HashSet<usize> = (0..len).filter(|i| !dropped.contains(i)).collect();
  for &index in keep.clone().iter() {
    if let Some(msg) = history.get(index) {
      match &msg.content() {
        MessageContent::ToolUse(MessageToolUse { id, .. }) => {
          if let Some(pair_index) = find_tool_result(&history, index, id, window_end) {
            keep.insert(pair_index);
          }
        }
        MessageContent::ToolResult(MessageToolResult { tool_use_id, .. }) => {
          if let Some(pair_index) = find_tool_use(&history, index, tool_use_id, 0) {
            keep.insert(pair_index);
          }
        }
        _ => {}
      }
    }
  }

  history.into_iter().enumerate().filter(|(i, _)| keep.contains(i)).map(|(_, m)| m).collect()
}
