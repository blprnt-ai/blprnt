use aes_gcm::Aes256Gcm;
use aes_gcm::Key;
use aes_gcm::Nonce;
use aes_gcm::aead::Aead;
use aes_gcm::aead::KeyInit;
use anyhow::Result;
use anyhow::anyhow;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use rand::RngCore;

const NONCE_LEN: usize = 12;
const ENCRYPTION_KEY: &[u8; 32] = b"4ccd22dfd5df6501780eb1dfb67b6818";

pub fn encrypt_string(plaintext: &str) -> Result<String> {
  let cipher_key = Key::<Aes256Gcm>::from_slice(ENCRYPTION_KEY);
  let cipher = Aes256Gcm::new(cipher_key);

  let mut nonce_bytes = [0u8; NONCE_LEN];
  rand::rng().fill_bytes(&mut nonce_bytes);

  let nonce = Nonce::from_slice(&nonce_bytes);
  let ciphertext =
    cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|error| anyhow!("encryption failed: {error}"))?;

  let mut combined_bytes = Vec::with_capacity(NONCE_LEN + ciphertext.len());
  combined_bytes.extend_from_slice(&nonce_bytes);
  combined_bytes.extend_from_slice(&ciphertext);

  Ok(STANDARD.encode(combined_bytes))
}

pub fn decrypt_string(encoded_data: &str) -> Result<String> {
  let combined_bytes = STANDARD.decode(encoded_data)?;
  if combined_bytes.len() < NONCE_LEN {
    return Err(anyhow!("ciphertext too short"));
  }

  let (nonce_bytes, ciphertext) = combined_bytes.split_at(NONCE_LEN);

  let cipher_key = Key::<Aes256Gcm>::from_slice(ENCRYPTION_KEY);
  let cipher = Aes256Gcm::new(cipher_key);

  let nonce = Nonce::from_slice(nonce_bytes);
  let plaintext_bytes = cipher.decrypt(nonce, ciphertext).map_err(|error| anyhow!("decryption failed: {error}"))?;

  let plaintext = String::from_utf8(plaintext_bytes)?;

  Ok(plaintext)
}
