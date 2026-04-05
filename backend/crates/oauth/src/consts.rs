const DAYS_IN_WEEK: u64 = 7;
const WEEKS_IN_MONTH: u64 = 4;
const HOURS_IN_DAY: u64 = 24;
const MINUTES_IN_HOUR: u64 = 60;
const SECONDS_IN_MINUTE: u64 = 60;
const MS_IN_SECOND: u64 = 1000;

pub const I28_DAYS_LATER: u64 =
  WEEKS_IN_MONTH * DAYS_IN_WEEK * HOURS_IN_DAY * MINUTES_IN_HOUR * SECONDS_IN_MINUTE * MS_IN_SECOND;

pub mod keychain {
  pub const SERVICE: &str = "blprnt";
}

pub mod flow {
  pub const REDIRECT_PATH: &str = "/callback";
  pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
}

pub mod anthropic {
  pub const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
  pub const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
  pub const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
  pub const DEVICE_AUTH_URL: &str = "https://console.anthropic.com/v1/oauth/device/code";
  pub const SCOPES: &[&str] = &["org:create_api_key", "user:profile", "user:inference"];
  pub const DEFAULT_SCOPES: &[&str] = SCOPES;
}

pub mod openai {
  pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
  pub const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
  pub const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
  pub const DEVICE_AUTH_URL: &str = "https://auth.openai.com/oauth/device/code";
  pub const SCOPE: &str = "openid profile email offline_access";
  pub const ORIGINATOR: &str = "auth";
  pub const REDIRECT_PATH: &str = "/auth/callback";
}
