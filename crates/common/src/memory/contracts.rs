use schemars::JsonSchema;
use surrealdb_types::SurrealValue;

use crate::tools::MemorySearchResultItem;

#[derive(
  Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema, SurrealValue,
)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum MemoryWriteSource {
  Explicit,
  Sweep,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema, SurrealValue)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum MemoryWriteStatus {
  Written,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAutomaticExtractionTrigger {
  /// App boot catch-up across dirty sessions, including sessions that are not open.
  BootCatchUp,
  /// Low-frequency app-global periodic sweep while the app is open.
  PeriodicSweep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryContract;

impl MemoryContract {
  /// Automatic extraction triggers allowed in v1.
  pub const AUTOMATIC_EXTRACTION_TRIGGERS: &'static [MemoryAutomaticExtractionTrigger] =
    &[MemoryAutomaticExtractionTrigger::BootCatchUp, MemoryAutomaticExtractionTrigger::PeriodicSweep];
  /// Canonical tool operations for the replacement design.
  pub const CANONICAL_TOOL_OPERATIONS: &'static [&'static str] = &["memory_write", "memory_search"];
  /// Collection name for the managed QMD memory collection.
  pub const COLLECTION_NAME: &'static str = "memories";
  /// Daily path layout under the global root using local system/app timezone.
  pub const DAILY_DIR: &'static str = "daily";
  /// Deletes are not supported in v1.
  pub const DELETE_SUPPORTED: bool = false;
  /// Direct writes must go straight to today's file and must not wait for sweep.
  pub const DIRECT_WRITES_TARGET_TODAY: bool = true;
  /// File creation rule required during boot.
  pub const ENSURE_TODAYS_FILE_ON_BOOT: bool = true;
  /// Memory is global to the app. Project-scoped memory is explicitly forbidden.
  pub const GLOBAL_SCOPE: bool = true;
  /// Canonical contract location for the replacement design.
  pub const IS_CANONICAL_REPLACEMENT_CONTRACT: bool = true;
  /// Plain markdown logs are first-class files.
  pub const MANUAL_EDITING_SUPPORTED: bool = true;
  /// Automatic extraction must not run post-turn.
  pub const POST_TURN_EXTRACTION_SUPPORTED: bool = false;
  /// Root directory relative to app home.
  pub const ROOT_DIR: &'static str = "memories";
  /// Markdown files are the only source of truth.
  pub const SOURCE_OF_TRUTH_FORMAT: &'static str = "markdown";
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemorySummaryContract;

impl MemorySummaryContract {
  pub const FILE_NAME: &'static str = "summary.md";
  pub const PATH_LAYOUT: &'static str = "memories/summary.md";
  pub const SOFT_TOKEN_BUDGET: usize = 1200;
}

#[derive(Debug, serde::Deserialize)]
pub struct QmdSearchContract {
  pub docid: String,
  pub score: f64,
  pub file:  String,
  pub title: String,
  pub body:  String,
}

impl From<QmdSearchContract> for MemorySearchResultItem {
  fn from(item: QmdSearchContract) -> Self {
    Self { score: item.score, title: item.title, content: item.body }
  }
}
