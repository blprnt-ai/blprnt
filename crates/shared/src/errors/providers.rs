#[derive(Clone, Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum ProviderError {
  #[error("user cancelled")]
  UserCancelled,

  #[error("external network error: {context}: {message}")]
  ExternalNetwork { context: String, message: String },

  #[error("decoding failed: {context}: {message}")]
  DecodingFailed { context: String, message: String },

  #[error("LLM mistake: {context}: {message}")]
  LlmMistake { context: String, message: String },

  #[error("LLM error: {context}: {message}")]
  LlmError { context: String, message: String },

  #[error("LLM unknown error: {context}: {message}")]
  LlmUnknownError { context: String, message: String },

  #[error("rate limit error: {context}: {message}")]
  RateLimit { context: String, message: String },

  #[error("rate limit error: {context}: {message}")]
  RateLimitUpstream { context: String, message: String },

  #[error("bad request error: {context}: {message}")]
  BadRequest { context: String, message: String },

  #[error("unauthorized error ({url}): {context}: {message}")]
  Unauthorized { url: String, context: String, message: String },

  #[error("cannot clone request")]
  CannotCloneRequest,

  #[error("UTF-8 error: {0}")]
  Utf8(String),

  #[error("parser error: {0}")]
  Parser(String),

  #[error("transport error: {0}")]
  Transport(String),

  #[error("middleware transport error: {0}")]
  MiddlewareTransport(String),

  #[error("invalid content type: {0}")]
  InvalidContentType(String),

  #[error("invalid status code: {code} {status_text}: {body}")]
  InvalidStatusCode { code: String, status_text: String, body: String },

  #[error("stream ended")]
  StreamEnded,

  #[error("decode error: {0}")]
  Decode(String),

  #[error("auth headers error: {0}")]
  AuthHeaders(String),

  #[error("timeout")]
  Timeout,

  #[error("canceled")]
  Canceled,

  #[error("not supported: {0}")]
  NotSupported(String),

  #[error("not supported streaming: {0}")]
  NotSupportedStreaming(String),

  #[error("upstream error: {0}")]
  Upstream(String),

  #[error("internal error: {0}")]
  Internal(String),

  #[error("invalid provider: {0}")]
  InvalidProvider(String),

  #[error("encoding error: {0}")]
  Encoding(String),

  #[error("invalid schema: {0}")]
  InvalidSchema(String),

  #[error("invalid tool id: {tool_id}: {context}: {message}")]
  InvalidToolId { call_id: String, tool_id: String, arguments: String, context: String, message: String },

  // ============================================================================
  // OpenRouter-specific errors
  // ============================================================================
  /// Content was blocked by moderation/safety filters
  #[error("{message}")]
  ContentModeration { code: String, message: String },

  /// Insufficient credits or billing not set up
  #[error("{message}")]
  InsufficientCredits { code: String, message: String },

  /// The requested model was not found
  #[error("{message}")]
  ModelNotFound { model: String, message: String },

  /// The model is temporarily unavailable
  #[error("{message}")]
  ModelUnavailable { model: String, message: String },

  /// Upstream provider is unavailable or returned an error
  #[error("{message}")]
  ProviderUnavailable { provider: String, message: String },

  /// Gateway timeout - the request took too long
  #[error("{message}")]
  GatewayTimeout { message: String },

  /// Server error from the API
  #[error("{message}")]
  ServerError { code: String, message: String },

  /// Invalid API key
  #[error("{message}")]
  InvalidApiKey { message: String },

  /// Context window exceeded for the model
  #[error("{message}")]
  ContextLengthExceeded { model: String, message: String },
}

impl ProviderError {
  pub fn auth(msg: impl Into<String>) -> Self {
    Self::AuthHeaders(msg.into())
  }

  pub fn rate_limit(msg: impl Into<String>) -> Self {
    Self::RateLimit { context: "unknown".to_string(), message: msg.into() }
  }

  pub fn timeout() -> Self {
    Self::Timeout
  }

  pub fn canceled() -> Self {
    Self::Canceled
  }

  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self::BadRequest { context: "unknown".to_string(), message: msg.into() }
  }

  pub fn not_supported(msg: impl Into<String>) -> Self {
    Self::NotSupported(msg.into())
  }

  pub fn upstream(msg: impl Into<String>) -> Self {
    Self::Upstream(msg.into())
  }

  pub fn internal(msg: impl Into<String>) -> Self {
    Self::Internal(msg.into())
  }
}
