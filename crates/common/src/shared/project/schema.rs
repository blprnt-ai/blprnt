use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

#[derive(Clone, Copy, Default, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum PlanningItemStatus {
  #[default]
  Pending,
  InProgress,
  Complete,
  #[schemars(skip)]
  Unknown,
}

impl FromStr for PlanningItemStatus {
  type Err = anyhow::Error;

  fn from_str(status: &str) -> anyhow::Result<Self> {
    match status {
      "pending" => Ok(PlanningItemStatus::Pending),
      "in_progress" => Ok(PlanningItemStatus::InProgress),
      "complete" => Ok(PlanningItemStatus::Complete),
      "completed" => Ok(PlanningItemStatus::Complete),
      _ => unreachable!(),
    }
  }
}

impl Display for PlanningItemStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PlanningItemStatus::Pending => write!(f, "pending"),
      PlanningItemStatus::InProgress => write!(f, "in_progress"),
      PlanningItemStatus::Complete => write!(f, "complete"),
      _ => unreachable!(),
    }
  }
}

impl PartialEq for PlanningItemStatus {
  fn eq(&self, other: &Self) -> bool {
    matches!(
      (self, other),
      (PlanningItemStatus::Pending, PlanningItemStatus::Pending)
        | (PlanningItemStatus::InProgress, PlanningItemStatus::InProgress)
        | (PlanningItemStatus::Complete, PlanningItemStatus::Complete)
    )
  }
}

impl Eq for PlanningItemStatus {}

impl Hash for PlanningItemStatus {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.to_string().hash(state);
  }
}

impl PartialOrd for PlanningItemStatus {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for PlanningItemStatus {
  fn cmp(&self, other: &Self) -> Ordering {
    if self == other {
      return Ordering::Equal;
    }

    match (self, other) {
      (PlanningItemStatus::Pending, PlanningItemStatus::InProgress) => Ordering::Less,
      (PlanningItemStatus::Pending, PlanningItemStatus::Complete) => Ordering::Less,

      (PlanningItemStatus::InProgress, PlanningItemStatus::Pending) => Ordering::Greater,
      (PlanningItemStatus::InProgress, PlanningItemStatus::Complete) => Ordering::Less,

      (PlanningItemStatus::Complete, PlanningItemStatus::Pending) => Ordering::Greater,
      (PlanningItemStatus::Complete, PlanningItemStatus::InProgress) => Ordering::Greater,
      _ => Ordering::Equal,
    }
  }
}
