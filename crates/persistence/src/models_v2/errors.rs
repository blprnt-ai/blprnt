#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
  #[error("failed to begin transaction: {0}")]
  FailedToBeginTransaction(anyhow::Error),

  #[error("failed to commit transaction: {0}")]
  FailedToCommitTransaction(anyhow::Error),

  #[error("failed to create issue: {0}")]
  FailedToCreateIssue(anyhow::Error),

  #[error("failed to relate issue to company: {0}")]
  FailedToRelateIssueToCompany(anyhow::Error),

  #[error("issue not found after creation")]
  IssueNotFoundAfterCreation,

  #[error("failed to get issue: {0}")]
  FailedToGetIssue(anyhow::Error),

  #[error("failed to list issues: {0}")]
  FailedToListIssues(anyhow::Error),

  #[error("failed to list children issues: {0}")]
  FailedToListChildrenIssues(anyhow::Error),

  #[error("failed to list comments: {0}")]
  FailedToListComments(anyhow::Error),

  #[error("failed to list actions: {0}")]
  FailedToListActions(anyhow::Error),

  #[error("failed to list attachments: {0}")]
  FailedToListAttachments(anyhow::Error),

  #[error("failed to update issue: {0}")]
  FailedToUpdateIssue(anyhow::Error),

  #[error("failed to delete issue: {0}")]
  FailedToDeleteIssue(anyhow::Error),

  #[error("failed to checkout issue: {0}")]
  FailedToCheckoutIssue(anyhow::Error),

  #[error("failed to release issue: {0}")]
  FailedToReleaseIssue(anyhow::Error),

  #[error("issue already checked out by another employee")]
  IssueAlreadyCheckedOutByAnotherEmployee,

  #[error("issue not found")]
  IssueNotFound,

  #[error("failed to create issue comment: {0}")]
  FailedToCreateIssueComment(anyhow::Error),

  #[error("failed to get issue comment: {0}")]
  FailedToGetIssueComment(anyhow::Error),

  #[error("issue comment not found")]
  IssueCommentNotFound,

  #[error("failed to create issue action: {0}")]
  FailedToCreateIssueAction(anyhow::Error),

  #[error("failed to get issue action: {0}")]
  FailedToGetIssueAction(anyhow::Error),

  #[error("issue action not found")]
  IssueActionNotFound,

  #[error("failed to create issue attachment: {0}")]
  FailedToCreateIssueAttachment(anyhow::Error),

  #[error("failed to get issue attachment: {0}")]
  FailedToGetIssueAttachment(anyhow::Error),

  #[error("issue attachment not found")]
  IssueAttachmentNotFound,

  #[error("failed to create employee: {0}")]
  FailedToCreateEmployee(anyhow::Error),

  #[error("failed to get employee: {0}")]
  FailedToGetEmployee(anyhow::Error),

  #[error("employee not found")]
  EmployeeNotFound,

  #[error("employee not found after creation")]
  EmployeeNotFoundAfterCreation,

  #[error("failed to relate employee to company: {0}")]
  FailedToRelateEmployeeToCompany(anyhow::Error),

  #[error("failed to list employees: {0}")]
  FailedToListEmployees(anyhow::Error),

  #[error("failed to update employee: {0}")]
  FailedToUpdateEmployee(anyhow::Error),

  #[error("failed to delete employee: {0}")]
  FailedToDeleteEmployee(anyhow::Error),

  #[error("failed to create company: {0}")]
  FailedToCreateCompany(anyhow::Error),

  #[error("failed to get company: {0}")]
  FailedToGetCompany(anyhow::Error),

  #[error("company not found")]
  CompanyNotFound,

  #[error("company not found after creation")]
  CompanyNotFoundAfterCreation,

  #[error("failed to list companies: {0}")]
  FailedToListCompanies(anyhow::Error),

  #[error("failed to update company: {0}")]
  FailedToUpdateCompany(anyhow::Error),

  #[error("failed to delete company: {0}")]
  FailedToDeleteCompany(anyhow::Error),

  #[error("failed to create run: {0}")]
  FailedToCreateRun(anyhow::Error),

  #[error("failed to get run: {0}")]
  FailedToGetRun(anyhow::Error),

  #[error("failed to list runs: {0}")]
  FailedToListRuns(anyhow::Error),

  #[error("run not found")]
  RunNotFound,

  #[error("run not found after creation")]
  RunNotFoundAfterCreation,

  #[error("failed to update run: {0}")]
  FailedToUpdateRun(anyhow::Error),

  #[error("failed to create turn: {0}")]
  FailedToCreateTurn(anyhow::Error),

  #[error("failed to get turn: {0}")]
  FailedToGetTurn(anyhow::Error),

  #[error("turn not found")]
  TurnNotFound,

  #[error("turn not found after creation")]
  TurnNotFoundAfterCreation,

  #[error("failed to update turn: {0}")]
  FailedToUpdateTurn(anyhow::Error),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
