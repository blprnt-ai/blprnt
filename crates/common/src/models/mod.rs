use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;

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
  SurrealEnumValue,
)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
  None,
  Minimal,
  Low,
  #[default]
  Medium,
  High,
  #[serde(rename = "xhigh")]
  XHigh,
}

impl ReasoningEffort {
  pub fn label(&self) -> String {
    match self {
      Self::XHigh => "XHigh",
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
      Self::XHigh => write!(f, "xhigh"),
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
      "xhigh" => Ok(ReasoningEffort::XHigh),
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
