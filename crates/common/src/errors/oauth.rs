#[derive(Debug, thiserror::Error)]
pub enum OauthError {
  #[error("failed to insert bearer: {0}")]
  FailedToInsertBearer(String),

  #[error("failed to insert api key: {0}")]
  FailedToInsertApiKey(String),

  #[error("invalid base authorize url: {0}")]
  InvalidBaseAuthorizeUrl(String),

  #[error("failed to bind fixed port: {0}")]
  FailedToBindFixedPort(String),

  #[error("failed to bind local callback listener: {0}")]
  FailedToBindLocalCallbackListener(String),

  #[error("failed to open browser: {0}")]
  FailedToOpenBrowser(String),

  #[error("failed to get local address: {0}")]
  FailedToGetLocalAddress(String),

  #[error("missing authorization code: {0}")]
  MissingAuthorizationCode(String),

  #[error("state mismatch: {0}")]
  StateMismatch(String),

  #[error("missing state: {0}")]
  MissingState(String),

  #[error("failed to exchange code for token: {0}")]
  FailedToExchangeCodeForToken(String),

  #[error("failed to parse token response: {0}")]
  FailedToParseTokenResponse(String),

  #[error("failed to send token request: {0}")]
  FailedToSendTokenRequest(String),

  #[error("failed to refresh with refresh token: {0}")]
  FailedToRefreshWithRefreshToken(String),

  #[error("failed to parse refresh response: {0}")]
  FailedToParseRefreshResponse(String),

  #[error("failed to send refresh request: {0}")]
  FailedToSendRefreshRequest(String),

  #[error("failed to parse id token: {0}")]
  FailedToParseIdToken(String),

  #[error("failed to decode id token: {0}")]
  FailedToDecodeIdToken(String),

  #[error("failed to get oauth token: {0}")]
  FailedToGetOauthToken(String),

  #[error("failed to start oauth: {0}")]
  FailedToStartOauth(String),

  #[error("failed to sign in: {0}")]
  FailedToSignIn(String),

  #[error("failed to get subscription: {0}")]
  FailedToGetSubscription(String),
}
