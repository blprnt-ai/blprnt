use async_trait::async_trait;
use oauth2::TokenResponse;
use persistence::prelude::DbId;
use persistence::prelude::McpServerId;
use persistence::prelude::McpServerPatch;
use persistence::prelude::McpServerRecord;
use persistence::prelude::McpServerRepository;
use rmcp::transport::AuthError;
use rmcp::transport::CredentialStore;
use rmcp::transport::StateStore;
use rmcp::transport::StoredAuthorizationState;
use rmcp::transport::StoredCredentials;
use rmcp::transport::auth::OAuthState;
use rmcp::transport::auth::OAuthTokenResponse;
use shared::tools::McpServerAuthState;
use uuid::Uuid;

use crate::routes::errors::ApiErrorKind;

const MCP_OAUTH_CREDENTIALS_PREFIX: &str = "mcp_oauth_credentials";
const MCP_OAUTH_STATE_PREFIX: &str = "mcp_oauth_state";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct McpOauthMetadataDto {
  pub authorization_endpoint: String,
  pub token_endpoint:         String,
  pub registration_endpoint:  Option<String>,
  pub issuer:                 Option<String>,
  pub scopes_supported:       Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct McpOauthLaunchDto {
  pub server_id:         Uuid,
  pub authorization_url: String,
  pub redirect_uri:      String,
  pub auth_state:        McpServerAuthState,
  pub auth_summary:      Option<String>,
  pub suggested_scopes:  Vec<String>,
  pub metadata:          Option<McpOauthMetadataDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct McpOauthStatusDto {
  pub server_id:         Uuid,
  pub auth_state:        McpServerAuthState,
  pub auth_summary:      Option<String>,
  pub authorization_url: Option<String>,
  pub has_token:         bool,
  pub token_expires_at:  Option<u64>,
  pub scopes:            Vec<String>,
  pub metadata:          Option<McpOauthMetadataDto>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct McpOauthCompletePayload {
  pub code:  String,
  pub state: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedMcpOauthTokenEnvelope {
  pub client_id:         String,
  pub token_response:    Option<serde_json::Value>,
  pub granted_scopes:    Vec<String>,
  pub token_received_at: Option<u64>,
  pub authorization_url: Option<String>,
}

#[derive(Clone)]
struct VaultCredentialStore {
  server_id: McpServerId,
}

#[derive(Clone)]
struct VaultStateStore {
  server_id: McpServerId,
}

#[async_trait]
impl CredentialStore for VaultCredentialStore {
  async fn load(&self) -> Result<Option<StoredCredentials>, AuthError> {
    let Some(raw) = vault::get_stronghold_secret(vault::Vault::Key, credential_store_key(&self.server_id)).await else {
      return Ok(None);
    };

    let persisted: PersistedMcpOauthTokenEnvelope =
      serde_json::from_str(&raw).map_err(|error| AuthError::InternalError(error.to_string()))?;
    let token_response = persisted
      .token_response
      .map(serde_json::from_value::<OAuthTokenResponse>)
      .transpose()
      .map_err(|error| AuthError::InternalError(error.to_string()))?;

    let credentials = serde_json::json!({
      "client_id": persisted.client_id,
      "token_response": token_response,
      "granted_scopes": persisted.granted_scopes,
      "token_received_at": persisted.token_received_at,
    });

    Ok(Some(serde_json::from_value(credentials).map_err(|error| AuthError::InternalError(error.to_string()))?))
  }

  async fn save(&self, credentials: StoredCredentials) -> Result<(), AuthError> {
    let existing = load_envelope(&self.server_id).await.map_err(|error| AuthError::InternalError(error.to_string()))?;
    let envelope = PersistedMcpOauthTokenEnvelope {
      client_id:         credentials.client_id,
      token_response:    credentials
        .token_response
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| AuthError::InternalError(error.to_string()))?,
      granted_scopes:    credentials.granted_scopes,
      token_received_at: credentials.token_received_at,
      authorization_url: existing.and_then(|value| value.authorization_url),
    };

    vault::set_stronghold_secret(
      vault::Vault::Key,
      credential_store_key(&self.server_id),
      &serde_json::to_string(&envelope).map_err(|error| AuthError::InternalError(error.to_string()))?,
    )
    .await
    .map_err(|error| AuthError::InternalError(error.to_string()))
  }

  async fn clear(&self) -> Result<(), AuthError> {
    vault::delete_stronghold_secret(vault::Vault::Key, credential_store_key(&self.server_id))
      .await
      .map_err(|error| AuthError::InternalError(error.to_string()))
  }
}

#[async_trait]
impl StateStore for VaultStateStore {
  async fn save(&self, csrf_token: &str, state: StoredAuthorizationState) -> Result<(), AuthError> {
    vault::set_stronghold_secret(
      vault::Vault::Key,
      state_store_key(&self.server_id, csrf_token),
      &serde_json::to_string(&state).map_err(|error| AuthError::InternalError(error.to_string()))?,
    )
    .await
    .map_err(|error| AuthError::InternalError(error.to_string()))
  }

  async fn load(&self, csrf_token: &str) -> Result<Option<StoredAuthorizationState>, AuthError> {
    let Some(raw) = vault::get_stronghold_secret(vault::Vault::Key, state_store_key(&self.server_id, csrf_token)).await
    else {
      return Ok(None);
    };
    serde_json::from_str(&raw).map(Some).map_err(|error| AuthError::InternalError(error.to_string()))
  }

  async fn delete(&self, csrf_token: &str) -> Result<(), AuthError> {
    vault::delete_stronghold_secret(vault::Vault::Key, state_store_key(&self.server_id, csrf_token))
      .await
      .map_err(|error| AuthError::InternalError(error.to_string()))
  }
}

fn credential_store_key(server_id: &McpServerId) -> Uuid {
  Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("{MCP_OAUTH_CREDENTIALS_PREFIX}:{}", server_id.uuid()).as_bytes())
}

fn state_store_key(server_id: &McpServerId, csrf_token: &str) -> Uuid {
  Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("{MCP_OAUTH_STATE_PREFIX}:{}:{csrf_token}", server_id.uuid()).as_bytes())
}

async fn load_envelope(server_id: &McpServerId) -> anyhow::Result<Option<PersistedMcpOauthTokenEnvelope>> {
  let Some(raw) = vault::get_stronghold_secret(vault::Vault::Key, credential_store_key(server_id)).await else {
    return Ok(None);
  };
  Ok(Some(serde_json::from_str(&raw)?))
}

fn redirect_uri(server_id: &McpServerId) -> String {
  format!("http://localhost:9171/api/v1/mcp-servers/{}/oauth/callback", server_id.uuid())
}

fn default_scopes() -> Vec<String> {
  vec!["mcp".to_string()]
}

fn api_error(error: impl ToString) -> ApiErrorKind {
  ApiErrorKind::BadRequest(serde_json::json!({ "message": error.to_string() }))
}

async fn build_oauth_state(server: &McpServerRecord) -> Result<OAuthState, AuthError> {
  let mut oauth = OAuthState::new(&server.endpoint_url, None).await?;
  match &mut oauth {
    OAuthState::Unauthorized(manager) => {
      manager.set_credential_store(VaultCredentialStore { server_id: server.id.clone() });
      manager.set_state_store(VaultStateStore { server_id: server.id.clone() });
    }
    _ => {}
  }

  if let Some(envelope) =
    load_envelope(&server.id).await.map_err(|error| AuthError::InternalError(error.to_string()))?
    && let Some(token_value) = envelope.token_response
  {
    let token = serde_json::from_value(token_value).map_err(|error| AuthError::InternalError(error.to_string()))?;
    oauth.set_credentials(&envelope.client_id, token).await?;
  }

  Ok(oauth)
}

fn current_metadata(_oauth: &OAuthState) -> Option<McpOauthMetadataDto> {
  None
}

async fn persist_launch_url(server_id: &McpServerId, authorization_url: String) -> anyhow::Result<()> {
  let mut envelope = load_envelope(server_id).await?.unwrap_or(PersistedMcpOauthTokenEnvelope {
    client_id:         String::new(),
    token_response:    None,
    granted_scopes:    Vec::new(),
    token_received_at: None,
    authorization_url: None,
  });
  envelope.authorization_url = Some(authorization_url);
  vault::set_stronghold_secret(vault::Vault::Key, credential_store_key(server_id), &serde_json::to_string(&envelope)?)
    .await?;
  Ok(())
}

pub async fn launch(server: &McpServerRecord, reconnect: bool) -> Result<McpOauthLaunchDto, ApiErrorKind> {
  let mut oauth = build_oauth_state(server).await.map_err(api_error)?;
  let redirect_uri = redirect_uri(&server.id);
  let scopes = default_scopes();
  let scope_refs = scopes.iter().map(String::as_str).collect::<Vec<_>>();

  oauth.start_authorization(&scope_refs, &redirect_uri, Some("blprnt MCP client")).await.map_err(api_error)?;
  let authorization_url = oauth.get_authorization_url().await.map_err(api_error)?;
  persist_launch_url(&server.id, authorization_url.clone())
    .await
    .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;

  let auth_state = if reconnect { McpServerAuthState::ReconnectRequired } else { McpServerAuthState::AuthRequired };
  let auth_summary = Some(if reconnect {
    format!("Reconnect required for '{}'. Re-authorize this MCP server.", server.display_name)
  } else {
    format!("OAuth authorization started for '{}'.", server.display_name)
  });

  McpServerRepository::update(
    server.id.clone(),
    McpServerPatch {
      auth_state: Some(auth_state.clone()),
      auth_summary: Some(auth_summary.clone()),
      ..Default::default()
    },
  )
  .await
  .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;

  Ok(McpOauthLaunchDto {
    server_id: server.id.uuid(),
    authorization_url,
    redirect_uri,
    auth_state,
    auth_summary,
    suggested_scopes: scopes,
    metadata: current_metadata(&oauth),
  })
}

pub async fn complete(server: &McpServerRecord, code: &str, state: &str) -> Result<McpOauthStatusDto, ApiErrorKind> {
  let mut oauth = build_oauth_state(server).await.map_err(api_error)?;
  oauth.handle_callback(code, state).await.map_err(api_error)?;
  let (client_id, token_response) = oauth.get_credentials().await.map_err(api_error)?;
  let token_response = token_response.ok_or_else(|| api_error("oauth callback completed without token response"))?;

  let authorization_url = load_envelope(&server.id).await.ok().flatten().and_then(|value| value.authorization_url);

  let token_json = serde_json::to_value(&token_response)
    .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;
  let scopes: Vec<String> =
    token_response.scopes().map(|v| v.iter().map(|s| s.to_string()).collect()).unwrap_or_default();
  let expires_at_ms = token_response
    .expires_in()
    .map(|duration| (chrono::Utc::now().timestamp_millis().max(0) as u64).saturating_add(duration.as_millis() as u64));

  let envelope = PersistedMcpOauthTokenEnvelope {
    client_id,
    token_response: Some(token_json),
    granted_scopes: scopes.clone(),
    token_received_at: Some(chrono::Utc::now().timestamp().max(0) as u64),
    authorization_url: authorization_url.clone(),
  };
  vault::set_stronghold_secret(
    vault::Vault::Key,
    credential_store_key(&server.id),
    &serde_json::to_string(&envelope).unwrap(),
  )
  .await
  .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;

  adapters::mcp::store_mcp_server_oauth_token(
    &server.id,
    &adapters::mcp::StoredMcpOauthToken {
      access_token: token_response.access_token().secret().to_string(),
      refresh_token: token_response.refresh_token().map(|value| value.secret().to_string()),
      expires_at_ms,
      token_type: Some(format!("{:?}", token_response.token_type())),
      scopes: scopes.clone(),
      authorization_url: authorization_url.clone(),
    },
  )
  .await
  .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;

  let updated = McpServerRepository::update(
    server.id.clone(),
    McpServerPatch {
      auth_state: Some(McpServerAuthState::Connected),
      auth_summary: Some(Some(format!("OAuth connected for '{}'.", server.display_name))),
      ..Default::default()
    },
  )
  .await
  .map_err(|error| ApiErrorKind::InternalServerError(serde_json::json!({ "message": error.to_string() })))?;

  Ok(McpOauthStatusDto {
    server_id: updated.id.uuid(),
    auth_state: updated.auth_state,
    auth_summary: updated.auth_summary,
    authorization_url,
    has_token: true,
    token_expires_at: expires_at_ms,
    scopes,
    metadata: current_metadata(&oauth),
  })
}

pub async fn status(server: &McpServerRecord) -> anyhow::Result<McpOauthStatusDto> {
  let envelope = load_envelope(&server.id).await?;
  let token = adapters::mcp::load_mcp_server_oauth_token(&server.id).await?;

  Ok(McpOauthStatusDto {
    server_id:         server.id.uuid(),
    auth_state:        server.auth_state.clone(),
    auth_summary:      server.auth_summary.clone(),
    authorization_url: envelope.and_then(|value| value.authorization_url),
    has_token:         token.is_some(),
    token_expires_at:  token.as_ref().and_then(|value| value.expires_at_ms),
    scopes:            token.map(|value| value.scopes).unwrap_or_default(),
    metadata:          None,
  })
}
