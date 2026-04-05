#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpenAiOauthToken {
  pub access_token:  String,
  pub refresh_token: String,
  pub expires_at_ms: u64,
  pub account_id:    Option<String>,
}

impl From<OpenAiOauthToken> for OauthToken {
  fn from(token: OpenAiOauthToken) -> Self {
    OauthToken::OpenAi(token)
  }
}

impl From<OpenAiOauthToken> for BlprntCredentials {
  fn from(token: OpenAiOauthToken) -> Self {
    BlprntCredentials::OauthToken(OauthToken::OpenAi(token))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AnthropicOauthToken {
  pub access_token:  String,
  pub refresh_token: String,
  pub expires_at_ms: u64,
}

impl From<AnthropicOauthToken> for OauthToken {
  fn from(token: AnthropicOauthToken) -> Self {
    OauthToken::Anthropic(token)
  }
}

impl From<AnthropicOauthToken> for BlprntCredentials {
  fn from(token: AnthropicOauthToken) -> Self {
    BlprntCredentials::OauthToken(OauthToken::Anthropic(token))
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthToken {
  #[serde(rename = "openai")]
  OpenAi(OpenAiOauthToken),
  Anthropic(AnthropicOauthToken),
}

impl From<OauthToken> for BlprntCredentials {
  fn from(token: OauthToken) -> Self {
    match token {
      OauthToken::OpenAi(token) => BlprntCredentials::OauthToken(token.into()),
      OauthToken::Anthropic(token) => BlprntCredentials::OauthToken(token.into()),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum BlprntCredentials {
  ApiKey(String),
  OauthToken(OauthToken),
  Unknown,
}
