use std::fmt::Debug;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use http::header::CONTENT_TYPE;
use serde::Deserialize;
use tauri_plugin_store::StoreExt;

use crate::blprnt::Blprnt;
use crate::errors::ApiError;

const MODELS_CACHE: &str = "models.json";
const MODELS_CACHE_KEY: &str = "models";

lazy_static::lazy_static! {
  static ref API_CLIENT: OnceLock<Arc<ApiClient>> = OnceLock::new();
}

pub struct ApiClient {
  base_url: String,
  client:   reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct SignInResponse {
  pub user_id: String,
  pub token:   String,
}

impl Debug for ApiClient {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ApiClient, base_url: {}", self.base_url)
  }
}

impl ApiClient {
  pub fn set() {
    let base_url = "https://api.blprnt.ai".into();

    let client = Self { base_url, client: reqwest::Client::new() };

    API_CLIENT.set(Arc::new(client)).expect("Failed to set API client");
  }

  pub fn get() -> Arc<Self> {
    API_CLIENT.get().expect("API client not set").clone()
  }

  pub async fn get_models(self: Arc<Self>) -> Result<Vec<LlmModelResponse>> {
    let response = self
      .client
      .get(format!("{}/admin/models", self.base_url))
      .send()
      .await
      .map_err(|e| ApiError::FailedToGetModels(e.to_string()))?;

    if !response.status().is_success() {
      if let Ok(store) = Blprnt::handle().store(MODELS_CACHE)
        && store.has(MODELS_CACHE_KEY)
        && let Ok(models) = serde_json::from_value::<Vec<LlmModelResponse>>(store.get(MODELS_CACHE_KEY).unwrap())
      {
        return Ok(models);
      };

      let body = response.text().await.map_err(|e| ApiError::FailedToGetModels(e.to_string()))?;
      return Err(ApiError::FailedToGetModels(body).into());
    }

    let models =
      response.json::<Vec<LlmModelResponse>>().await.map_err(|e| ApiError::FailedToGetModels(e.to_string()))?;

    if let Ok(store) = Blprnt::handle().store(MODELS_CACHE) {
      store.clear();
      store.set(MODELS_CACHE_KEY, serde_json::to_value(&models).unwrap());
    };

    Ok(models)
  }

  pub async fn submit_report_bug(self: Arc<Self>, request: ReportBugSubmitRequest) -> Result<ReportBugSubmitResponse> {
    let response = self
      .client
      .post(format!("{}/report-bug/submit", self.base_url))
      .header(CONTENT_TYPE, "application/json")
      .json(&request)
      .send()
      .await
      .map_err(|e| ApiError::FailedToSubmitReportBug(e.to_string()))?;

    if !response.status().is_success() {
      let body = response.text().await.map_err(|e| ApiError::FailedToSubmitReportBug(e.to_string()))?;
      tracing::error!("Failed to submit report bug: {}", body);
      return Err(ApiError::FailedToSubmitReportBug(body).into());
    }

    let body = response.text().await.map_err(|e| ApiError::FailedToSubmitReportBug(e.to_string()))?;

    tracing::info!("Report bug response: {}", body);

    let payload = serde_json::from_str::<ReportBugSubmitResponse>(&body)
      .map_err(|e| ApiError::FailedToSubmitReportBug(e.to_string()))?;

    Ok(payload)
  }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct LlmModelResponse {
  pub id:                 i32,
  pub name:               String,
  pub slug:               String,
  pub description:        String,
  pub input_price:        String,
  pub output_price:       String,
  pub context_length:     i64,
  pub is_free:            bool,
  pub supports_reasoning: bool,
  pub auto_router:        bool,
  pub enabled:            bool,
  pub supports_oauth:     bool,
  pub oauth_slug:         Option<String>,
  #[specta(type = String)]
  pub created_at:         DateTime<Utc>,
  #[specta(type = String)]
  pub updated_at:         DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ReportBugSeverity {
  #[serde(rename = "LOW")]
  Low,
  #[serde(rename = "MEDIUM")]
  Medium,
  #[serde(rename = "HIGH")]
  High,
  #[serde(rename = "CRITICAL")]
  Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReportBugScreenshotKind {
  #[serde(rename = "inline_base64")]
  InlineBase64,
  #[serde(rename = "reference_url")]
  ReferenceUrl,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugScreenshotPayload {
  pub kind:          ReportBugScreenshotKind,
  pub file_name:     String,
  pub mime_type:     String,
  pub byte_len:      u64,
  pub data_base64:   Option<String>,
  pub reference_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReportBugPastedAttachmentKind {
  Image,
  File,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReportBugAttachmentPayloadKind {
  #[serde(rename = "inline_base64")]
  InlineBase64,
  #[serde(rename = "file_reference")]
  FileReference,
  #[serde(rename = "reference_url")]
  ReferenceUrl,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugPastedAttachmentPayload {
  pub kind:          ReportBugPastedAttachmentKind,
  pub file_name:     String,
  pub mime_type:     String,
  pub byte_len:      u64,
  pub payload_kind:  ReportBugAttachmentPayloadKind,
  pub data_base64:   Option<String>,
  pub file_path:     Option<String>,
  pub reference_url: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugSubmitRequest {
  pub title:              String,
  pub description:        String,
  pub severity:           ReportBugSeverity,
  pub screenshot:         Option<ReportBugScreenshotPayload>,
  #[serde(default)]
  pub pasted_attachments: Vec<ReportBugPastedAttachmentPayload>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugNormalizedSubmission {
  pub title:              String,
  pub description:        String,
  pub severity:           ReportBugSeverity,
  pub screenshot:         Option<ReportBugScreenshotPayload>,
  #[serde(default)]
  pub pasted_attachments: Vec<ReportBugPastedAttachmentPayload>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReportBugSubmitState {
  Submitted,
  Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum ReportBugErrorCategory {
  Config,
  Validation,
  Screenshot,
  AttachmentUpload,
  Github,
  Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum ReportBugErrorCode {
  #[serde(rename = "RB_CONFIG_MISSING")]
  ConfigMissing,
  #[serde(rename = "RB_CONFIG_INVALID")]
  ConfigInvalid,
  #[serde(rename = "RB_CONFIG_STORE_UNAVAILABLE")]
  ConfigStoreUnavailable,
  #[serde(rename = "RB_VALIDATION_FAILED")]
  ValidationFailed,
  #[serde(rename = "RB_SCREENSHOT_CONTRACT_VIOLATION")]
  ScreenshotContractViolation,
  #[serde(rename = "RB_ATTACHMENT_CONTRACT_VIOLATION")]
  AttachmentContractViolation,
  #[serde(rename = "RB_ATTACHMENT_UPLOAD_CONFIG_INVALID")]
  AttachmentUploadConfigInvalid,
  #[serde(rename = "RB_ATTACHMENT_UPLOAD_PERMISSION_DENIED")]
  AttachmentUploadPermissionDenied,
  #[serde(rename = "RB_ATTACHMENT_UPLOAD_RATE_LIMITED")]
  AttachmentUploadRateLimited,
  #[serde(rename = "RB_ATTACHMENT_UPLOAD_FAILED")]
  AttachmentUploadFailed,
  #[serde(rename = "RB_GITHUB_AUTH_FAILED")]
  GithubAuthFailed,
  #[serde(rename = "RB_GITHUB_PERMISSION_DENIED")]
  GithubPermissionDenied,
  #[serde(rename = "RB_GITHUB_RATE_LIMITED")]
  GithubRateLimited,
  #[serde(rename = "RB_GITHUB_NOT_FOUND")]
  GithubNotFound,
  #[serde(rename = "RB_GITHUB_API_ERROR")]
  GithubApiError,
  #[serde(rename = "RB_GITHUB_NETWORK_ERROR")]
  GithubNetworkError,
  #[serde(rename = "RB_GITHUB_RESPONSE_INVALID")]
  GithubResponseInvalid,
  #[serde(rename = "RB_SUBMIT_NOT_IMPLEMENTED")]
  SubmitNotImplemented,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugFieldError {
  pub field:   String,
  pub code:    String,
  pub message: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugSubmitError {
  pub code:         ReportBugErrorCode,
  pub category:     ReportBugErrorCategory,
  pub message:      String,
  pub retryable:    bool,
  pub field_errors: Vec<ReportBugFieldError>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugScreenshotContract {
  pub max_bytes:                 u64,
  pub allowed_mime_types:        Vec<String>,
  pub allowed_reference_schemes: Vec<String>,
  pub supported_kinds:           Vec<ReportBugScreenshotKind>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ReportBugSubmitResponse {
  pub state:                 ReportBugSubmitState,
  pub normalized_submission: Option<ReportBugNormalizedSubmission>,
  pub error:                 Option<ReportBugSubmitError>,
  pub screenshot_contract:   ReportBugScreenshotContract,
}
