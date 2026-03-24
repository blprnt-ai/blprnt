use super::DatabaseError;

#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
  #[error("employee is not managed by Coordinator")]
  EmployeeNotManaged,

  #[error("no run slots available for employee")]
  NoRunSlotsAvailable,

  #[error("database error: {0}")]
  DatabaseError(DatabaseError),

  #[error("failed to emit coordinator event: {0}")]
  FailedToEmitCoordinatorEvent(anyhow::Error),

  #[error("failed to await oneshot channel: {0}")]
  FailedToAwaitOneshotChannel(tokio::sync::oneshot::error::RecvError),

  #[error("adapter runtime failed: {0}")]
  AdapterRuntimeFailed(anyhow::Error),
}

pub type CoordinatorResult<T> = Result<T, CoordinatorError>;
