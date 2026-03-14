use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use surrealdb_types::SurrealValue;

use crate::shared::prelude::SurrealId;
use crate::tools::ToolUseResponseData;

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  fake::Dummy,
  SurrealEnumValue,
)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
  #[default]
  Pending,
  InProgress,
  Complete,
  #[schemars(skip)]
  #[serde(other)]
  Unknown,
}

impl Display for PlanStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PlanStatus::Pending => "pending",
        PlanStatus::InProgress => "in_progress",
        PlanStatus::Complete => "complete",
        PlanStatus::Unknown => "unknown",
      }
    )
  }
}

impl FromStr for PlanStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    match s {
      "pending" => Ok(PlanStatus::Pending),
      "in_progress" => Ok(PlanStatus::InProgress),
      "complete" => Ok(PlanStatus::Complete),
      "unknown" => Ok(PlanStatus::Unknown),
      _ => Err(anyhow::Error::msg(format!("invalid plan status: {}", s))),
    }
  }
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema, SurrealValue,
)]
#[schemars(inline)]
pub struct PlanTodoItem {
  pub id:      String,
  pub content: String,
  #[schemars(default)]
  pub status:  PlanStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct PlanSummary {
  pub id:          String,
  pub name:        String,
  pub description: String,
  pub created_at:  String,
  pub updated_at:  String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(
  title = "plan_create",
  description = "Creates a new plan for the current project. Description is a short user facing description of the plan. Content is the full plan in markdown format."
)]
pub struct PlanCreateArgs {
  pub name:        String,
  pub description: String,
  pub content:     String,
  pub todos:       Option<Vec<PlanTodoItem>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct PlanCreatePayload {
  pub id:          String,
  pub name:        String,
  pub description: String,
  pub created_at:  String,
  pub updated_at:  String,
}

impl From<PlanCreatePayload> for ToolUseResponseData {
  fn from(payload: PlanCreatePayload) -> Self {
    Self::PlanCreate(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(title = "plan_list", description = "Lists plans for the current project.")]
pub struct PlanListArgs {}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct PlanListPayload {
  pub items: Vec<PlanSummary>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PlanListSortBy {
  Name,
  CreatedAt,
  #[default]
  UpdatedAt,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
  Asc,
  #[default]
  Desc,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanListSort {
  #[serde(default)]
  pub by:        PlanListSortBy,
  #[serde(default)]
  pub direction: SortDirection,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanListQuery {
  #[serde(default)]
  pub search:        Option<String>,
  #[serde(default)]
  pub sort:          Option<PlanListSort>,
  #[serde(default)]
  pub status_filter: Option<Vec<PlanDocumentStatus>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectPlanListItem {
  pub id:                String,
  pub name:              String,
  pub description:       String,
  pub created_at:        String,
  pub updated_at:        String,
  #[serde(default)]
  pub status:            PlanDocumentStatus,
  #[serde(default)]
  pub parent_session_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct ProjectPlanListPayload {
  pub items: Vec<ProjectPlanListItem>,
}

impl From<PlanListPayload> for ToolUseResponseData {
  fn from(payload: PlanListPayload) -> Self {
    Self::PlanList(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(title = "plan_get", description = "Retrieves a single plan by its ID.")]
pub struct PlanGetArgs {
  pub id: String,
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema, SurrealValue,
)]
pub struct PlanGetPayload {
  pub id:                String,
  pub name:              String,
  pub description:       String,
  pub content:           String,
  pub created_at:        String,
  pub updated_at:        String,
  #[serde(default)]
  pub status:            PlanDocumentStatus,
  #[serde(default)]
  pub parent_session_id: Option<String>,
  #[schemars(default)]
  pub todos:             Vec<PlanTodoItem>,
}

impl From<PlanGetPayload> for ToolUseResponseData {
  fn from(payload: PlanGetPayload) -> Self {
    Self::PlanGet(payload)
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(inline)]
pub struct PlanContentPatch {
  pub hunks: Vec<PlanContentPatchHunk>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(inline)]
pub struct PlanContentPatchHunk {
  #[serde(default)]
  #[schemars(default)]
  pub before: Vec<String>,
  #[serde(default)]
  #[schemars(default)]
  pub delete: Vec<String>,
  #[serde(default)]
  #[schemars(default)]
  pub insert: Vec<String>,
  #[serde(default)]
  #[schemars(default)]
  pub after:  Vec<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(
  title = "plan_update",
  description = "Updates an existing plan by its ID. Description is a short user facing description of the plan. Content is the full plan markdown body. content and content_patch are mutually exclusive. content_patch applies only to the plan_get.content body using exact-match, line-oriented hunks and must fail if any hunk matches zero or multiple locations. Patch application is all-or-nothing."
)]
pub struct PlanUpdateArgs {
  pub id:            String,
  #[schemars(default)]
  pub name:          Option<String>,
  #[schemars(default)]
  pub description:   Option<String>,
  #[schemars(default)]
  pub content:       Option<String>,
  #[schemars(default)]
  pub content_patch: Option<PlanContentPatch>,
  #[schemars(default)]
  pub todos:         Option<Vec<PlanTodoItem>>,
  #[schemars(default)]
  pub status:        Option<PlanDocumentStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct PlanUpdatePayload {
  pub id:          String,
  pub name:        String,
  pub description: String,
  pub created_at:  String,
  pub updated_at:  String,
}

impl From<PlanUpdatePayload> for ToolUseResponseData {
  fn from(payload: PlanUpdatePayload) -> Self {
    Self::PlanUpdate(payload)
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type, schemars::JsonSchema)]
#[schemars(title = "plan_delete", description = "Deletes a plan by its ID.")]
pub struct PlanDeleteArgs {
  pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
pub struct PlanDeletePayload {
  pub id: String,
}

impl From<PlanDeletePayload> for ToolUseResponseData {
  fn from(payload: PlanDeletePayload) -> Self {
    Self::PlanDelete(payload)
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanWriteContext {
  pub plan_id:    String,
  pub plan_path:  String,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanMeta {
  pub name:              String,
  pub description:       String,
  pub todos:             Vec<PlanTodoItem>,
  pub created_at:        String,
  pub updated_at:        String,
  pub status:            PlanDocumentStatus,
  pub parent_session_id: Option<String>,
}

#[derive(
  Clone,
  Debug,
  Default,
  PartialEq,
  Eq,
  serde::Serialize,
  serde::Deserialize,
  specta::Type,
  schemars::JsonSchema,
  SurrealEnumValue,
)]
#[schemars(inline)]
#[serde(rename_all = "snake_case")]
pub enum PlanDocumentStatus {
  #[default]
  Pending,
  InProgress,
  Completed,
  Archived,
}

impl Display for PlanDocumentStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PlanDocumentStatus::Pending => "pending",
        PlanDocumentStatus::InProgress => "in_progress",
        PlanDocumentStatus::Completed => "completed",
        PlanDocumentStatus::Archived => "archived",
      }
    )
  }
}

impl FromStr for PlanDocumentStatus {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    match s {
      "pending" => Ok(PlanDocumentStatus::Pending),
      "in_progress" => Ok(PlanDocumentStatus::InProgress),
      "completed" => Ok(PlanDocumentStatus::Completed),
      "archived" => Ok(PlanDocumentStatus::Archived),
      _ => Err(anyhow::Error::msg(format!("invalid plan document status: {}", s))),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanItem {
  pub id:          String,
  pub name:        String,
  pub description: String,
  pub content:     String,
  pub created_at:  String,
  pub updated_at:  String,
  pub todos:       Vec<PlanTodoItem>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PlanDirectory {
  pub project_id: SurrealId,
  pub path:       String,
}
