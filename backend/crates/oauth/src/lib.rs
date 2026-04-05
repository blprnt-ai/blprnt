#![warn(unused, unused_crate_dependencies)]

pub mod browser;
pub mod consts;
pub mod types;

pub mod anthropic;
pub mod openai;

pub mod pkce;

use anyhow::Result;
use http::HeaderMap;
use http::header::AUTHORIZATION;
use http::header::HeaderValue;
use shared::errors::OauthError;

pub fn insert_bearer(headers: &mut HeaderMap, token: &str) -> Result<()> {
  let value = format!("Bearer {token}");
  headers
    .insert(AUTHORIZATION, HeaderValue::from_str(&value).map_err(|e| OauthError::FailedToInsertBearer(e.to_string()))?);
  Ok(())
}

pub fn insert_api_key(headers: &mut HeaderMap, key: &str, header_name: &str) -> Result<()> {
  let name = http::header::HeaderName::from_bytes(header_name.as_bytes())
    .map_err(|e| OauthError::FailedToInsertApiKey(e.to_string()))?;
  headers.insert(name, HeaderValue::from_str(key).map_err(|e| OauthError::FailedToInsertApiKey(e.to_string()))?);
  Ok(())
}
