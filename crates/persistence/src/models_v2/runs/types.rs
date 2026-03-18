use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use macros::SurrealEnumValue;
use surrealdb_types::SurrealValue;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
pub enum RunStatus {
  Pending,
  Running,
  Completed,
  Cancelled,
  Failed,
}

impl Display for RunStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RunStatus::Pending => write!(f, "pending"),
      RunStatus::Running => write!(f, "running"),
      RunStatus::Completed => write!(f, "completed"),
      RunStatus::Cancelled => write!(f, "cancelled"),
      RunStatus::Failed => write!(f, "failed"),
    }
  }
}

impl FromStr for RunStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "pending" => Ok(RunStatus::Pending),
      "running" => Ok(RunStatus::Running),
      "completed" => Ok(RunStatus::Completed),
      "cancelled" => Ok(RunStatus::Cancelled),
      "failed" => Ok(RunStatus::Failed),
      _ => Err(anyhow::anyhow!("Invalid employee run status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum RunTrigger {
  Manual,
  Timer,
  Event,
}

impl Display for RunTrigger {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RunTrigger::Manual => write!(f, "manual"),
      RunTrigger::Timer => write!(f, "timer"),
      RunTrigger::Event => write!(f, "event"),
    }
  }
}

impl FromStr for RunTrigger {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "manual" => Ok(RunTrigger::Manual),
      "timer" => Ok(RunTrigger::Timer),
      "event" => Ok(RunTrigger::Event),
      _ => Err(anyhow::anyhow!("Invalid employee run trigger: {}", s)),
    }
  }
}
