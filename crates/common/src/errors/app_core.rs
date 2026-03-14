#[derive(Debug, thiserror::Error)]
pub enum AppCoreError {
  #[error("failed to open store: {0}")]
  FailedToOpenStore(String),

  #[error("project not found")]
  ProjectNotFound,

  #[error("session not found: {0}")]
  SessionNotFound(String),

  #[error("indexing is not enabled for this project")]
  IndexingNotEnabled,

  #[error("codex credentials not found")]
  CodexCredentialsNotFound,

  #[error("claude credentials not found")]
  ClaudeCredentialsNotFound,

  #[error("Another plan is already in progress")]
  PlanAlreadyInProgress,

  #[error("plan '{plan_id}' is not editable because status is '{status}'")]
  PlanNotPending { plan_id: String, status: String },

  #[error("plan '{0}' is not associated with any session")]
  PlanStatusNotFound(String),

  #[error(
    "Session already has a plan attached ('{existing_plan_id}'). Detach it first before attaching '{requested_plan_id}'."
  )]
  SessionAlreadyHasDifferentPlan { session_id: String, existing_plan_id: String, requested_plan_id: String },

  #[error(
    "Plan '{plan_id}' is already attached to session '{parent_session_id}'. Unassign it first before attaching to session '{requested_session_id}'."
  )]
  PlanAlreadyAttachedToDifferentSession {
    plan_id:              String,
    parent_session_id:    String,
    requested_session_id: String,
  },

  #[error(
    "Plan '{plan_id}' is attached to session '{parent_session_id}', not session '{requested_session_id}'. Detach it from the current parent first."
  )]
  PlanAttachedToDifferentSession {
    plan_id:              String,
    parent_session_id:    String,
    requested_session_id: String,
  },

  #[error("Plan '{plan_id}' is not attached to session '{session_id}'.")]
  PlanNotAttachedToSession { plan_id: String, session_id: String },
}
