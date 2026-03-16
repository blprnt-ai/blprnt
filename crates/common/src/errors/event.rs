use std::fmt::Display;
use std::str::FromStr;

use macros::SurrealEnumValue;
use surrealdb_types::SurrealValue;

use super::ApiError;
use super::AppCoreError;
use super::EngineError;
use super::OauthError;
use super::ProviderError;
use super::SessionError;
use super::ToolError;

/// Structured error event for frontend consumption.
/// Carries category, code, message, and recoverability information.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type, SurrealValue)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
  pub category:    ErrorCategory,
  pub code:        String,
  pub message:     String,
  pub recoverable: bool,
}

impl ErrorEvent {
  pub fn new(category: ErrorCategory, code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
    Self { category, code: code.into(), message: message.into(), recoverable }
  }

  pub fn internal(message: impl Into<String>) -> Self {
    Self::new(ErrorCategory::Internal, "internal", message, false)
  }
}

/// Category of error for frontend handling.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type, SurrealEnumValue)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
  Provider,
  Tool,
  Engine,
  Auth,
  Network,
  Internal,
}

impl Display for ErrorCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ErrorCategory::Provider => "provider",
        ErrorCategory::Tool => "tool",
        ErrorCategory::Engine => "engine",
        ErrorCategory::Auth => "auth",
        ErrorCategory::Network => "network",
        ErrorCategory::Internal => "internal",
      }
    )
  }
}

impl FromStr for ErrorCategory {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, anyhow::Error> {
    match s {
      "provider" => Ok(ErrorCategory::Provider),
      "tool" => Ok(ErrorCategory::Tool),
      "engine" => Ok(ErrorCategory::Engine),
      "auth" => Ok(ErrorCategory::Auth),
      "network" => Ok(ErrorCategory::Network),
      "internal" => Ok(ErrorCategory::Internal),
      _ => Err(anyhow::Error::msg(format!("invalid error category: {}", s))),
    }
  }
}

// ============================================================================
// From implementations for each error type
// ============================================================================

impl From<&ProviderError> for ErrorEvent {
  fn from(error: &ProviderError) -> Self {
    let (code, recoverable) = match error {
      ProviderError::UserCancelled => ("user_cancelled", false),
      ProviderError::ExternalNetwork { .. } => ("external_network", true),
      ProviderError::DecodingFailed { .. } => ("decoding_failed", false),
      ProviderError::LlmMistake { .. } => ("llm_mistake", true),
      ProviderError::LlmError { .. } => ("llm_error", true),
      ProviderError::LlmUnknownError { .. } => ("llm_unknown_error", true),
      ProviderError::RateLimit { .. } => ("rate_limit", true),
      ProviderError::BadRequest { .. } => ("bad_request", false),
      ProviderError::Unauthorized { .. } => ("unauthorized", false),
      ProviderError::CannotCloneRequest => ("cannot_clone_request", false),
      ProviderError::Utf8(_) => ("utf8_error", false),
      ProviderError::Parser(_) => ("parser_error", false),
      ProviderError::Transport(_) => ("transport_error", true),
      ProviderError::MiddlewareTransport(_) => ("middleware_transport", true),
      ProviderError::InvalidContentType(_) => ("invalid_content_type", false),
      ProviderError::InvalidStatusCode { .. } => ("invalid_status_code", true),
      ProviderError::StreamEnded => ("stream_ended", false),
      ProviderError::Decode(_) => ("decode_error", false),
      ProviderError::AuthHeaders(_) => ("auth_headers", false),
      ProviderError::Timeout => ("timeout", true),
      ProviderError::Canceled => ("canceled", false),
      ProviderError::NotSupported(_) => ("not_supported", false),
      ProviderError::NotSupportedStreaming(_) => ("not_supported_streaming", false),
      ProviderError::Upstream(_) => ("upstream_error", true),
      ProviderError::Internal(_) => ("internal", false),
      ProviderError::InvalidProvider(_) => ("invalid_provider", false),
      ProviderError::Encoding(_) => ("encoding_error", false),
      ProviderError::InvalidSchema(_) => ("invalid_schema", false),
      ProviderError::RateLimitUpstream { .. } => ("rate_limit_upstream", true),
      ProviderError::InvalidToolId { .. } => ("invalid_tool_id", false),
      // OpenRouter-specific errors
      ProviderError::ContentModeration { code, .. } => (code.as_str(), false),
      ProviderError::InsufficientCredits { .. } => ("insufficient_credits", false),
      ProviderError::ModelNotFound { .. } => ("model_not_found", false),
      ProviderError::ModelUnavailable { .. } => ("model_unavailable", true),
      ProviderError::ProviderUnavailable { .. } => ("provider_unavailable", true),
      ProviderError::GatewayTimeout { .. } => ("gateway_timeout", true),
      ProviderError::ServerError { .. } => ("server_error", true),
      ProviderError::InvalidApiKey { .. } => ("invalid_api_key", false),
      ProviderError::ContextLengthExceeded { .. } => ("context_length_exceeded", true),
    };

    ErrorEvent::new(ErrorCategory::Provider, code, error.to_string(), recoverable)
  }
}

impl From<ProviderError> for ErrorEvent {
  fn from(error: ProviderError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&ToolError> for ErrorEvent {
  fn from(error: &ToolError) -> Self {
    let (code, recoverable) = match error {
      ToolError::InvalidArgs { .. } => ("invalid_args", false),
      ToolError::FailedToParseArgs { .. } => ("failed_to_parse_args", false),
      ToolError::UnknownTool(_) => ("unknown_tool", false),
      ToolError::RenameFailed(_) => ("rename_failed", true),
      ToolError::SymbolsFailed(_) => ("symbols_failed", true),
      ToolError::General(_) => ("general", true),
      ToolError::FileReadLineStartBeyondFileEnd { .. } => ("file_read_line_start_beyond_end", false),
      ToolError::FileReadLineStartGreaterThanLineEnd { .. } => ("file_read_line_start_greater_than_end", false),
      ToolError::NoTodoItemsProvided => ("no_todo_items_provided", false),
      ToolError::AccessDenied { .. } => ("access_denied", false),
      ToolError::InvalidWorkspaceRoot { .. } => ("invalid_workspace_root", false),
      ToolError::FileReadFailed { .. } => ("file_read_failed", true),
      ToolError::FileWriteFailed { .. } => ("file_write_failed", true),
      ToolError::PatchParseFailed { .. } => ("patch_parse_failed", false),
      ToolError::PatchApplyFailed { .. } => ("patch_apply_failed", true),
      ToolError::InvalidRegex(_) => ("invalid_regex", false),
      ToolError::SpawnFailed(_) => ("spawn_failed", true),
      ToolError::CommandTimeout => ("command_timeout", true),
      ToolError::ProcessOutputFailed(_) => ("process_output_failed", true),
    };

    ErrorEvent::new(ErrorCategory::Tool, code, error.to_string(), recoverable)
  }
}

impl From<ToolError> for ErrorEvent {
  fn from(error: ToolError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&EngineError> for ErrorEvent {
  fn from(error: &EngineError) -> Self {
    let (code, recoverable) = match error {
      EngineError::ContextWindowTooLarge { .. } => ("context_window_too_large", true),
      EngineError::FailedToParseToolArgs(_) => ("failed_to_parse_tool_args", false),
      EngineError::ProviderChannelError(_) => ("provider_channel_error", true),
      EngineError::ProviderNotFound => ("provider_not_found", false),
      EngineError::FailedToDeserializeCredentials(_) => ("failed_to_deserialize_credentials", false),
      EngineError::ProviderCredentialsNotFound => ("provider_credentials_not_found", false),
      EngineError::ProjectNotFound => ("project_not_found", false),
      EngineError::NoAutoRouterModels => ("no_auto_router_models", false),
      EngineError::InvalidModelStore(_) => ("invalid_model_store", false),
      EngineError::ModelNotFound(_) => ("model_not_found", false),
      EngineError::InvalidProviderId(_) => ("invalid_provider_id", false),
      EngineError::InvalidModelSlug(_) => ("invalid_model_slug", false),
      EngineError::SessionProjectMissing => ("session_project_missing", false),
      EngineError::SessionPersonalityMissing => ("session_personality_missing", false),
      EngineError::InvalidSubagentId(_) => ("invalid_subagent_id", false),
    };

    ErrorEvent::new(ErrorCategory::Engine, code, error.to_string(), recoverable)
  }
}

impl From<EngineError> for ErrorEvent {
  fn from(error: EngineError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&SessionError> for ErrorEvent {
  fn from(error: &SessionError) -> Self {
    let (code, recoverable) = match error {
      SessionError::FailedToCreate(_) => ("failed_to_create_session", false),
      SessionError::FailedToOpen(_) => ("failed_to_open_session", false),
      SessionError::UserInterrupted => ("user_interrupted", true),
    };

    ErrorEvent::new(ErrorCategory::Engine, code, error.to_string(), recoverable)
  }
}

impl From<SessionError> for ErrorEvent {
  fn from(error: SessionError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&OauthError> for ErrorEvent {
  fn from(error: &OauthError) -> Self {
    let (code, recoverable) = match error {
      OauthError::FailedToInsertBearer(_) => ("failed_to_insert_bearer", false),
      OauthError::FailedToInsertApiKey(_) => ("failed_to_insert_api_key", false),
      OauthError::InvalidBaseAuthorizeUrl(_) => ("invalid_base_authorize_url", false),
      OauthError::FailedToBindFixedPort(_) => ("failed_to_bind_fixed_port", true),
      OauthError::FailedToBindLocalCallbackListener(_) => ("failed_to_bind_local_callback_listener", true),
      OauthError::FailedToOpenBrowser(_) => ("failed_to_open_browser", true),
      OauthError::FailedToGetLocalAddress(_) => ("failed_to_get_local_address", true),
      OauthError::MissingAuthorizationCode(_) => ("missing_authorization_code", false),
      OauthError::StateMismatch(_) => ("state_mismatch", false),
      OauthError::MissingState(_) => ("missing_state", false),
      OauthError::FailedToExchangeCodeForToken(_) => ("failed_to_exchange_code_for_token", true),
      OauthError::FailedToParseTokenResponse(_) => ("failed_to_parse_token_response", false),
      OauthError::FailedToSendTokenRequest(_) => ("failed_to_send_token_request", true),
      OauthError::FailedToRefreshWithRefreshToken(_) => ("failed_to_refresh_with_refresh_token", true),
      OauthError::FailedToParseRefreshResponse(_) => ("failed_to_parse_refresh_response", false),
      OauthError::FailedToSendRefreshRequest(_) => ("failed_to_send_refresh_request", true),
      OauthError::FailedToParseIdToken(_) => ("failed_to_parse_id_token", false),
      OauthError::FailedToDecodeIdToken(_) => ("failed_to_decode_id_token", false),
      OauthError::FailedToGetOauthToken(_) => ("failed_to_get_oauth_token", true),
      OauthError::FailedToStartOauth(_) => ("failed_to_start_oauth", true),
      OauthError::FailedToSignIn(_) => ("failed_to_sign_in", true),
      OauthError::FailedToGetSubscription(_) => ("failed_to_get_subscription", true),
    };

    ErrorEvent::new(ErrorCategory::Auth, code, error.to_string(), recoverable)
  }
}

impl From<OauthError> for ErrorEvent {
  fn from(error: OauthError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&AppCoreError> for ErrorEvent {
  fn from(error: &AppCoreError) -> Self {
    let (code, recoverable) = match error {
      AppCoreError::FailedToOpenStore(_) => ("failed_to_open_store", true),
      AppCoreError::ProjectNotFound => ("project_not_found", false),
      AppCoreError::SessionNotFound(_) => ("session_not_found", false),
      AppCoreError::IndexingNotEnabled => ("indexing_not_enabled", true),
      AppCoreError::CodexCredentialsNotFound => ("codex_credentials_not_found", false),
      AppCoreError::ClaudeCredentialsNotFound => ("claude_credentials_not_found", false),
      AppCoreError::PlanAlreadyInProgress => ("plan_already_in_progress", false),
      AppCoreError::PlanNotPending { .. } => ("plan_not_pending", false),
      AppCoreError::PlanStatusNotFound(_) => ("plan_status_not_found", false),
      AppCoreError::SessionAlreadyHasDifferentPlan { .. } => ("session_already_has_different_plan", false),
      AppCoreError::PlanAlreadyAttachedToDifferentSession { .. } => {
        ("plan_already_attached_to_different_session", false)
      }
      AppCoreError::PlanAttachedToDifferentSession { .. } => ("plan_attached_to_different_session", false),
      AppCoreError::PlanNotAttachedToSession { .. } => ("plan_not_attached_to_session", false),
    };

    ErrorEvent::new(ErrorCategory::Internal, code, error.to_string(), recoverable)
  }
}

impl From<AppCoreError> for ErrorEvent {
  fn from(error: AppCoreError) -> Self {
    ErrorEvent::from(&error)
  }
}

impl From<&ApiError> for ErrorEvent {
  fn from(error: &ApiError) -> Self {
    let (code, recoverable) = match error {
      ApiError::FailedToGetUser(_) => ("failed_to_get_user", true),
      ApiError::FailedToInitializeUser(_) => ("failed_to_initialize_user", true),
      ApiError::FailedToGetModels(_) => ("failed_to_get_models", true),
      ApiError::FailedToCreatePaymentIntent(_) => ("failed_to_create_payment_intent", true),
      ApiError::FailedToCreateCheckoutSession(_) => ("failed_to_create_checkout_session", true),
      ApiError::FailedToCreateBillingPortal(_) => ("failed_to_create_billing_portal", true),
      ApiError::FailedToListInvoices(_) => ("failed_to_list_invoices", true),
      ApiError::FailedToListPaymentMethods(_) => ("failed_to_list_payment_methods", true),
      ApiError::FailedToGetResponse(_) => ("failed_to_get_blprnt_response", true),
      ApiError::FailedToOpenStore(_) => ("failed_to_open_store", true),
      ApiError::FailedToGetCreditBalance(_) => ("failed_to_get_credit_balance", true),
      ApiError::FailedToSignIn(_) => ("failed_to_sign_in", true),
      ApiError::FailedToSignOut(_) => ("failed_to_sign_out", true),
    };

    ErrorEvent::new(ErrorCategory::Network, code, error.to_string(), recoverable)
  }
}

impl From<ApiError> for ErrorEvent {
  fn from(error: ApiError) -> Self {
    ErrorEvent::from(&error)
  }
}

// Generic fallback for anyhow::Error
impl From<&anyhow::Error> for ErrorEvent {
  fn from(error: &anyhow::Error) -> Self {
    // Try to downcast to known error types
    if let Some(e) = error.downcast_ref::<ProviderError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<ToolError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<EngineError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<SessionError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<OauthError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<AppCoreError>() {
      return ErrorEvent::from(e);
    }
    if let Some(e) = error.downcast_ref::<ApiError>() {
      return ErrorEvent::from(e);
    }

    // Fallback to internal error
    ErrorEvent::internal(error.to_string())
  }
}

impl From<anyhow::Error> for ErrorEvent {
  fn from(error: anyhow::Error) -> Self {
    ErrorEvent::from(&error)
  }
}
