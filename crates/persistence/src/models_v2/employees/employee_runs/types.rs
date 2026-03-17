use std::fmt::Display;
use std::str::FromStr;

use anyhow::Result;
use macros::SurrealEnumValue;
use surrealdb_types::SurrealValue;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
pub enum EmployeeRunStatus {
  Pending,
  Running,
  Completed,
  Cancelled,
  Failed,
}

impl Display for EmployeeRunStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeRunStatus::Pending => write!(f, "pending"),
      EmployeeRunStatus::Running => write!(f, "running"),
      EmployeeRunStatus::Completed => write!(f, "completed"),
      EmployeeRunStatus::Cancelled => write!(f, "cancelled"),
      EmployeeRunStatus::Failed => write!(f, "failed"),
    }
  }
}

impl FromStr for EmployeeRunStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "pending" => Ok(EmployeeRunStatus::Pending),
      "running" => Ok(EmployeeRunStatus::Running),
      "completed" => Ok(EmployeeRunStatus::Completed),
      "cancelled" => Ok(EmployeeRunStatus::Cancelled),
      "failed" => Ok(EmployeeRunStatus::Failed),
      _ => Err(anyhow::anyhow!("Invalid employee run status: {}", s)),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub enum EmployeeRunTurnTrigger {
  Manual,
  Timer,
  Event,
}

impl Display for EmployeeRunTurnTrigger {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      EmployeeRunTurnTrigger::Manual => write!(f, "manual"),
      EmployeeRunTurnTrigger::Timer => write!(f, "timer"),
      EmployeeRunTurnTrigger::Event => write!(f, "event"),
    }
  }
}

impl FromStr for EmployeeRunTurnTrigger {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "manual" => Ok(EmployeeRunTurnTrigger::Manual),
      "timer" => Ok(EmployeeRunTurnTrigger::Timer),
      "event" => Ok(EmployeeRunTurnTrigger::Event),
      _ => Err(anyhow::anyhow!("Invalid employee run turn trigger: {}", s)),
    }
  }
}
