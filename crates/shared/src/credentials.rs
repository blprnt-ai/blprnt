use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
  pub access:  String,
  pub refresh: String,
  pub expires: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClaudeCredential {
  Oauth { provider: String, access: String, refresh: String, expires: i64 },
  Token { provider: String, token: String, expires: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexCredential {
  pub access:     String,
  pub refresh:    String,
  pub expires:    i64,
  pub account_id: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn now_ms() -> i64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis() as i64
}

fn home_dir() -> PathBuf {
  directories::UserDirs::new().unwrap().home_dir().to_path_buf()
}

fn resolve_path(input: &str) -> PathBuf {
  match input.strip_prefix("~/") {
    Some(rest) => home_dir().join(rest),
    None if input == "~" => home_dir(),
    None => PathBuf::from(input),
  }
}

fn read_json(path: &Path) -> Option<serde_json::Value> {
  serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn write_json(path: &Path, value: &serde_json::Value) -> bool {
  serde_json::to_vec_pretty(value).ok().and_then(|bytes| fs::write(path, bytes).ok()).is_some()
}

/// Helper trait for ergonomic JSON field extraction
trait JsonExt {
  fn str_field(&self, key: &str) -> Option<String>;
  fn i64_field(&self, key: &str) -> Option<i64>;
  fn obj_field(&self, key: &str) -> Option<&serde_json::Map<String, serde_json::Value>>;
}

impl JsonExt for serde_json::Value {
  fn str_field(&self, key: &str) -> Option<String> {
    self.get(key)?.as_str().map(String::from)
  }

  fn i64_field(&self, key: &str) -> Option<i64> {
    self.get(key)?.as_i64()
  }

  fn obj_field(&self, key: &str) -> Option<&serde_json::Map<String, serde_json::Value>> {
    self.get(key)?.as_object()
  }
}

impl JsonExt for serde_json::Map<String, serde_json::Value> {
  fn str_field(&self, key: &str) -> Option<String> {
    self.get(key)?.as_str().map(String::from)
  }

  fn i64_field(&self, key: &str) -> Option<i64> {
    self.get(key)?.as_i64()
  }

  fn obj_field(&self, key: &str) -> Option<&serde_json::Map<String, serde_json::Value>> {
    self.get(key)?.as_object()
  }
}

// ============================================================================
// macOS Keychain
// ============================================================================

#[cfg(target_os = "macos")]
mod keychain {
  use std::process::Command;
  use std::process::Stdio;

  pub fn find_password(service: &str, account: Option<&str>) -> Option<String> {
    let mut cmd = Command::new("security");
    cmd.args(["find-generic-password", "-s", service]);
    if let Some(acc) = account {
      cmd.args(["-a", acc]);
    }
    cmd.arg("-w").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());

    let output = cmd.output().ok()?;
    output.status.success().then(|| String::from_utf8(output.stdout).ok()).flatten().map(|s| s.trim().to_string())
  }

  pub fn set_password(service: &str, account: &str, value: &str) -> bool {
    Command::new("security")
      .args(["add-generic-password", "-U", "-s", service, "-a", account, "-w", value])
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  }
}

#[cfg(not(target_os = "macos"))]
mod keychain {
  pub fn find_password(_service: &str, _account: Option<&str>) -> Option<String> {
    None
  }

  pub fn set_password(_service: &str, _account: &str, _value: &str) -> bool {
    false
  }
}

// ============================================================================
// Claude CLI
// ============================================================================

const CLAUDE_CREDS_PATH: &str = ".claude/.credentials.json";
const CLAUDE_KEYCHAIN_SERVICE: &str = "Claude Code-credentials";
const CLAUDE_KEYCHAIN_ACCOUNT: &str = "Claude Code";

fn claude_creds_path(home: Option<&Path>) -> PathBuf {
  home.map(PathBuf::from).unwrap_or_else(home_dir).join(CLAUDE_CREDS_PATH)
}

fn parse_claude_oauth(data: &serde_json::Value) -> Option<ClaudeCredential> {
  let oauth = data.obj_field("claudeAiOauth")?;
  let access = oauth.str_field("accessToken")?;
  let expires = oauth.i64_field("expiresAt")?;
  let refresh = oauth.str_field("refreshToken").filter(|s| !s.is_empty());

  Some(match refresh {
    Some(refresh) => ClaudeCredential::Oauth { provider: "anthropic".into(), access, refresh, expires },
    None => ClaudeCredential::Token { provider: "anthropic".into(), token: access, expires },
  })
}

pub fn read_claude_credentials(allow_keychain: bool, home: Option<&Path>) -> Option<ClaudeCredential> {
  // Try keychain first on macOS
  if allow_keychain
    && let Some(secret) = keychain::find_password(CLAUDE_KEYCHAIN_SERVICE, None)
    && let Some(cred) = serde_json::from_str(&secret).ok().and_then(|v| parse_claude_oauth(&v))
  {
    return Some(cred);
  }

  // Fall back to file
  read_json(&claude_creds_path(home)).and_then(|v| parse_claude_oauth(&v))
}

pub fn write_claude_credentials(creds: &OAuthCredentials, home: Option<&Path>) -> bool {
  write_claude_keychain(creds) || write_claude_file(creds, home)
}

fn write_claude_keychain(creds: &OAuthCredentials) -> bool {
  let Some(existing) = keychain::find_password(CLAUDE_KEYCHAIN_SERVICE, None) else {
    return false;
  };

  let mut data: serde_json::Value = match serde_json::from_str(&existing) {
    Ok(v) => v,
    Err(_) => return false,
  };

  if let Some(oauth) = data.get_mut("claudeAiOauth").and_then(|v| v.as_object_mut()) {
    oauth.insert("accessToken".into(), creds.access.clone().into());
    oauth.insert("refreshToken".into(), creds.refresh.clone().into());
    oauth.insert("expiresAt".into(), creds.expires.into());

    if let Ok(json) = serde_json::to_string(&data) {
      return keychain::set_password(CLAUDE_KEYCHAIN_SERVICE, CLAUDE_KEYCHAIN_ACCOUNT, &json);
    }
  }
  false
}

fn write_claude_file(creds: &OAuthCredentials, home: Option<&Path>) -> bool {
  let path = claude_creds_path(home);
  let Some(mut data) = read_json(&path) else {
    return false;
  };

  if let Some(oauth) = data.get_mut("claudeAiOauth").and_then(|v| v.as_object_mut()) {
    oauth.insert("accessToken".into(), creds.access.clone().into());
    oauth.insert("refreshToken".into(), creds.refresh.clone().into());
    oauth.insert("expiresAt".into(), creds.expires.into());
    return write_json(&path, &data);
  }
  false
}

// ============================================================================
// Codex CLI
// ============================================================================

const CODEX_AUTH_FILENAME: &str = "auth.json";
const CODEX_KEYCHAIN_SERVICE: &str = "Codex Auth";

fn codex_home() -> PathBuf {
  let path = env::var("CODEX_HOME").map(|s| resolve_path(&s)).unwrap_or_else(|_| resolve_path("~/.codex"));
  fs::canonicalize(&path).unwrap_or(path)
}

fn codex_auth_path() -> PathBuf {
  codex_home().join(CODEX_AUTH_FILENAME)
}

fn codex_keychain_account() -> String {
  let hash = Sha256::digest(codex_home().to_string_lossy().as_bytes());
  format!("cli|{}", &hex::encode(hash)[..16])
}

fn parse_codex_tokens(tokens: &serde_json::Map<String, serde_json::Value>, expires: i64) -> Option<CodexCredential> {
  Some(CodexCredential {
    access: tokens.str_field("access_token")?,
    refresh: tokens.str_field("refresh_token")?,
    expires,
    account_id: tokens.str_field("account_id"),
  })
}

pub fn read_codex_credentials() -> Option<CodexCredential> {
  // Try keychain first
  if let Some(secret) = keychain::find_password(CODEX_KEYCHAIN_SERVICE, Some(&codex_keychain_account()))
    && let Ok(data) = serde_json::from_str::<serde_json::Value>(&secret)
    && let Some(tokens) = data.obj_field("tokens")
  {
    // Keychain stores last_refresh; estimate expiry as +1 hour
    let expires = data.i64_field("last_refresh").unwrap_or_else(now_ms).saturating_add(3600 * 1000);
    if let Some(cred) = parse_codex_tokens(tokens, expires) {
      return Some(cred);
    }
  }

  // Fall back to file
  let path = codex_auth_path();
  let data = read_json(&path)?;
  let tokens = data.obj_field("tokens")?;

  let expires = fs::metadata(&path)
    .and_then(|m| m.modified())
    .ok()
    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
    .map(|d| d.as_millis() as i64 + 3600 * 1000)
    .unwrap_or_else(|| now_ms() + 3600 * 1000);

  parse_codex_tokens(tokens, expires)
}
