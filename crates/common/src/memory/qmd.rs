use std::collections::BTreeSet;
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use anyhow::Result;
use schemars::JsonSchema;
use thiserror::Error;

use super::api::MemorySearchRequest;
use super::api::MemorySearchResult;
use super::api::MemorySearchResultItem;
use super::contracts::MemoryContract;
use crate::memory::QmdSearchContract;
#[cfg(not(test))]
use crate::paths::BlprntPath;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Error)]
pub enum QmdMemorySearchError {
  #[error("qmd readiness failed [{state}]: {detail}")]
  Readiness { state: QmdMemoryReadinessState, detail: String },
  #[error("qmd recovery failed [{state}] after action={action} attempts={attempts}: {detail}")]
  Recovery { state: QmdMemoryReadinessState, action: QmdRecoveryAction, attempts: u8, detail: String },
  #[error("qmd bootstrap failed: {0}")]
  Bootstrap(String),
  #[error("qmd refresh failed: {0}")]
  Refresh(String),
  #[error("qmd query failed: {0}")]
  Query(String),
  #[error("qmd response parse failed: {0}")]
  Response(String),
  #[error("qmd hit path rejected '{path}': {reason}")]
  HitPath { path: String, reason: String },
  #[error("memory file shape failed for '{path}': {reason}")]
  MemoryShape { path: String, reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QmdMemoryReadinessState {
  RuntimeMissing,
  RuntimeUnsupported,
  QmdMissingFromPath,
  QmdUnavailable,
  Ready,
}

impl std::fmt::Display for QmdMemoryReadinessState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let value = match self {
      Self::RuntimeMissing => "runtime_missing/prompt_eligible",
      Self::RuntimeUnsupported => "runtime_unsupported",
      Self::QmdMissingFromPath => "qmd_missing_from_path",
      Self::QmdUnavailable => "qmd_unavailable",
      Self::Ready => "ready",
    };
    write!(f, "{value}")
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct QmdMemoryReadiness {
  pub state:  QmdMemoryReadinessState,
  pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QmdManagedCollectionLastEvent {
  Registered,
  Repaired,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct QmdManagedCollectionDiagnostics {
  pub collection_name:         String,
  pub registered:              bool,
  pub bootstrap_state_present: bool,
  pub root_matches:            bool,
  pub last_event:              Option<QmdManagedCollectionLastEvent>,
  pub last_event_at:           Option<String>,
  pub detail:                  Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QmdRecoveryAction {
  None,
  PromptForRuntimeSetup,
  HonorPromptSuppression,
  RestartSidecar,
  RebuildManagedCollection,
}

impl std::fmt::Display for QmdRecoveryAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let value = match self {
      Self::None => "none",
      Self::PromptForRuntimeSetup => "prompt_for_runtime_setup",
      Self::HonorPromptSuppression => "honor_prompt_suppression",
      Self::RestartSidecar => "restart_sidecar",
      Self::RebuildManagedCollection => "rebuild_managed_collection",
    };
    write!(f, "{value}")
  }
}

#[derive(Clone, Debug)]
pub struct QmdMemorySearchService {
  root:       PathBuf,
  qmd_bin:    String,
  project_id: String,
}

impl QmdMemorySearchService {
  pub fn new(project_id: String) -> Self {
    #[cfg(not(test))]
    let root = BlprntPath::memories_root();
    #[cfg(test)]
    let root = PathBuf::from("/Users/supagoku/Library/Application Support/ai.blprnt/memories");

    #[cfg(not(test))]
    let home = BlprntPath::home();
    #[cfg(test)]
    let home = PathBuf::from("/Users/supagoku");

    let qmd_bin = home.join(".bun").join("bin").join("qmd");
    let qmd_bin = qmd_bin.to_string_lossy().to_string();

    let root = root.join(&project_id);

    Self { root, qmd_bin, project_id }
  }

  pub async fn search(
    &self,
    request: &MemorySearchRequest,
    min_score: Option<f64>,
  ) -> Result<MemorySearchResult, QmdMemorySearchError> {
    let result_limit = request.limit.unwrap_or(10);

    let mut command = Command::new(&self.qmd_bin);
    let command = command
      .arg("query")
      .arg(request.query.clone())
      .arg("-c")
      .arg(self.collection_name())
      .arg("--json")
      .arg("--full")
      .args(["-n", &result_limit.to_string()])
      .args(["--min-score", &min_score.unwrap_or(0.6).to_string()]);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let response = command.output().map_err(|error| QmdMemorySearchError::Query(error.to_string()))?;

    if !response.status.success() {
      return Err(QmdMemorySearchError::Query(best_effort_command_error(&response)));
    }

    let body = String::from_utf8_lossy(&response.stdout);
    let body: Vec<QmdSearchContract> =
      serde_json::from_str(&body).map_err(|error| QmdMemorySearchError::Response(error.to_string()))?;

    Ok(MemorySearchResult {
      memories: body
        .into_iter()
        .filter_map(|item| if item.file.ends_with("summary.md") { None } else { Some(item.into()) })
        .collect::<Vec<MemorySearchResultItem>>(),
    })
  }

  pub async fn refresh(&self) -> Result<(), QmdMemorySearchError> {
    self.index_collections()?;
    self.embed_collections()?;

    Ok(())
  }

  pub fn bootstrap_collections(&self) -> Result<(), QmdMemorySearchError> {
    fs::create_dir_all(&self.root).map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;
    // fs::create_dir_all(&self.daily).map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;

    let collections = self.list_collections()?;
    let memory_collection_missing = !collections.contains(&self.collection_name());

    if memory_collection_missing {
      tracing::info!(
        collection = %self.collection_name(),
        registration_path = %self.root.display(),
        "Registering managed QMD memory collection"
      );
      self.add_collection(&self.root.join(MemoryContract::DAILY_DIR), &self.collection_name())?;
      self.index_collections()?;
      self.embed_collections()?;
    }

    Ok(())
  }

  fn list_collections(&self) -> Result<BTreeSet<String>, QmdMemorySearchError> {
    let mut command = Command::new(&self.qmd_bin);
    command.args(["collection", "list"]);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;
    parse_qmd_collection_list_output(&output)
  }

  fn add_collection(&self, registration_path: &Path, name: &str) -> Result<(), QmdMemorySearchError> {
    let mut command = Command::new(&self.qmd_bin);
    command.args(["collection", "add"]);
    command.args(["--name", name]);

    let registration_path = registration_path.to_string_lossy().to_string();
    let registration_path = registration_path.strip_suffix("/").unwrap_or(&registration_path);
    command.arg(registration_path);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;
    if output.status.success() {
      return Ok(());
    }

    Err(QmdMemorySearchError::Bootstrap(best_effort_command_error(&output)))
  }

  fn index_collections(&self) -> Result<(), QmdMemorySearchError> {
    let mut command = Command::new(&self.qmd_bin);
    command.args(["update"]);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;
    if output.status.success() {
      return Ok(());
    }

    Err(QmdMemorySearchError::Bootstrap(best_effort_command_error(&output)))
  }

  fn embed_collections(&self) -> Result<(), QmdMemorySearchError> {
    let mut command = Command::new(&self.qmd_bin);
    command.args(["embed"]);

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command.output().map_err(|error| QmdMemorySearchError::Bootstrap(error.to_string()))?;
    if output.status.success() {
      return Ok(());
    }

    Err(QmdMemorySearchError::Bootstrap(best_effort_command_error(&output)))
  }

  pub fn collection_name(&self) -> String {
    format!("{}-{}", MemoryContract::COLLECTION_NAME, self.project_id)
  }
}

pub fn detect_command(command: &str) -> bool {
  let mut command = Command::new(command);
  command.arg("--version");

  #[cfg(windows)]
  command.creation_flags(CREATE_NO_WINDOW);

  match command.output() {
    Ok(output) if !output.status.success() => false,
    Ok(_) => true,
    _ => false,
  }
}

pub(crate) fn parse_qmd_collection_list_output(output: &Output) -> Result<BTreeSet<String>, QmdMemorySearchError> {
  if !output.status.success() {
    return Err(QmdMemorySearchError::Bootstrap(best_effort_command_error(output)));
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let mut collections = BTreeSet::new();
  for line in stdout.lines() {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("Collections (") {
      continue;
    }
    if let Some((name, _)) = trimmed.split_once(" (qmd://") {
      collections.insert(name.trim().to_string());
    }
  }
  Ok(collections)
}

fn best_effort_command_error(output: &Output) -> String {
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
  if !stderr.is_empty() {
    stderr
  } else if !stdout.is_empty() {
    stdout
  } else {
    format!("exit status {}", output.status)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn serach_test() {
    let service = QmdMemorySearchService::new("test".to_string());
    let request = MemorySearchRequest { query: "test".to_string(), limit: None };
    let result = service.search(&request, None).await.unwrap();
    println!("{:?}", result);
  }
}
