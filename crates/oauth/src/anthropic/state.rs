#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenResponse {
  pub access_token:  String,
  #[serde(default)]
  pub refresh_token: String,
  pub expires_in:    u64,
}
