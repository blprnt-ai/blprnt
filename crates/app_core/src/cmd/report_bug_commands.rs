use common::api::ApiClient;
pub use common::api::ReportBugAttachmentPayloadKind;
pub use common::api::ReportBugErrorCategory;
pub use common::api::ReportBugFieldError;
pub use common::api::ReportBugNormalizedSubmission;
pub use common::api::ReportBugPastedAttachmentKind;
pub use common::api::ReportBugPastedAttachmentPayload;
pub use common::api::ReportBugScreenshotContract;
pub use common::api::ReportBugScreenshotKind;
pub use common::api::ReportBugScreenshotPayload;
pub use common::api::ReportBugSeverity;
use common::api::ReportBugSubmitRequest;
use common::api::ReportBugSubmitResponse;
use common::errors::TauriResult;
use common::report_bug::map_submit_report_bug_api_failure;
use common::report_bug::normalize_submission;
use common::report_bug::report_bug_screenshot_contract;
use common::report_bug::validate_submit_request;
use common::report_bug::validation_rejection_response;

#[tauri::command]
#[specta::specta]
pub async fn report_bug_submit(request: ReportBugSubmitRequest) -> TauriResult<ReportBugSubmitResponse> {
  let screenshot_contract = report_bug_screenshot_contract();

  let field_errors = validate_submit_request(&request, &screenshot_contract);
  if !field_errors.is_empty() {
    return Ok(validation_rejection_response(field_errors, screenshot_contract));
  }

  let normalized_submission = normalize_submission(&request);

  let api_response = match ApiClient::get().submit_report_bug(request).await {
    Ok(response) => response,
    Err(error) => {
      tracing::error!("Failed to submit report bug: {}", error);
      return Ok(map_submit_report_bug_api_failure(error, screenshot_contract, Some(normalized_submission)));
    }
  };

  Ok(api_response)
}
