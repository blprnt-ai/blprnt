#[derive(Debug, thiserror::Error)]
pub enum EngineError {
  #[error("invalid subagent id: {0}")]
  InvalidSubagentId(String),

  #[error("context window too large: {token_count} > {token_max}")]
  ContextWindowTooLarge { token_count: u32, token_max: u32 },

  #[error("failed to parse tool arguments: {0}")]
  FailedToParseToolArgs(String),

  #[error("provider channel error: {0}")]
  ProviderChannelError(String),

  #[error("provider not found")]
  ProviderNotFound,

  #[error("failed to deserialize credentials: {0}")]
  FailedToDeserializeCredentials(String),

  #[error("provider credentials not found")]
  ProviderCredentialsNotFound,

  #[error("project not found")]
  ProjectNotFound,

  #[error("no auto router models")]
  NoAutoRouterModels,

  #[error("invalid model store: {0}")]
  InvalidModelStore(String),

  #[error("model not found: {0}")]
  ModelNotFound(String),

  #[error("invalid provider id: {0}")]
  InvalidProviderId(String),

  #[error("invalid model slug: {0}")]
  InvalidModelSlug(String),

  #[error("session project missing")]
  SessionProjectMissing,

  #[error("session personality missing")]
  SessionPersonalityMissing,
}
