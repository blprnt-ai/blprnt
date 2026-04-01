use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(
  title = "files_read",
  description = "Reads the content of multiple files, optionally limiting to specific line ranges."
)]
pub struct FilesReadArgs {
  pub items:                Vec<FilesReadItem>,
  #[schemars(default, description = "Setting this to true will prefix each new line with `{line_number}: `")]
  pub include_line_numbers: Option<bool>,
  #[schemars(default)]
  #[schemars(
    description = "Optional zero-based workspace index to use. If not provided, the first workspace will be used."
  )]
  pub workspace_index:      Option<u8>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(inline)]
pub struct FilesReadItem {
  pub path:       String,
  #[schemars(default)]
  pub line_start: Option<usize>,
  #[schemars(default)]
  pub line_end:   Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FileReadPayload {
  pub path:    String,
  pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FilesReadPayload {
  pub files:  Vec<FileReadPayload>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub errors: Vec<FilesReadErrorPayload>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FilesReadErrorPayload {
  pub path:  String,
  pub error: String,
}

impl From<FilesReadPayload> for ToolUseResponseData {
  fn from(payload: FilesReadPayload) -> Self {
    Self::FilesRead(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(title = "apply_patch", description = "Applies a V4A patch to one or more files.")]
pub struct ApplyPatchArgs {
  #[schemars(description = "A patch string whose Add File, Update File, Delete File, and Move to headers use absolute paths.")]
  pub diff: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ApplyPatchPayload {
  pub paths: Vec<String>,
}

impl From<ApplyPatchPayload> for ToolUseResponseData {
  fn from(payload: ApplyPatchPayload) -> Self {
    Self::ApplyPatch(payload)
  }
}
