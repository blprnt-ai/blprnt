#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenResponse {
  pub access_token:  String,
  pub refresh_token: String,
  pub id_token:      Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExchangedTokens {
  pub access_token:  String,
  pub refresh_token: String,
  pub account_id:    Option<String>,
}
