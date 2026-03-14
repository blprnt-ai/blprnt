use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::errors::OauthError;
use common::shared::prelude::*;
use rand::RngCore;
use reqwest::Client;

use super::state::ExchangedTokens;
use super::state::TokenResponse;
use crate::browser;
use crate::consts;
use crate::pkce::Pkce;

#[derive(Clone, Debug)]
pub struct OauthConfig {
  pub client_id:     String,
  pub authorize_url: String,
  pub token_url:     String,
  pub success_html:  Option<String>,
}

impl Default for OauthConfig {
  fn default() -> Self {
    Self {
      client_id:     consts::openai::CLIENT_ID.into(),
      authorize_url: consts::openai::AUTHORIZE_URL.into(),
      token_url:     consts::openai::TOKEN_URL.into(),
      success_html:  None,
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct IdTokenClaims {
  #[serde(rename = "https://api.openai.com/auth")]
  auth_info: IdTokenAuthInfo,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct IdTokenAuthInfo {
  chatgpt_account_id: String,
}

#[derive(Clone, Debug)]
pub struct OpenAiOauth;

impl OpenAiOauth {
  pub async fn authenticate_with_oauth() -> Result<Option<OauthToken>> {
    let cfg = OauthConfig::default();

    let (verifier, challenge) = Pkce::generate()?;
    let state = Self::generate_state();

    let params = Self::build_oauth_params(&cfg.client_id, &challenge, &state);
    let query: Vec<(&str, String)> = params.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    let qp: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let success_html = cfg.success_html.as_deref();
    let cb = browser::run_local_browser_flow(
      &cfg.authorize_url,
      consts::openai::REDIRECT_PATH,
      &qp,
      success_html,
      consts::flow::DEFAULT_TIMEOUT_SECS,
    )
    .await?;

    if cb.is_none() {
      return Ok(None);
    }

    let cb = cb.unwrap();

    if let Some(cb_state) = cb.params.get("state") {
      if cb_state != &state {
        return Err(OauthError::StateMismatch("state mismatch in OAuth callback".into()).into());
      }
    } else {
      return Err(OauthError::MissingState("missing state in OAuth callback".into()).into());
    }

    let code = cb
      .params
      .get("code")
      .cloned()
      .ok_or_else(|| OauthError::MissingAuthorizationCode("missing authorization code in callback".into()))?;
    let redirect_uri = cb.redirect_uri;
    let token_response =
      Self::exchange_code_for_token(&cfg.token_url, &cfg.client_id, &redirect_uri, &verifier, &code).await?;

    let oauth_token = OpenAiOauthToken {
      access_token:  token_response.access_token.clone(),
      refresh_token: token_response.refresh_token.clone(),
      expires_at_ms: Self::now_ms() + consts::I28_DAYS_LATER,
      account_id:    token_response.account_id.clone(),
    };

    Ok(Some(oauth_token.into()))
  }

  pub async fn refresh_with_refresh_token(
    token_url: &str,
    client_id: &str,
    refresh_token: &str,
  ) -> Result<Option<TokenResponse>> {
    let client = Client::new();
    let body = serde_json::json!({
      "grant_type": "refresh_token",
      "refresh_token": refresh_token,
      "client_id": client_id,
      "scope": "openid profile email",
    });

    tracing::info!("Sending refresh request to {}", token_url);
    tracing::info!("Body: {}", serde_json::to_string_pretty(&body).unwrap());

    let res = client
      .post(token_url)
      .json(&body)
      .send()
      .await
      .map_err(|e| OauthError::FailedToSendRefreshRequest(e.to_string()))?;

    if !res.status().is_success() {
      let status = res.status();
      let body = res.text().await;
      tracing::error!("Body: {:?}", body);

      return Err(OauthError::FailedToRefreshWithRefreshToken(format!("status: {}", status)).into());
    }

    let t: TokenResponse = res.json().await.map_err(|e| OauthError::FailedToParseRefreshResponse(e.to_string()))?;
    Ok(Some(t))
  }

  fn build_oauth_params(client_id: &str, code_challenge: &str, state: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("response_type".into(), "code".into());
    m.insert("client_id".into(), client_id.to_string());
    m.insert("scope".into(), consts::openai::SCOPE.into());
    m.insert("code_challenge".into(), code_challenge.to_string());
    m.insert("code_challenge_method".into(), "S256".into());
    m.insert("id_token_add_organizations".into(), "true".into());
    m.insert("codex_cli_simplified_flow".into(), "true".into());
    m.insert("state".into(), state.to_string());
    m.insert("originator".into(), "codex_cli_rs".to_string());
    m
  }

  async fn exchange_code_for_token(
    token_url: &str,
    client_id: &str,
    redirect_uri: &str,
    code_verifier: &str,
    code: &str,
  ) -> Result<ExchangedTokens> {
    let client = Client::new();
    let json = format!(
      "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
      urlencoding::encode(code),
      urlencoding::encode(redirect_uri),
      urlencoding::encode(client_id),
      urlencoding::encode(code_verifier)
    );
    let request = client.post(token_url).header("Content-Type", "application/x-www-form-urlencoded").body(json);
    let resp = request.send().await.map_err(|e| OauthError::FailedToSendTokenRequest(e.to_string()))?;

    if !resp.status().is_success() {
      let status = resp.status();
      let body = resp.text().await.unwrap_or_default();
      return Err(OauthError::FailedToExchangeCodeForToken(format!("{}: {}", status, body)).into());
    }

    let tokens: TokenResponse = resp.json().await.map_err(|e| OauthError::FailedToParseTokenResponse(e.to_string()))?;
    let account_id = if let Some(id_token) = &tokens.id_token { Self::parse_id_token(id_token).ok() } else { None };

    Ok(ExchangedTokens { access_token: tokens.access_token, refresh_token: tokens.refresh_token, account_id })
  }

  fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
  }

  fn parse_id_token(id_token: &str) -> Result<String> {
    let mut parts = id_token.split('.');
    let (_h, payload_b64, _s) = match (parts.next(), parts.next(), parts.next()) {
      (Some(h), Some(p), Some(s)) if !h.is_empty() && !p.is_empty() && !s.is_empty() => (h, p, s),
      _ => {
        return Err(OauthError::FailedToParseIdToken("invalid JWT format".into()).into());
      }
    };

    match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload_b64) {
      Ok(payload) => match serde_json::from_slice::<IdTokenClaims>(&payload) {
        Ok(claims) => Ok(claims.auth_info.chatgpt_account_id.clone()),
        Err(e) => Err(OauthError::FailedToParseIdToken(e.to_string()).into()),
      },
      Err(e) => Err(OauthError::FailedToDecodeIdToken(e.to_string()).into()),
    }
  }

  fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_millis() as u64
  }
}
