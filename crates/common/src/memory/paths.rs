use std::path::PathBuf;

use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDate;
use schemars::JsonSchema;

use super::contracts::MemoryContract;
use super::contracts::MemorySummaryContract;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct MemoryPathInfo {
  pub root:          PathBuf,
  pub relative_path: PathBuf,
  pub absolute_path: PathBuf,
  pub date:          String,
  pub year:          i32,
  pub month:         u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub enum MemorySummaryScope {
  Rolling,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct MemorySummaryPathInfo {
  pub root:          PathBuf,
  pub relative_path: PathBuf,
  pub absolute_path: PathBuf,
  pub scope:         MemorySummaryScope,
}

impl MemorySummaryPathInfo {
  pub fn rolling(root: PathBuf) -> Self {
    let relative_path = PathBuf::from(MemorySummaryContract::FILE_NAME);
    let absolute_path = root.join(&relative_path);
    Self { root, relative_path, absolute_path, scope: MemorySummaryScope::Rolling }
  }

  pub fn relative_display_path(&self) -> String {
    format!("{}/{}", MemoryContract::ROOT_DIR, self.relative_path.display())
  }
}

impl MemoryPathInfo {
  pub fn for_date(root: PathBuf, date: NaiveDate) -> Self {
    let year = date.year();
    let month = date.month();
    let relative_path = PathBuf::from(format!("{}/{}", MemoryContract::DAILY_DIR, date.format("%Y-%m-%d")));
    let absolute_path = root.join(&relative_path);

    Self { root, relative_path, absolute_path, date: date.format("%Y-%m-%d").to_string(), year, month }
  }

  pub fn relative_display_path(&self) -> String {
    let relative = self.relative_path.display();
    format!("{}/{relative}", MemoryContract::ROOT_DIR)
  }
}

pub fn local_today() -> NaiveDate {
  Local::now().date_naive()
}

pub fn daily_relative_path_for_date(date: NaiveDate) -> PathBuf {
  MemoryPathInfo::for_date(PathBuf::new(), date).relative_path
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn daily_memory_paths_use_daily_directory_layout() {
    let root = PathBuf::from("memories");
    let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
    let info = MemoryPathInfo::for_date(root.clone(), date);

    assert_eq!(info.relative_path, PathBuf::from("daily/2026-03-08.md"));
    assert_eq!(info.absolute_path, root.join("daily/2026-03-08.md"));
    assert_eq!(info.relative_display_path(), "memories/daily/2026-03-08.md");
    assert_eq!(daily_relative_path_for_date(date), PathBuf::from("daily/2026-03-08.md"));
  }

  #[test]
  fn rolling_summary_path_is_summary_markdown_at_memory_root() {
    let root = PathBuf::from("memories");
    let info = MemorySummaryPathInfo::rolling(root.clone());

    assert_eq!(info.relative_path, PathBuf::from("summary.md"));
    assert_eq!(info.absolute_path, root.join("summary.md"));
    assert_eq!(info.relative_display_path(), "memories/summary.md");
    assert_eq!(info.scope, MemorySummaryScope::Rolling);
  }
}
