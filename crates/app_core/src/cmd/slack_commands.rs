use base64::Engine;
use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use serde::Serialize;
use surrealdb::types::Uuid;
use tauri::State;
use vault::Vault;

use crate::engine_manager::EngineManager;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SlackStatus {
  pub enabled:    bool,
  pub connected:  bool,
  pub last_error: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn slack_set_enabled(manager: State<'_, std::sync::Arc<EngineManager>>, enabled: bool) -> TauriResult<()> {
  manager.slack.set_enabled(enabled)?;
  Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn slack_disconnect(manager: State<'_, std::sync::Arc<EngineManager>>) -> TauriResult<()> {
  manager.slack.set_status(false, None)?;
  manager.slack.set_oauth_state(None)?;
  // Best-effort: clear secrets from Stronghold.
  let tunnel_id = EngineManager::get_tunnel_uuid();
  let token_key = slack_access_token_key(tunnel_id);
  let user_key = slack_authed_user_id_key(tunnel_id);
  tokio::spawn(async move {
    let _ = vault::delete_stronghold_secret(Vault::Key, token_key).await;
    let _ = vault::delete_stronghold_secret(Vault::Key, user_key).await;
  });
  Ok(())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SlackStartOAuthResponse {
  pub url:   String,
  pub state: String,
}

#[derive(Clone, Debug, Serialize)]
struct SlackDesktopState {
  tunnel_id: String,
}

#[tauri::command]
#[specta::specta]
pub async fn slack_start_oauth(
  manager: State<'_, std::sync::Arc<EngineManager>>,
) -> TauriResult<SlackStartOAuthResponse> {
  const SLACK_OAUTH_START_URL: &str = "https://relay.blprnt.ai/slack/oauth/start";

  let tunnel_id = EngineManager::get_tunnel_uuid().to_string();
  let state_json = SlackDesktopState { tunnel_id };
  let state_bytes = serde_json::to_vec(&state_json).map_err(anyhow::Error::from).into_tauri()?;
  let state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_bytes);

  manager.slack.set_oauth_state(Some(state.clone()))?;

  let url = format!("{base}?state={state}", base = SLACK_OAUTH_START_URL, state = urlencoding::encode(&state));
  Ok(SlackStartOAuthResponse { url, state })
}

pub fn slack_access_token_key(tunnel_id: Uuid) -> Uuid {
  uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, format!("slack:access_token:{tunnel_id}").as_bytes()).into()
}

pub fn slack_authed_user_id_key(tunnel_id: Uuid) -> Uuid {
  uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, format!("slack:authed_user_id:{tunnel_id}").as_bytes()).into()
}

#[tauri::command]
#[specta::specta]
pub async fn slack_status(manager: State<'_, std::sync::Arc<EngineManager>>) -> TauriResult<SlackStatus> {
  Ok(SlackStatus {
    enabled:    manager.slack.enabled(),
    connected:  manager.slack.connected(),
    last_error: manager.slack.last_error(),
  })
}
