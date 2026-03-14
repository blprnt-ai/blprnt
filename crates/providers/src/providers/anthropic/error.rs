//! Anthropic-specific error handling
//!
//! Anthropic returns errors in the format:
//! { "type": "error", "error": { "type": "...", "message": "..." } }

use common::errors::ProviderError;
use http::StatusCode;

/// Anthropic API error response
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicErrorResponse {
  #[serde(rename = "type")]
  pub kind:    Option<String>,
  pub error:   Option<AnthropicErrorDetail>,
  // Sometimes errors come at the top level
  pub message: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicErrorDetail {
  #[serde(rename = "type")]
  pub kind:    Option<String>,
  pub message: Option<String>,
}

impl AnthropicErrorResponse {
  /// Parse an error response from JSON text
  pub fn parse(text: &str) -> Option<Self> {
    serde_json::from_str(text).ok()
  }

  /// Convert to a ProviderError with user-friendly messages
  pub fn to_provider_error(self, status: StatusCode, model: Option<&str>) -> ProviderError {
    // Get the error details
    let (error_type, message) = if let Some(error) = self.error {
      (error.kind, error.message.or(self.message))
    } else {
      (self.kind, self.message)
    };

    let error_type_str = error_type.as_deref();
    let raw_message = message.clone().unwrap_or_else(|| "Unknown error".to_string());
    let model_str = model.unwrap_or("unknown");

    // First, try to match on error type
    if let Some(error) = Self::match_error_type(error_type_str, &raw_message, model_str) {
      return error;
    }

    // Fall back to HTTP status code
    Self::match_http_status(status, &raw_message, model_str)
  }

  fn match_error_type(error_type: Option<&str>, raw_message: &str, model: &str) -> Option<ProviderError> {
    match error_type? {
      // Authentication errors
      "authentication_error" => Some(ProviderError::InvalidApiKey {
        message: "Your Anthropic API key is invalid. Please check your settings.".into(),
      }),
      "permission_error" => Some(ProviderError::InvalidApiKey {
        message: "Permission denied. Your API key may not have access to this resource.".into(),
      }),

      // Rate limiting
      "rate_limit_error" => Some(ProviderError::RateLimit {
        context: "rate_limit".into(),
        message: "You've hit the rate limit. Please wait a moment and try again.".into(),
      }),

      // Credit/billing errors
      "insufficient_credit" | "billing_error" => Some(ProviderError::InsufficientCredits {
        code:    error_type.unwrap_or("billing").to_string(),
        message: "You've run out of API credits. Please check your Anthropic billing settings.".into(),
      }),

      // Overload errors
      "overloaded_error" => Some(ProviderError::ProviderUnavailable {
        provider: "Anthropic".into(),
        message:  "Anthropic's API is currently overloaded. Please try again in a moment.".into(),
      }),

      // Invalid request
      "invalid_request_error" => {
        // Check for specific error conditions in the message
        let msg_lower = raw_message.to_lowercase();

        if msg_lower.contains("context length")
          || msg_lower.contains("too many tokens")
          || msg_lower.contains("maximum")
        {
          return Some(ProviderError::ContextLengthExceeded {
            model:   model.to_string(),
            message: format!(
              "The conversation is too long for {}. Try starting a new conversation or reducing the context.",
              model
            ),
          });
        }

        if msg_lower.contains("model") && (msg_lower.contains("not found") || msg_lower.contains("does not exist")) {
          return Some(ProviderError::ModelNotFound {
            model:   model.to_string(),
            message: format!("The model '{}' was not found or is not available.", model),
          });
        }

        Some(ProviderError::BadRequest {
          context: "invalid_request".into(),
          message: Self::truncate_message(raw_message, 300),
        })
      }

      // Not found
      "not_found_error" => Some(ProviderError::ModelNotFound {
        model:   model.to_string(),
        message: format!("The requested resource was not found. Model: {}", model),
      }),

      // Server errors
      "api_error" | "internal_error" => Some(ProviderError::ServerError {
        code:    error_type.unwrap_or("api_error").to_string(),
        message: "Anthropic encountered an internal error. Please try again.".into(),
      }),

      // Request too large
      "request_too_large" => Some(ProviderError::ContextLengthExceeded {
        model:   model.to_string(),
        message: "The request is too large. Try reducing the conversation length or attachments.".into(),
      }),

      _ => None,
    }
  }

  fn match_http_status(status: StatusCode, raw_message: &str, model: &str) -> ProviderError {
    match status {
      StatusCode::BAD_REQUEST => {
        ProviderError::BadRequest { context: "bad_request".into(), message: Self::truncate_message(raw_message, 300) }
      }
      StatusCode::UNAUTHORIZED => ProviderError::InvalidApiKey {
        message: "Your Anthropic API key is invalid. Please check your settings.".into(),
      },
      StatusCode::PAYMENT_REQUIRED => ProviderError::InsufficientCredits {
        code:    "402".into(),
        message: "Payment required. Please check your Anthropic billing settings.".into(),
      },
      StatusCode::FORBIDDEN => ProviderError::ContentModeration {
        code:    "forbidden".into(),
        message: "Access denied. Your request may have been blocked.".into(),
      },
      StatusCode::NOT_FOUND => ProviderError::ModelNotFound {
        model:   model.to_string(),
        message: format!("The model '{}' was not found.", model),
      },
      StatusCode::REQUEST_TIMEOUT => {
        ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
      }
      StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimit {
        context: "rate_limit".into(),
        message: "Too many requests. Please wait and try again.".into(),
      },
      StatusCode::INTERNAL_SERVER_ERROR => ProviderError::ServerError {
        code:    "500".into(),
        message: "Anthropic server error. Please try again.".into(),
      },
      StatusCode::BAD_GATEWAY => ProviderError::ProviderUnavailable {
        provider: "Anthropic".into(),
        message:  "Anthropic is temporarily unavailable. Please try again.".into(),
      },
      StatusCode::SERVICE_UNAVAILABLE => ProviderError::ProviderUnavailable {
        provider: "Anthropic".into(),
        message:  "Anthropic is temporarily unavailable due to high demand. Please try again.".into(),
      },
      StatusCode::GATEWAY_TIMEOUT => {
        ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
      }
      _ => ProviderError::LlmUnknownError {
        context: format!("anthropic::{}", status.as_u16()),
        message: Self::truncate_message(raw_message, 300),
      },
    }
  }

  fn truncate_message(message: &str, max_len: usize) -> String {
    if message.len() <= max_len { message.to_string() } else { format!("{}...", &message[..max_len.saturating_sub(3)]) }
  }
}
