use base64::Engine;
use url::Url;

use crate::api::{
  ReportBugAttachmentPayloadKind,
  ReportBugErrorCategory,
  ReportBugErrorCode,
  ReportBugFieldError,
  ReportBugNormalizedSubmission,
  ReportBugPastedAttachmentPayload,
  ReportBugScreenshotContract,
  ReportBugScreenshotKind,
  ReportBugScreenshotPayload,
  ReportBugSeverity,
  ReportBugSubmitError,
  ReportBugSubmitRequest,
  ReportBugSubmitResponse,
  ReportBugSubmitState,
};
use crate::errors::ApiError;

const REPORT_BUG_TITLE_MAX_CHARS: usize = 180;
const REPORT_BUG_DESCRIPTION_MAX_CHARS: usize = 10_000;
const REPORT_BUG_SCREENSHOT_MAX_BYTES: u64 = 5 * 1024 * 1024;
const REPORT_BUG_ATTACHMENT_MAX_BYTES: u64 = 25 * 1024 * 1024;
const REPORT_BUG_SCREENSHOT_ALLOWED_MIME_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/webp", "image/gif"];
const REPORT_BUG_SCREENSHOT_ALLOWED_REFERENCE_SCHEMES: [&str; 1] = ["https"];
const REPORT_BUG_ATTACHMENT_ALLOWED_REFERENCE_SCHEMES: [&str; 1] = ["https"];
const REPORT_BUG_SCREENSHOT_INLINE_FALLBACK_NOTE: &str = "Screenshot omitted: inline screenshot upload is not configured; provide screenshot.reference_url to include an image in GitHub Projects content.";

#[derive(Debug, Eq, PartialEq)]
enum ReportBugScreenshotStrategyOutcome {
  None,
  ReferenceMarkdown { markdown: String },
  InlineFallback { note: String },
}

pub fn report_bug_screenshot_contract() -> ReportBugScreenshotContract {
  ReportBugScreenshotContract {
    max_bytes:                 REPORT_BUG_SCREENSHOT_MAX_BYTES,
    allowed_mime_types:        REPORT_BUG_SCREENSHOT_ALLOWED_MIME_TYPES.iter().map(|value| value.to_string()).collect(),
    allowed_reference_schemes: REPORT_BUG_SCREENSHOT_ALLOWED_REFERENCE_SCHEMES
      .iter()
      .map(|value| value.to_string())
      .collect(),
    supported_kinds:           vec![ReportBugScreenshotKind::InlineBase64, ReportBugScreenshotKind::ReferenceUrl],
  }
}

pub fn validate_submit_request(
  request: &ReportBugSubmitRequest,
  screenshot_contract: &ReportBugScreenshotContract,
) -> Vec<ReportBugFieldError> {
  let mut field_errors = Vec::new();

  let normalized_title = request.title.trim();
  if normalized_title.is_empty() {
    field_errors.push(required_field_error("title"));
  } else if normalized_title.chars().count() > REPORT_BUG_TITLE_MAX_CHARS {
    field_errors.push(length_field_error("title", format!("Title must be at most {REPORT_BUG_TITLE_MAX_CHARS} characters.")));
  }

  let normalized_description = request.description.trim();
  if normalized_description.is_empty() {
    field_errors.push(required_field_error("description"));
  } else if normalized_description.chars().count() > REPORT_BUG_DESCRIPTION_MAX_CHARS {
    field_errors.push(length_field_error(
      "description",
      format!("Description must be at most {REPORT_BUG_DESCRIPTION_MAX_CHARS} characters."),
    ));
  }

  if let Some(screenshot) = &request.screenshot {
    validate_screenshot_payload(screenshot, screenshot_contract, &mut field_errors);
  }

  validate_pasted_attachments(&request.pasted_attachments, &mut field_errors);

  field_errors
}

pub fn validation_rejection_details(
  field_errors: &[ReportBugFieldError],
) -> (ReportBugErrorCode, ReportBugErrorCategory, &'static str) {
  let has_screenshot_error = field_errors.iter().any(|error| error.field.starts_with("screenshot"));
  let has_attachment_error = field_errors.iter().any(|error| error.field.starts_with("pasted_attachments"));
  if has_attachment_error {
    (
      ReportBugErrorCode::AttachmentContractViolation,
      ReportBugErrorCategory::AttachmentUpload,
      "Attachment payload does not satisfy contract requirements.",
    )
  } else if has_screenshot_error {
    (
      ReportBugErrorCode::ScreenshotContractViolation,
      ReportBugErrorCategory::Screenshot,
      "Screenshot payload does not satisfy contract requirements.",
    )
  } else {
    (
      ReportBugErrorCode::ValidationFailed,
      ReportBugErrorCategory::Validation,
      "Bug report request failed validation.",
    )
  }
}

pub fn validation_rejection_response(
  field_errors: Vec<ReportBugFieldError>,
  screenshot_contract: ReportBugScreenshotContract,
) -> ReportBugSubmitResponse {
  let (code, category, message) = validation_rejection_details(&field_errors);
  ReportBugSubmitResponse {
    state: ReportBugSubmitState::Rejected,
    normalized_submission: None,
    error: Some(ReportBugSubmitError {
      code,
      category,
      message: message.to_string(),
      retryable: false,
      field_errors,
    }),
    screenshot_contract,
  }
}

pub fn github_auth_failure_response(
  screenshot_contract: ReportBugScreenshotContract,
  normalized_submission: ReportBugNormalizedSubmission,
) -> ReportBugSubmitResponse {
  ReportBugSubmitResponse {
    state: ReportBugSubmitState::Rejected,
    normalized_submission: Some(normalized_submission),
    error: Some(github_submit_error(
      ReportBugErrorCode::GithubAuthFailed,
      "Bug report authentication failed.",
      false,
    )),
    screenshot_contract,
  }
}

pub fn normalize_submission(request: &ReportBugSubmitRequest) -> ReportBugNormalizedSubmission {
  ReportBugNormalizedSubmission {
    title:              normalize_report_bug_title(&request.title, request.severity),
    description:        normalize_report_bug_description(request.description.trim(), request.screenshot.as_ref()),
    severity:           request.severity,
    screenshot:         request.screenshot.clone(),
    pasted_attachments: normalize_pasted_attachments(&request.pasted_attachments),
  }
}

pub fn normalize_report_bug_description(description: &str, screenshot: Option<&ReportBugScreenshotPayload>) -> String {
  match screenshot_strategy_outcome(screenshot) {
    ReportBugScreenshotStrategyOutcome::None => description.to_string(),
    ReportBugScreenshotStrategyOutcome::ReferenceMarkdown { markdown } => format!("{description}\n\n{markdown}"),
    ReportBugScreenshotStrategyOutcome::InlineFallback { note } => format!("{description}\n\n> {note}"),
  }
}

pub fn normalize_pasted_attachments(attachments: &[ReportBugPastedAttachmentPayload]) -> Vec<ReportBugPastedAttachmentPayload> {
  attachments
    .iter()
    .map(|attachment| ReportBugPastedAttachmentPayload {
      kind:          attachment.kind,
      file_name:     attachment.file_name.trim().to_string(),
      mime_type:     attachment.mime_type.trim().to_string(),
      byte_len:      attachment.byte_len,
      payload_kind:  attachment.payload_kind,
      data_base64:   attachment
        .data_base64
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()),
      file_path:     attachment
        .file_path
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()),
      reference_url: attachment
        .reference_url
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()),
    })
    .collect()
}

pub fn map_submit_report_bug_api_failure(
  error: anyhow::Error,
  screenshot_contract: ReportBugScreenshotContract,
  normalized_submission: Option<ReportBugNormalizedSubmission>,
) -> ReportBugSubmitResponse {
  let message = if let Some(ApiError::FailedToSubmitReportBug(message)) = error.downcast_ref::<ApiError>() {
    message.clone()
  } else {
    error.to_string()
  };

  if let Ok(response) = serde_json::from_str::<ReportBugSubmitResponse>(&message) {
    return response;
  }

  let message_lower = message.to_ascii_lowercase();
  let fallback_error = if message_lower.contains("rate limit") {
    github_submit_error(ReportBugErrorCode::GithubRateLimited, "Bug report service is rate limited.", true)
  } else if message_lower.contains("unauthorized") || message_lower.contains("401") {
    github_submit_error(ReportBugErrorCode::GithubAuthFailed, "Bug report authentication failed.", false)
  } else if message_lower.contains("forbidden") || message_lower.contains("403") {
    github_submit_error(ReportBugErrorCode::GithubPermissionDenied, "Bug report permission denied.", false)
  } else if message_lower.contains("not found") || message_lower.contains("404") {
    github_submit_error(ReportBugErrorCode::GithubNotFound, "Bug report destination was not found.", false)
  } else if message_lower.contains("timed out")
    || message_lower.contains("timeout")
    || message_lower.contains("connection")
    || message_lower.contains("network")
  {
    github_submit_error(ReportBugErrorCode::GithubNetworkError, "Network error while submitting bug report.", true)
  } else {
    github_submit_error(ReportBugErrorCode::GithubApiError, "Bug report service returned an error.", true)
  };

  ReportBugSubmitResponse {
    state: ReportBugSubmitState::Rejected,
    normalized_submission,
    error: Some(fallback_error),
    screenshot_contract,
  }
}

pub fn github_submit_error(code: ReportBugErrorCode, message: &str, retryable: bool) -> ReportBugSubmitError {
  ReportBugSubmitError {
    code,
    category: ReportBugErrorCategory::Github,
    message: message.to_string(),
    retryable,
    field_errors: vec![],
  }
}

pub fn normalize_report_bug_title(title: &str, severity: ReportBugSeverity) -> String {
  let normalized_title = strip_severity_prefix(title);
  format!("[{}] {}", severity.label(), normalized_title)
}

impl ReportBugSeverity {
  const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Critical];

  fn label(self) -> &'static str {
    match self {
      Self::Low => "LOW",
      Self::Medium => "MEDIUM",
      Self::High => "HIGH",
      Self::Critical => "CRITICAL",
    }
  }
}

fn strip_severity_prefix(title: &str) -> String {
  let title = title.trim();
  for severity in ReportBugSeverity::ALL {
    let prefix = format!("[{}]", severity.label());
    if title.get(..prefix.len()).is_some_and(|value| value.eq_ignore_ascii_case(&prefix)) {
      return title[prefix.len()..].trim_start().to_string();
    }
  }
  title.to_string()
}

fn validate_pasted_attachments(attachments: &[ReportBugPastedAttachmentPayload], field_errors: &mut Vec<ReportBugFieldError>) {
  for (index, attachment) in attachments.iter().enumerate() {
    validate_single_pasted_attachment(index, attachment, field_errors);
  }
}

fn validate_single_pasted_attachment(
  index: usize,
  attachment: &ReportBugPastedAttachmentPayload,
  field_errors: &mut Vec<ReportBugFieldError>,
) {
  let field_prefix = format!("pasted_attachments[{index}]");

  if attachment.file_name.trim().is_empty() {
    field_errors.push(required_field_error(&format!("{field_prefix}.file_name")));
  }

  if attachment.mime_type.trim().is_empty() || !attachment.mime_type.contains('/') {
    field_errors.push(format_field_error(
      &format!("{field_prefix}.mime_type"),
      "Attachment MIME type must be a valid MIME value.",
    ));
  }

  if attachment.byte_len == 0 || attachment.byte_len > REPORT_BUG_ATTACHMENT_MAX_BYTES {
    field_errors.push(attachment_size_field_error(&format!("{field_prefix}.byte_len")));
  }

  match attachment.payload_kind {
    ReportBugAttachmentPayloadKind::InlineBase64 => {
      if attachment.data_base64.as_ref().is_none_or(|value| value.trim().is_empty()) {
        field_errors.push(required_field_error(&format!("{field_prefix}.data_base64")));
      } else {
        validate_inline_attachment_data(index, attachment, field_errors);
      }

      if attachment.file_path.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.file_path"),
          "file_path must be omitted for inline_base64 attachments.",
        ));
      }

      if attachment.reference_url.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.reference_url"),
          "reference_url must be omitted for inline_base64 attachments.",
        ));
      }
    }
    ReportBugAttachmentPayloadKind::FileReference => {
      if attachment.file_path.as_ref().is_none_or(|value| value.trim().is_empty()) {
        field_errors.push(required_field_error(&format!("{field_prefix}.file_path")));
      }

      if attachment.data_base64.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.data_base64"),
          "data_base64 must be omitted for file_reference attachments.",
        ));
      }

      if attachment.reference_url.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.reference_url"),
          "reference_url must be omitted for file_reference attachments.",
        ));
      }
    }
    ReportBugAttachmentPayloadKind::ReferenceUrl => {
      if attachment.reference_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        field_errors.push(required_field_error(&format!("{field_prefix}.reference_url")));
      }

      if attachment.data_base64.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.data_base64"),
          "data_base64 must be omitted for reference_url attachments.",
        ));
      }

      if attachment.file_path.is_some() {
        field_errors.push(format_field_error(
          &format!("{field_prefix}.file_path"),
          "file_path must be omitted for reference_url attachments.",
        ));
      }

      if let Some(reference_url) = &attachment.reference_url {
        validate_attachment_reference_url(index, reference_url, field_errors);
      }
    }
  }
}

fn validate_inline_attachment_data(
  index: usize,
  attachment: &ReportBugPastedAttachmentPayload,
  field_errors: &mut Vec<ReportBugFieldError>,
) {
  let Some(data_base64) = attachment.data_base64.as_ref() else {
    return;
  };

  let field_prefix = format!("pasted_attachments[{index}]");
  let decoded = match base64::engine::general_purpose::STANDARD.decode(data_base64.trim()) {
    Ok(decoded) => decoded,
    Err(_) => {
      field_errors.push(format_field_error(
        &format!("{field_prefix}.data_base64"),
        "Attachment inline payload must be valid base64.",
      ));
      return;
    }
  };

  let decoded_len = decoded.len() as u64;
  if decoded_len == 0 || decoded_len > REPORT_BUG_ATTACHMENT_MAX_BYTES {
    field_errors.push(attachment_size_field_error(&format!("{field_prefix}.byte_len")));
  }

  if decoded_len != attachment.byte_len {
    field_errors.push(format_field_error(
      &format!("{field_prefix}.byte_len"),
      "Attachment byte_len does not match decoded inline payload size.",
    ));
  }
}

fn validate_attachment_reference_url(index: usize, reference_url: &str, field_errors: &mut Vec<ReportBugFieldError>) {
  let field = format!("pasted_attachments[{index}].reference_url");
  let parsed_url = match Url::parse(reference_url.trim()) {
    Ok(url) => url,
    Err(_) => {
      field_errors.push(format_field_error(&field, "Attachment reference URL must be a valid URL."));
      return;
    }
  };

  if !REPORT_BUG_ATTACHMENT_ALLOWED_REFERENCE_SCHEMES
    .iter()
    .any(|scheme| scheme.eq_ignore_ascii_case(parsed_url.scheme()))
  {
    field_errors.push(format_field_error(&field, "Attachment reference URL scheme is not allowed."));
  }
}

fn validate_screenshot_payload(
  screenshot: &ReportBugScreenshotPayload,
  screenshot_contract: &ReportBugScreenshotContract,
  field_errors: &mut Vec<ReportBugFieldError>,
) {
  if screenshot.file_name.trim().is_empty() {
    field_errors.push(required_field_error("screenshot.file_name"));
  }

  if !screenshot_contract
    .allowed_mime_types
    .iter()
    .any(|allowed| allowed.eq_ignore_ascii_case(screenshot.mime_type.trim()))
  {
    field_errors.push(format_field_error("screenshot.mime_type", "Screenshot MIME type is not supported."));
  }

  if screenshot.byte_len == 0 || screenshot.byte_len > screenshot_contract.max_bytes {
    field_errors.push(range_field_error(
      "screenshot.byte_len",
      format!("Screenshot size must be between 1 and {} bytes.", screenshot_contract.max_bytes),
    ));
  }

  match screenshot.kind {
    ReportBugScreenshotKind::InlineBase64 => {
      if screenshot.data_base64.as_ref().is_none_or(|value| value.trim().is_empty()) {
        field_errors.push(required_field_error("screenshot.data_base64"));
      } else {
        validate_inline_screenshot_data(screenshot, screenshot_contract, field_errors);
      }

      if screenshot.reference_url.is_some() {
        field_errors.push(format_field_error(
          "screenshot.reference_url",
          "Reference URL must be omitted for inline_base64 screenshots.",
        ));
      }
    }
    ReportBugScreenshotKind::ReferenceUrl => {
      if screenshot.reference_url.as_ref().is_none_or(|value| value.trim().is_empty()) {
        field_errors.push(required_field_error("screenshot.reference_url"));
      }

      if screenshot.data_base64.is_some() {
        field_errors.push(format_field_error(
          "screenshot.data_base64",
          "Inline base64 payload must be omitted for reference_url screenshots.",
        ));
      }

      if let Some(reference_url) = &screenshot.reference_url {
        validate_screenshot_reference_url(reference_url, screenshot_contract, field_errors);
      }
    }
  }
}

fn validate_inline_screenshot_data(
  screenshot: &ReportBugScreenshotPayload,
  screenshot_contract: &ReportBugScreenshotContract,
  field_errors: &mut Vec<ReportBugFieldError>,
) {
  let Some(data_base64) = screenshot.data_base64.as_ref() else {
    return;
  };

  let decoded = match base64::engine::general_purpose::STANDARD.decode(data_base64.trim()) {
    Ok(decoded) => decoded,
    Err(_) => {
      field_errors.push(format_field_error("screenshot.data_base64", "Screenshot inline payload must be valid base64."));
      return;
    }
  };

  if decoded.is_empty() {
    field_errors.push(range_field_error(
      "screenshot.byte_len",
      format!("Screenshot size must be between 1 and {} bytes.", screenshot_contract.max_bytes),
    ));
  }

  let decoded_len = decoded.len() as u64;
  if decoded_len > screenshot_contract.max_bytes {
    field_errors.push(range_field_error(
      "screenshot.byte_len",
      format!("Screenshot size must be between 1 and {} bytes.", screenshot_contract.max_bytes),
    ));
  }

  if decoded_len != screenshot.byte_len {
    field_errors.push(format_field_error(
      "screenshot.byte_len",
      "Screenshot byte_len does not match decoded inline payload size.",
    ));
  }
}

fn screenshot_strategy_outcome(screenshot: Option<&ReportBugScreenshotPayload>) -> ReportBugScreenshotStrategyOutcome {
  let Some(screenshot) = screenshot else {
    return ReportBugScreenshotStrategyOutcome::None;
  };

  match screenshot.kind {
    ReportBugScreenshotKind::ReferenceUrl => {
      let Some(reference_url) = screenshot.reference_url.as_ref() else {
        return ReportBugScreenshotStrategyOutcome::InlineFallback {
          note: REPORT_BUG_SCREENSHOT_INLINE_FALLBACK_NOTE.to_string(),
        };
      };

      let file_name = screenshot.file_name.trim();
      let alt_text = if file_name.is_empty() { "screenshot" } else { file_name };
      ReportBugScreenshotStrategyOutcome::ReferenceMarkdown {
        markdown: format!("![{alt_text}]({})", reference_url.trim()),
      }
    }
    ReportBugScreenshotKind::InlineBase64 => ReportBugScreenshotStrategyOutcome::InlineFallback {
      note: REPORT_BUG_SCREENSHOT_INLINE_FALLBACK_NOTE.to_string(),
    },
  }
}

fn validate_screenshot_reference_url(
  reference_url: &str,
  screenshot_contract: &ReportBugScreenshotContract,
  field_errors: &mut Vec<ReportBugFieldError>,
) {
  let parsed_url = match Url::parse(reference_url.trim()) {
    Ok(url) => url,
    Err(_) => {
      field_errors.push(format_field_error("screenshot.reference_url", "Screenshot reference URL must be a valid URL."));
      return;
    }
  };

  if !screenshot_contract
    .allowed_reference_schemes
    .iter()
    .any(|scheme| scheme.eq_ignore_ascii_case(parsed_url.scheme()))
  {
    field_errors.push(format_field_error("screenshot.reference_url", "Screenshot reference URL scheme is not allowed."));
  }
}

fn required_field_error(field: &str) -> ReportBugFieldError {
  ReportBugFieldError {
    field:   field.to_string(),
    code:    "required".to_string(),
    message: "Field is required.".to_string(),
  }
}

fn length_field_error(field: &str, message: String) -> ReportBugFieldError {
  ReportBugFieldError { field: field.to_string(), code: "length".to_string(), message }
}

fn range_field_error(field: &str, message: String) -> ReportBugFieldError {
  ReportBugFieldError { field: field.to_string(), code: "range".to_string(), message }
}

fn attachment_size_field_error(field: &str) -> ReportBugFieldError {
  ReportBugFieldError {
    field:   field.to_string(),
    code:    "attachment_max_bytes".to_string(),
    message: format!("Attachment size must be between 1 and {REPORT_BUG_ATTACHMENT_MAX_BYTES} bytes."),
  }
}

fn format_field_error(field: &str, message: &str) -> ReportBugFieldError {
  ReportBugFieldError { field: field.to_string(), code: "format".to_string(), message: message.to_string() }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalize_title_replaces_existing_prefix() {
    let output = normalize_report_bug_title(" [high] Old title ", ReportBugSeverity::Low);
    assert_eq!(output, "[LOW] Old title");
  }

  #[test]
  fn validation_rejection_prioritizes_attachment_errors() {
    let field_errors = vec![ReportBugFieldError {
      field: "pasted_attachments[0].byte_len".to_string(),
      code: "range".to_string(),
      message: "bad".to_string(),
    }];
    let (code, category, message) = validation_rejection_details(&field_errors);
    assert_eq!(code, ReportBugErrorCode::AttachmentContractViolation);
    assert_eq!(category, ReportBugErrorCategory::AttachmentUpload);
    assert_eq!(message, "Attachment payload does not satisfy contract requirements.");
  }

  #[test]
  fn normalize_description_appends_reference_markdown() {
    let screenshot = ReportBugScreenshotPayload {
      kind: ReportBugScreenshotKind::ReferenceUrl,
      file_name: "snap.png".to_string(),
      mime_type: "image/png".to_string(),
      byte_len: 1,
      data_base64: None,
      reference_url: Some("https://example.com/snap.png".to_string()),
    };
    let output = normalize_report_bug_description("body", Some(&screenshot));
    assert_eq!(output, "body\n\n![snap.png](https://example.com/snap.png)");
  }
}