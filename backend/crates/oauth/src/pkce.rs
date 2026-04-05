use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::Digest;
use sha2::Sha256;

pub struct Pkce;

impl Pkce {
  pub fn generate_verifier() -> String {
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);
    Self::b64url(&buf)
  }

  pub fn challenge_from_verifier(verifier: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    Ok(Self::b64url(&hash))
  }

  pub fn generate() -> Result<(String, String)> {
    let v = Self::generate_verifier();
    let c = Self::challenge_from_verifier(&v)?;
    Ok((v, c))
  }

  fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
  }
}
