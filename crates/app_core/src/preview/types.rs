use chrono::DateTime;
use chrono::Utc;
use common::errors::TauriError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMode {
  Dev,
  Static,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PreviewSessionStatus {
  Starting,
  Ready,
  Error,
  Stopped,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PreviewServerAction {
  Attached,
  Started,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PreviewSession {
  pub id:           String,
  pub project_id:   String,
  pub mode:         PreviewMode,
  pub status:       PreviewSessionStatus,
  pub partition_id: String,
  pub url:          String,
  #[specta(type = String)]
  pub created_at:   DateTime<Utc>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PreviewStartParams {
  pub project_id:              String,
  pub mode:                    PreviewMode,
  pub dev_server_url:          Option<String>,
  pub static_path:             Option<String>,
  pub allowed_hosts:           Option<Vec<String>>,
  #[serde(default = "default_instrumentation_enabled")]
  pub instrumentation_enabled: bool,
  pub proxy_port:              Option<u16>,
  pub static_port:             Option<u16>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PreviewDetectedServer {
  pub language:         Option<String>,
  pub framework:        Option<String>,
  pub suggested_port:   Option<u16>,
  pub detected_command: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct PreviewStatusResponse {
  pub status:           PreviewSessionStatus,
  pub last_error:       Option<TauriError>,
  pub server_action:    Option<PreviewServerAction>,
  pub detected:         Option<PreviewDetectedServer>,
  pub url:              Option<String>,
  pub was_auto_started: Option<bool>,
}

fn default_instrumentation_enabled() -> bool {
  true
}
