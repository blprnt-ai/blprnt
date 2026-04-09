use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionCookieSameSite {
  Lax,
  Strict,
  None,
}

impl SessionCookieSameSite {
  pub(crate) fn as_cookie_value(self) -> &'static str {
    match self {
      Self::Lax => "Lax",
      Self::Strict => "Strict",
      Self::None => "None",
    }
  }
}

fn parse_bool(value: &str) -> Option<bool> {
  match value.trim().to_ascii_lowercase().as_str() {
    "1" | "true" | "yes" | "on" => Some(true),
    "0" | "false" | "no" | "off" => Some(false),
    _ => None,
  }
}

pub(crate) fn deployed_mode() -> bool {
  env::var("BLPRNT_DEPLOYED").ok().as_deref().and_then(parse_bool).unwrap_or(false)
}

pub(crate) fn allow_owner_recovery_bootstrap() -> bool {
  env::var("BLPRNT_ALLOW_OWNER_RECOVERY_BOOTSTRAP").ok().as_deref().and_then(parse_bool).unwrap_or(!deployed_mode())
}

pub(crate) fn session_cookie_secure() -> bool {
  env::var("BLPRNT_SESSION_COOKIE_SECURE").ok().as_deref().and_then(parse_bool).unwrap_or_else(deployed_mode)
}

pub(crate) fn session_cookie_same_site() -> SessionCookieSameSite {
  match env::var("BLPRNT_SESSION_COOKIE_SAME_SITE").ok().map(|value| value.trim().to_ascii_lowercase()).as_deref() {
    Some("strict") => SessionCookieSameSite::Strict,
    Some("none") => SessionCookieSameSite::None,
    _ => SessionCookieSameSite::Lax,
  }
}
