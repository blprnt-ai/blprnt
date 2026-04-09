use std::fmt::Display;

#[derive(Debug)]
pub enum DatabaseEntity {
  Provider,
  McpServer,
  RunEnabledMcpServer,
  TelegramConfig,
  TelegramLinkCode,
  TelegramLink,
  TelegramIssueWatch,
  TelegramMessageCorrelation,
  LoginCredential,
  AuthSession,
  Employee,
  Minion,
  Project,
  Issue,
  IssueComment,
  IssueAction,
  IssueAttachment,
  Run,
  Turn,
}

impl Display for DatabaseEntity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DatabaseEntity::Provider => write!(f, "provider"),
      DatabaseEntity::McpServer => write!(f, "mcp server"),
      DatabaseEntity::RunEnabledMcpServer => write!(f, "run enabled mcp server"),
      DatabaseEntity::TelegramConfig => write!(f, "telegram config"),
      DatabaseEntity::TelegramLinkCode => write!(f, "telegram link code"),
      DatabaseEntity::TelegramLink => write!(f, "telegram link"),
      DatabaseEntity::TelegramIssueWatch => write!(f, "telegram issue watch"),
      DatabaseEntity::TelegramMessageCorrelation => write!(f, "telegram message correlation"),
      DatabaseEntity::LoginCredential => write!(f, "login credential"),
      DatabaseEntity::AuthSession => write!(f, "auth session"),
      DatabaseEntity::Employee => write!(f, "employee"),
      DatabaseEntity::Minion => write!(f, "minion"),
      DatabaseEntity::Project => write!(f, "project"),
      DatabaseEntity::Issue => write!(f, "issue"),
      DatabaseEntity::IssueComment => write!(f, "issue comment"),
      DatabaseEntity::IssueAction => write!(f, "issue action"),
      DatabaseEntity::IssueAttachment => write!(f, "issue attachment"),
      DatabaseEntity::Run => write!(f, "run"),
      DatabaseEntity::Turn => write!(f, "turn"),
    }
  }
}

#[derive(Debug)]
pub enum DatabaseOperation {
  BeginTransaction,
  CommitTransaction,
  Create,
  Get,
  List,
  Update,
  Delete,
}

impl Display for DatabaseOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DatabaseOperation::BeginTransaction => write!(f, "begin transaction"),
      DatabaseOperation::CommitTransaction => write!(f, "commit transaction"),
      DatabaseOperation::Create => write!(f, "create"),
      DatabaseOperation::Get => write!(f, "get"),
      DatabaseOperation::List => write!(f, "list"),
      DatabaseOperation::Update => write!(f, "update"),
      DatabaseOperation::Delete => write!(f, "delete"),
    }
  }
}

#[derive(Debug)]
pub enum DatabaseConflict {
  AlreadyCheckedOut,
  AlreadyExists,
}

impl Display for DatabaseConflict {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DatabaseConflict::AlreadyCheckedOut => write!(f, "already checked out"),
      DatabaseConflict::AlreadyExists => write!(f, "already exists"),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
  #[error("{entity} {operation} failed: {source}")]
  Operation {
    entity:    DatabaseEntity,
    operation: DatabaseOperation,
    #[source]
    source:    anyhow::Error,
  },

  #[error("{entity} not found")]
  NotFound { entity: DatabaseEntity },

  #[error("{entity} not found after creation")]
  NotFoundAfterCreate { entity: DatabaseEntity },

  #[error("{entity} conflict: {reason}")]
  Conflict { entity: DatabaseEntity, reason: DatabaseConflict },
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
