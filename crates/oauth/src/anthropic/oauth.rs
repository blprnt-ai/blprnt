use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http::HeaderMap;
use http::header::ACCEPT;
use http::header::USER_AGENT;
use rand::RngCore;
use reqwest::Client;
use shared::errors::OauthError;

use crate::anthropic::state::TokenResponse;
use crate::browser;
use crate::consts;
use crate::pkce::Pkce;
use crate::types::AnthropicOauthToken;
use crate::types::OauthToken;

#[derive(Clone, Debug)]
pub struct OauthConfig {
  pub client_id:     String,
  pub authorize_url: String,
  pub token_url:     String,
  pub scopes:        Vec<String>,
  pub success_html:  Option<String>,
}

impl Default for OauthConfig {
  fn default() -> Self {
    Self {
      client_id:     consts::anthropic::CLIENT_ID.into(),
      authorize_url: consts::anthropic::AUTHORIZE_URL.into(),
      token_url:     consts::anthropic::TOKEN_URL.into(),
      scopes:        consts::anthropic::SCOPES.iter().map(|s| s.to_string()).collect(),
      success_html:  None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct AnthropicOauth;

impl AnthropicOauth {
  pub async fn authenticate_with_oauth() -> Result<Option<OauthToken>> {
    let cfg = OauthConfig::default();

    let (verifier, challenge) = Pkce::generate()?;
    let state = Self::generate_state();

    let params = Self::build_oauth_params(&cfg.client_id, &cfg.scopes);
    let query: Vec<(&str, String)> = params.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    let mut qp: Vec<(&str, String)> = query;
    qp.push(("code_challenge", challenge));
    qp.push(("code_challenge_method", "S256".into()));
    qp.push(("state", state.clone()));

    let qp: Vec<(&str, &str)> = qp.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let success_html = cfg.success_html.as_deref();
    let cb = browser::run_local_browser_flow(
      &cfg.authorize_url,
      consts::flow::REDIRECT_PATH,
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
      Self::exchange_code_for_token(&cfg.token_url, &cfg.client_id, &code, &redirect_uri, &verifier, &state).await?;

    let oauth_token = AnthropicOauthToken {
      access_token:  token_response.access_token.clone(),
      refresh_token: token_response.refresh_token.clone(),
      expires_at_ms: Self::now_ms() + (token_response.expires_in.saturating_mul(1000)),
    };

    Ok(Some(oauth_token.into()))
  }

  pub async fn refresh_with_refresh_token(refresh_token: &str) -> Result<Option<TokenResponse>> {
    let client = Client::new();
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": consts::anthropic::CLIENT_ID,
    });
    let mut headers = HeaderMap::new();
    Self::with_anthropic_headers(&mut headers);
    let res = client
      .post(consts::anthropic::TOKEN_URL)
      .headers(headers)
      .json(&body)
      .send()
      .await
      .map_err(|e| OauthError::FailedToSendRefreshRequest(e.to_string()))?;

    if !res.status().is_success() {
      return Err(OauthError::FailedToRefreshWithRefreshToken(format!("status: {}", res.status())).into());
    }

    let t: TokenResponse = res.json().await.map_err(|e| OauthError::FailedToParseRefreshResponse(e.to_string()))?;
    Ok(Some(t))
  }

  fn build_oauth_params(client_id: &str, scopes: &[String]) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("code".into(), "true".into());
    m.insert("response_type".into(), "code".into());
    m.insert("client_id".into(), client_id.to_string());
    if !scopes.is_empty() {
      m.insert("scope".into(), scopes.join(" "));
    }
    m
  }

  async fn exchange_code_for_token(
    token_url: &str,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    state: &str,
  ) -> Result<TokenResponse> {
    let client = Client::new();
    let json = serde_json::json!({
      "grant_type": "authorization_code",
      "code": code,
      "client_id": client_id,
      "redirect_uri": redirect_uri,
      "code_verifier": code_verifier,
      "state": state,
    });

    let mut headers = HeaderMap::new();
    Self::with_anthropic_headers(&mut headers);
    let res = client
      .post(token_url)
      .headers(headers)
      .json(&json)
      .send()
      .await
      .map_err(|e| OauthError::FailedToSendTokenRequest(e.to_string()))?;

    if !res.status().is_success() {
      let status = res.status();
      let body = res.text().await.unwrap_or_default();
      return Err(OauthError::FailedToExchangeCodeForToken(format!("{}: {}", status, body)).into());
    }

    res.json().await.map_err(|e| OauthError::FailedToParseTokenResponse(e.to_string()).into())
  }

  fn with_anthropic_headers(headers: &mut http::HeaderMap) {
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(USER_AGENT, "claude-cli/2.0.8 (external, cli)".parse().unwrap());
    headers.insert("anthropic-beta", "oauth-2025-04-20".parse().unwrap());
  }

  fn generate_state() -> String {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
  }

  fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_millis() as u64
  }
}
