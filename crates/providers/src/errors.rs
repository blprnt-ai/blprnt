

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ProviderErrorKind {
  AuthHeaders,
  BadRequest,
  Canceled,
  Decoding,
  Encoding,
  External,
  Internal,
  InvalidProvider,
  LlmError,
  LlmMistake,
  Network,
  NotSupported,
  NotSupportedStreaming,
  RateLimit,
  Timeout,
  Upstream,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProviderError {
  pub kind:    ProviderErrorKind,
  pub http:    Option<u16>,
  pub code:    Option<String>,
  pub message: String,
  pub data:    Option<serde_json::Value>,
}

impl ProviderError {
  pub fn new(kind: ProviderErrorKind, message: impl Into<String>) -> Self {
    Self { kind, http: None, code: None, message: message.into(), data: None }
  }

  pub fn with_http(mut self, status: u16) -> Self {
    self.http = Some(status);
    self
  }

  pub fn with_code(mut self, code: impl Into<String>) -> Self {
    self.code = Some(code.into());
    self
  }

  pub fn with_data(mut self, data: serde_json::Value) -> Self {
    self.data = Some(data);
    self
  }

  pub fn auth(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::AuthHeaders, msg)
  }

  pub fn rate_limit(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::RateLimit, msg)
  }

  pub fn timeout() -> Self {
    Self::new(ProviderErrorKind::Timeout, "request timed out")
  }

  pub fn canceled() -> Self {
    Self::new(ProviderErrorKind::Canceled, "request canceled")
  }

  pub fn bad_request(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::BadRequest, msg)
  }

  pub fn not_supported(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::NotSupported, msg)
  }

  pub fn upstream(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::Upstream, msg)
  }

  pub fn internal(msg: impl Into<String>) -> Self {
    Self::new(ProviderErrorKind::Internal, msg)
  }
}

impl std::fmt::Display for ProviderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}: {}", self.kind, self.message)
  }
}

impl std::error::Error for ProviderError {}
