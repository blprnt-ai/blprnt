//! OpenRouter-specific error handling
//!
//! OpenRouter can return errors in several formats:
//! 1. Standard OpenAI-compatible format: { "error": { "message", "type", "code" } }
//! 2. Direct error object: { "message", "code", "type" }
//! 3. OpenRouter-specific format: { "error": { "message", "code", "metadata": { ... } } }

use common::errors::ProviderError;
use http::StatusCode;

/// OpenRouter API error response
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterErrorResponse {
  pub error:   Option<OpenRouterErrorDetail>,
  // Sometimes errors come at the top level
  pub message: Option<String>,
  pub code:    Option<ErrorCodeValue>,
  #[serde(rename = "type")]
  pub kind:    Option<String>,
}

/// The error detail object
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterErrorDetail {
  pub message:  Option<String>,
  pub code:     Option<ErrorCodeValue>,
  #[serde(rename = "type")]
  pub kind:     Option<String>,
  pub param:    Option<String>,
  pub metadata: Option<OpenRouterErrorMetadata>,
}

/// Error code can be a string or number
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ErrorCodeValue {
  String(String),
  Number(i64),
}

impl ErrorCodeValue {
  pub fn as_str(&self) -> String {
    match self {
      ErrorCodeValue::String(s) => s.clone(),
      ErrorCodeValue::Number(n) => n.to_string(),
    }
  }
}

/// OpenRouter-specific metadata in errors
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct OpenRouterErrorMetadata {
  pub provider_name: Option<String>,
  pub model_id:      Option<String>,
  pub raw:           Option<serde_json::Value>,
}

impl OpenRouterErrorResponse {
  /// Parse an error response from JSON text
  pub fn parse(text: &str) -> Option<Self> {
    serde_json::from_str(text).ok()
  }

  /// Convert to a ProviderError with user-friendly messages
  pub fn to_provider_error(&self, status: StatusCode, model: Option<&str>) -> ProviderError {
    tracing::error!("To provider error: {:?}, {:?}, {:?}", status, model, self);

    // Get the error details, either from nested error object or top-level
    let (code, kind, metadata) = if let Some(error) = self.error.clone() {
      (error.code.or(self.code.clone()), error.kind.or(self.kind.clone()), error.metadata)
    } else {
      (self.code.clone(), self.kind.clone(), None)
    };

    let code_str = code.as_ref().map(|c| c.as_str());
    let raw_message = self.error.as_ref().and_then(|e| e.metadata.as_ref()).and_then(|m| m.raw.clone());
    let raw_message =
      raw_message.map(|r| r.as_str().unwrap_or("Unknown error").to_string()).unwrap_or("Unknown error".to_string());

    let model_str = model.unwrap_or("unknown");
    let provider = metadata.as_ref().and_then(|m| m.provider_name.clone()).unwrap_or_else(|| "provider".to_string());

    // First, try to match on error code
    if let Some(code) = &code_str
      && let Some(error) = Self::match_error_code(code, &raw_message, model_str, &provider)
    {
      return error;
    }

    // Then, try to match on error type/kind
    if let Some(kind) = &kind
      && let Some(error) = Self::match_error_kind(kind, &raw_message, model_str)
    {
      return error;
    }

    // Finally, fall back to HTTP status code
    Self::match_http_status(status, &raw_message, model_str, &provider)
  }

  fn match_error_code(code: &str, raw_message: &str, model: &str, provider: &str) -> Option<ProviderError> {
    let code_lower = code.to_lowercase();

    tracing::error!("Match error code: {:?}, {:?}, {:?}, {:?}", code_lower, raw_message, model, provider);

    match code_lower.as_str() {
      // Rate limiting and quota errors
      "rate_limit_exceeded" | "429" => {
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
      "insufficient_quota" | "insufficient_credits" | "credits_exhausted" => Some(ProviderError::InsufficientCredits {
        code:    code.to_string(),
        message: "You've run out of credits.".into(),
      }),
      "billing_not_active" | "payment_required" | "402" => Some(ProviderError::InsufficientCredits {
        code:    code.to_string(),
        message: "Plan is out of credits.".into(),
      }),

      // Context and token errors
      "context_length_exceeded" | "max_tokens_exceeded" => Some(ProviderError::ContextLengthExceeded {
        model:   model.to_string(),
        message: format!(
          "The conversation is too long for {}. Try starting a new conversation or using a model with a larger context window.",
          model
        ),
      }),

      // Model errors
      "model_not_found" | "invalid_model" | "404" => Some(ProviderError::ModelNotFound {
        model:   model.to_string(),
        message: format!(
          "The model '{}' was not found. It may have been removed or renamed.",
          model.split_once('/').unwrap_or(("", model)).1
        ),
      }),
      "model_unavailable" | "model_overloaded" => Some(ProviderError::ModelUnavailable {
        model:   model.to_string(),
        message: format!(
          "The model '{}' is temporarily unavailable. Please try again in a moment or select a different model.",
          model
        ),
      }),

      // Auth errors
      "invalid_api_key" | "authentication_error" | "401" => {
        Some(ProviderError::InvalidApiKey { message: "Your API key is invalid or has been revoked.".into() })
      }

      // Content moderation
      "content_filter" | "content_policy_violation" | "moderation_required" | "flagged" => {
        Some(ProviderError::ContentModeration {
          code:    code.to_string(),
          message: "Your message was flagged by content moderation. Please rephrase your request.".into(),
        })
      }

      // Provider/upstream errors
      "provider_error" | "upstream_error" => Some(ProviderError::ProviderUnavailable {
        provider: provider.to_string(),
        message:  format!(
          "The upstream provider ({}) encountered an error: {}",
          provider,
          Self::truncate_message(raw_message, 200)
        ),
      }),

      // Server errors
      "internal_error" | "server_error" | "500" => Some(ProviderError::ServerError {
        code:    code.to_string(),
        message: "blprnt encountered an internal error. Please try again.".into(),
      }),
      "bad_gateway" | "502" => Some(ProviderError::ProviderUnavailable {
        provider: provider.to_string(),
        message:  format!("The upstream provider ({}) is temporarily unavailable. Please try again.", provider),
      }),
      "service_unavailable" | "503" => Some(ProviderError::ProviderUnavailable {
        provider: provider.to_string(),
        message:  "The service is temporarily unavailable. Please try again in a moment.".into(),
      }),
      "gateway_timeout" | "timeout" | "504" => Some(ProviderError::GatewayTimeout {
        message: "The request timed out. The model may be overloaded. Please try again.".into(),
      }),

      // Bad request
      "invalid_request" | "invalid_request_error" | "bad_request" | "400" => {
        Some(ProviderError::BadRequest { context: code.to_string(), message: Self::truncate_message(raw_message, 300) })
      }

      _ => None,
    }
  }

  fn match_error_kind(kind: &str, raw_message: &str, model: &str) -> Option<ProviderError> {
    let kind_lower = kind.to_lowercase();

    match kind_lower.as_str() {
      "invalid_request_error" => Some(ProviderError::BadRequest {
        context: "invalid_request".into(),
        message: Self::truncate_message(raw_message, 300),
      }),
      "authentication_error" => {
        Some(ProviderError::InvalidApiKey { message: "Authentication failed. Please check your API key.".into() })
      }
      "rate_limit_error" => Some(ProviderError::RateLimit {
        context: "rate_limit".into(),
        message: "Rate limit exceeded. Please slow down your requests.".into(),
      }),
      "usage_limit_reached" => Some(ProviderError::InsufficientCredits {
        code:    "usage_limit_reached".into(),
        message: "You've reached your usage limit. Please check your account.".into(),
      }),
      "context_length_exceeded" => Some(ProviderError::ContextLengthExceeded {
        model:   model.to_string(),
        message: format!("The conversation is too long for {}.", model),
      }),
      _ => None,
    }
  }

  fn match_http_status(status: StatusCode, raw_message: &str, model: &str, provider: &str) -> ProviderError {
    tracing::error!("Provider error: {:?}, {:?}, {:?}, {:?}", status, raw_message, model, provider);

    match status {
      StatusCode::BAD_REQUEST => {
        ProviderError::BadRequest { context: "bad_request".into(), message: Self::truncate_message(raw_message, 300) }
      }
      StatusCode::UNAUTHORIZED => ProviderError::InvalidApiKey { message: "Your API key is invalid.".into() },
      StatusCode::PAYMENT_REQUIRED => {
        ProviderError::InsufficientCredits { code: "402".into(), message: "Payment required.".into() }
      }
      StatusCode::FORBIDDEN => ProviderError::ContentModeration {
        code:    "forbidden".into(),
        message: "Access denied. Your request may have been blocked by content moderation.".into(),
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
        ProviderError::ServerError { code: "500".into(), message: "Server error. Please try again.".into() }
      }
      StatusCode::BAD_GATEWAY => ProviderError::ProviderUnavailable {
        provider: provider.to_string(),
        message:  format!("The upstream provider ({}) is temporarily unavailable.", provider),
      },
      StatusCode::SERVICE_UNAVAILABLE => ProviderError::ProviderUnavailable {
        provider: provider.to_string(),
        message:  "The service is temporarily unavailable. Please try again.".into(),
      },
      StatusCode::GATEWAY_TIMEOUT => {
        ProviderError::GatewayTimeout { message: "The request timed out. Please try again.".into() }
      }
      _ => ProviderError::LlmUnknownError {
        context: format!("blprnt:{}", status.as_u16()),
        message: Self::truncate_message(raw_message, 300),
      },
    }
  }

  /// Truncate a message to a maximum length, adding ellipsis if needed
  fn truncate_message(message: &str, max_len: usize) -> String {
    if message.len() <= max_len { message.to_string() } else { format!("{}...", &message[..max_len.saturating_sub(3)]) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_openrouter_error() -> anyhow::Result<()> {
    let json = r#"{"error":{"message":"Rate limit exceeded","code":"rate_limit_exceeded","type":"rate_limit_error"}}"#;
    let parsed = OpenRouterErrorResponse::parse(json).ok_or_else(|| anyhow::anyhow!("missing error response"))?;
    let error = parsed.to_provider_error(StatusCode::TOO_MANY_REQUESTS, Some("gpt-4"));
    assert!(matches!(error, ProviderError::RateLimit { .. }));
    Ok(())
  }

  #[test]
  fn test_parse_insufficient_credits() -> anyhow::Result<()> {
    let json = r#"{"error":{"message":"You have insufficient credits","code":"insufficient_credits"}}"#;
    let parsed = OpenRouterErrorResponse::parse(json).ok_or_else(|| anyhow::anyhow!("missing error response"))?;
    let error = parsed.to_provider_error(StatusCode::PAYMENT_REQUIRED, None);
    assert!(matches!(error, ProviderError::InsufficientCredits { .. }));
    Ok(())
  }

  #[test]
  fn test_parse_model_not_found() -> anyhow::Result<()> {
    let json = r#"{"error":{"message":"Model not found","code":"model_not_found"}}"#;
    let parsed = OpenRouterErrorResponse::parse(json).ok_or_else(|| anyhow::anyhow!("missing error response"))?;
    let error = parsed.to_provider_error(StatusCode::NOT_FOUND, Some("unknown-model"));
    assert!(matches!(error, ProviderError::ModelNotFound { .. }));
    Ok(())
  }
}
