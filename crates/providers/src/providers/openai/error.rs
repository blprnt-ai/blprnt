//! OpenAI-specific error handling
//!
//! OpenAI returns errors in the format:
//! { "error": { "message", "type", "code", "param" } }
//! or directly as { "message", "type", "code", "param" }

use common::errors::ProviderError;
use http::StatusCode;
use regex::Regex;

use super::responses::fallback::FallbackError;
use super::responses::response::StreamError;

struct Limits {
  limit:     u32,
  used:      u32,
  requested: u32,
  try_after: f32,
}

/// OpenAI API error response
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiErrorResponse {
  pub error:   Option<OpenAiErrorDetail>,
  // Sometimes errors come at the top level
  #[serde(rename = "type")]
  pub kind:    Option<String>,
  pub code:    Option<String>,
  pub param:   Option<String>,
  pub message: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiErrorDetail {
  #[serde(rename = "type")]
  pub kind:    Option<String>,
  pub code:    Option<String>,
  pub param:   Option<String>,
  pub message: Option<String>,
}

impl OpenAiErrorResponse {
  /// Parse an error response from JSON text
  pub fn parse(text: &str) -> Option<Self> {
    serde_json::from_str(text).ok()
  }

  /// Convert to a ProviderError with user-friendly messages
  pub fn to_provider_error(self, status: StatusCode, model: Option<&str>) -> ProviderError {
    // Get the error details, either from nested error object or top-level
    let (message, code, kind, param) = if let Some(error) = self.error {
      (error.message.or(self.message), error.code.or(self.code), error.kind.or(self.kind), error.param.or(self.param))
    } else {
      (self.message, self.code, self.kind, self.param)
    };

    let code_str = code.as_deref();
    let kind_str = kind.as_deref();
    let param_str = param.as_deref();
    let raw_message = message.clone().unwrap_or_else(|| "Unknown error".to_string());
    let model_str = model.unwrap_or("unknown");

    // Check for streaming not supported error
    if Self::is_stream_error(kind_str, code_str, param_str) {
      return ProviderError::NotSupportedStreaming(
        "Streaming is not supported for this model. Falling back to non-streaming mode.".into(),
      );
    }

    // First, try to match on error code
    if let Some(error) = Self::match_error_code(code_str, &raw_message, model_str) {
      return error;
    }

    // Then, try to match on error type/kind
    if let Some(error) = Self::match_error_kind(kind_str, &raw_message) {
      return error;
    }

    // Finally, fall back to HTTP status code
    Self::match_http_status(status, &raw_message, model_str)
  }

  fn is_stream_error(kind: Option<&str>, code: Option<&str>, param: Option<&str>) -> bool {
    matches!(kind, Some("invalid_request_error"))
      && matches!(code, Some("unsupported_value"))
      && matches!(param, Some("stream"))
  }

  fn match_error_code(code: Option<&str>, raw_message: &str, model: &str) -> Option<ProviderError> {
    match code? {
      // Context and token errors
      "context_length_exceeded" | "max_tokens_exceeded" => Some(ProviderError::ContextLengthExceeded {
        model:   model.to_string(),
        message: format!(
          "The conversation is too long for {}. Try starting a new conversation or using a model with a larger context window.",
          model
        ),
      }),

      // Rate limiting and quota errors
      "rate_limit_exceeded" => {
        let limits = Self::parse_limits_from_message(raw_message);
        if let Some(limits) = limits {
          Some(ProviderError::RateLimit {
            context: "rate_limit_exceeded".into(),
            message: format!(
              "You've hit the rate limit. Limit: {}, Used: {}, Requested: {}. Try again in {} seconds.",
              limits.limit, limits.used, limits.requested, limits.try_after
            ),
          })
        } else {
          if raw_message.contains("temporarily rate-limited upstream") {
            Some(ProviderError::RateLimitUpstream {
              context: "rate_limit_exceeded_upstream".into(),
              message: "The upstream provider has temporarily rate-limited all requests. Please retry shortly.".into(),
            })
          } else {
            Some(ProviderError::RateLimit {
              context: "rate_limit_exceeded".into(),
              message: "You've hit the rate limit. Please wait a moment and try again.".into(),
            })
          }
        }
      }
      "insufficient_quota" => Some(ProviderError::InsufficientCredits {
        code:    "insufficient_quota".into(),
        message: "You've run out of API credits. Please check your OpenAI billing settings.".into(),
      }),
      "billing_hard_limit_reached" => Some(ProviderError::InsufficientCredits {
        code:    "billing_hard_limit_reached".into(),
        message: "You've reached your billing limit. Please update your OpenAI billing settings.".into(),
      }),

      // Auth errors
      "invalid_api_key" => Some(ProviderError::InvalidApiKey {
        message: "Your OpenAI API key is invalid. Please check your settings.".into(),
      }),

      // Model errors
      "model_not_found" => Some(ProviderError::ModelNotFound {
        model:   model.to_string(),
        message: format!("The model '{}' was not found or you don't have access to it.", model),
      }),

      // Content moderation
      "content_policy_violation" | "content_filter" => Some(ProviderError::ContentModeration {
        code:    code.unwrap_or("content_filter").to_string(),
        message: "Your message was flagged by content moderation. Please rephrase your request.".into(),
      }),

      // Server errors
      "server_error" | "internal_error" => Some(ProviderError::ServerError {
        code:    code.unwrap_or("server_error").to_string(),
        message: "OpenAI encountered an internal error. Please try again.".into(),
      }),

      // Bad request
      "invalid_request_error" | "invalid_value" => Some(ProviderError::BadRequest {
        context: code.unwrap_or("invalid_request").to_string(),
        message: Self::truncate_message(raw_message, 300),
      }),

      _ => None,
    }
  }

  fn match_error_kind(kind: Option<&str>, raw_message: &str) -> Option<ProviderError> {
    match kind? {
      "invalid_request_error" => Some(ProviderError::BadRequest {
        context: "invalid_request".into(),
        message: Self::truncate_message(raw_message, 300),
      }),
      "authentication_error" => Some(ProviderError::InvalidApiKey {
        message: "Authentication failed. Please check your OpenAI API key.".into(),
      }),
      "rate_limit_error" => Some(ProviderError::RateLimit {
        context: "rate_limit".into(),
        message: "Rate limit exceeded. Please slow down your requests.".into(),
      }),
      "usage_limit_reached" => Some(ProviderError::InsufficientCredits {
        code:    "usage_limit_reached".into(),
        message: "You've reached your usage limit. Please check your OpenAI account.".into(),
      }),
      "server_error" => Some(ProviderError::ServerError {
        code:    "server_error".into(),
        message: "OpenAI server error. Please try again.".into(),
      }),
      _ => None,
    }
  }

  fn match_http_status(status: StatusCode, raw_message: &str, model: &str) -> ProviderError {
    match status {
      StatusCode::BAD_REQUEST => {
        ProviderError::BadRequest { context: "bad_request".into(), message: Self::truncate_message(raw_message, 300) }
      }
      StatusCode::UNAUTHORIZED => {
        ProviderError::InvalidApiKey { message: "Your OpenAI API key is invalid. Please check your settings.".into() }
      }
      StatusCode::PAYMENT_REQUIRED => ProviderError::InsufficientCredits {
        code:    "402".into(),
        message: "Payment required. Please check your OpenAI billing settings.".into(),
      },
      StatusCode::FORBIDDEN => ProviderError::ContentModeration {
        code:    "forbidden".into(),
        message: "Access denied. Your request may have been blocked.".into(),
      },
      StatusCode::NOT_FOUND => ProviderError::ModelNotFound {
        model:   model.to_string(),
        message: format!("The model '{}' was not found.", model),
      },
      StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimit {
        context: "rate_limit".into(),
        message: "Too many requests. Please wait and try again.".into(),
      },
      StatusCode::INTERNAL_SERVER_ERROR => {
        ProviderError::ServerError { code: "500".into(), message: "OpenAI server error. Please try again.".into() }
      }
      StatusCode::BAD_GATEWAY => ProviderError::ProviderUnavailable {
        provider: "OpenAI".into(),
        message:  "OpenAI is temporarily unavailable. Please try again.".into(),
      },
      StatusCode::SERVICE_UNAVAILABLE => ProviderError::ProviderUnavailable {
        provider: "OpenAI".into(),
        message:  "OpenAI is temporarily unavailable. Please try again.".into(),
      },
      StatusCode::GATEWAY_TIMEOUT => {
        ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
      }
      _ => ProviderError::LlmUnknownError {
        context: format!("openai::{}", status.as_u16()),
        message: Self::truncate_message(raw_message, 300),
      },
    }
  }

  fn parse_limits_from_message(message: &str) -> Option<Limits> {
    let re = Regex::new(r"Limit\s+(\d+),\s+Used\s+(\d+),\s+Requested\s+(\d+).*?in\s+([0-9]+(?:\.[0-9]+)?)s")
      .expect("rate limit regex should be valid");
    let caps = re.captures(message)?;

    let limit = caps[1].parse::<u32>().ok()?;
    let used = caps[2].parse::<u32>().ok()?;
    let requested = caps[3].parse::<u32>().ok()?;
    let try_after = caps[4].parse::<f32>().ok()?;

    Some(Limits { limit, used, requested, try_after })
  }

  fn truncate_message(message: &str, max_len: usize) -> String {
    if message.len() <= max_len { message.to_string() } else { format!("{}...", &message[..max_len.saturating_sub(3)]) }
  }
}

// Legacy CodexError for backward compatibility
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CodexError {
  #[serde(skip)]
  pub url:         String,
  #[serde(skip)]
  pub http_status: StatusCode,
  #[serde(rename = "type")]
  pub kind:        Option<String>,
  pub code:        Option<String>,
  pub param:       Option<String>,
  pub message:     Option<String>,
}

impl CodexError {
  pub fn new(status: StatusCode, text: String) -> Option<Self> {
    let error = serde_json::from_str::<CodexError>(&text);
    match error {
      Ok(error) => Some(CodexError { http_status: status, ..error }),
      _ => None,
    }
  }

  pub fn from_fallback_error(url: &str, status: StatusCode, error: FallbackError) -> Option<Self> {
    Some(CodexError {
      url:         url.to_string(),
      http_status: status,
      code:        Some(error.code),
      param:       error.param,
      message:     Some(error.message),
      kind:        None,
    })
  }

  pub fn from_stream_error(url: &str, status: StatusCode, error: StreamError) -> Option<Self> {
    Some(CodexError {
      url:         url.to_string(),
      http_status: status,
      code:        Some(error.code),
      param:       error.param,
      message:     Some(error.message),
      kind:        None,
    })
  }

  pub fn to_provider_error(self) -> ProviderError {
    // Convert to new format and use that
    let response = OpenAiErrorResponse {
      error:   Some(OpenAiErrorDetail {
        kind:    self.kind.clone(),
        code:    self.code.clone(),
        param:   self.param.clone(),
        message: self.message.clone(),
      }),
      kind:    None,
      code:    None,
      param:   None,
      message: None,
    };
    response.to_provider_error(self.http_status, None)
  }
}
