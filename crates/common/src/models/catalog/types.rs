use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

use crate::models::LlmModel;
use crate::shared::prelude::Provider;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ContextLimit {
  pub native_limit:     u32,
  pub openrouter_limit: u32,
}

impl PartialEq for ContextLimit {
  fn eq(&self, other: &Self) -> bool {
    self.native_limit == other.native_limit && self.openrouter_limit == other.openrouter_limit
  }
}

impl Eq for ContextLimit {}

impl PartialOrd for ContextLimit {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.native_limit.cmp(&other.native_limit))
  }
}

impl Ord for ContextLimit {
  fn cmp(&self, other: &Self) -> Ordering {
    self.native_limit.cmp(&other.native_limit)
  }
}

#[derive(Clone, Default, PartialOrd, Ord, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ModelInfo {
  pub model:             LlmModel,
  pub provider:          Provider,
  pub display_name:      String,
  pub reasoning_efforts: Vec<ReasoningEffort>,
  pub is_free:           bool,
  pub output_limit:      ContextLimit,
  pub total_limit:       ContextLimit,
}

impl ModelInfo {
  pub fn supports_reasoning(&self) -> bool {
    !self.reasoning_efforts.is_empty()
  }
}

impl Debug for ModelInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Model: {}", self.display_name)
  }
}

impl PartialEq for ModelInfo {
  fn eq(&self, other: &Self) -> bool {
    self.model == other.model
  }
}

impl Hash for ModelInfo {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.model.hash(state);
  }
}

impl Eq for ModelInfo {}

#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  PartialEq,
  Eq,
  Hash,
  Ord,
  PartialOrd,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  fake::Dummy,
)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
  High,
  #[default]
  Medium,
  Low,
  Minimal,
  None,
}

impl ReasoningEffort {
  pub fn label(&self) -> String {
    match self {
      Self::High => "High",
      Self::Medium => "Medium",
      Self::Low => "Low",
      Self::Minimal => "Minimal",
      Self::None => "None",
    }
    .to_string()
  }
}

impl Display for ReasoningEffort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::High => write!(f, "high"),
      Self::Medium => write!(f, "medium"),
      Self::Low => write!(f, "low"),
      Self::Minimal => write!(f, "minimal"),
      Self::None => write!(f, "none"),
    }
  }
}

impl FromStr for ReasoningEffort {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    match s {
      "high" => Ok(ReasoningEffort::High),
      "medium" => Ok(ReasoningEffort::Medium),
      "low" => Ok(ReasoningEffort::Low),
      "minimal" => Ok(ReasoningEffort::Minimal),
      "none" => Ok(ReasoningEffort::None),
      _ => unreachable!(),
    }
  }
}

pub static ALL_REASONING_EFFORTS: [ReasoningEffort; 4] =
  [ReasoningEffort::High, ReasoningEffort::Medium, ReasoningEffort::Low, ReasoningEffort::Minimal];
