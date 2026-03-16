use aes_gcm::Aes256Gcm;
use aes_gcm::Key;
use aes_gcm::Nonce;
use aes_gcm::aead::Aead;
use aes_gcm::aead::KeyInit;
use anyhow::Result;
use anyhow::anyhow;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

const NONCE_LEN: usize = 12;

/// Derives a 32-byte encryption key from the machine's unique identifier
/// using HKDF-SHA256 (RFC 5869) with an application-specific info string
/// for domain separation. The key is stable across restarts but different
/// on every machine, so encrypted data cannot be decrypted elsewhere.
fn derive_key() -> Result<[u8; 32]> {
  let uid = machine_uid::get()
    .map_err(|e| anyhow!("failed to obtain machine UID: {e}"))?;
  let hk = Hkdf::<Sha256>::new(None, uid.as_bytes());
  let mut key = [0u8; 32];
  hk.expand(b"blprnt-quick-encrypt-v1", &mut key)
    .map_err(|e| anyhow!("HKDF expand failed: {e}"))?;
  Ok(key)
}

pub fn encrypt_string(plaintext: &str) -> Result<String> {
  let key_bytes = derive_key()?;
  let cipher_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
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

  let key_bytes = derive_key()?;
  let cipher_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
  let cipher = Aes256Gcm::new(cipher_key);

  let nonce = Nonce::from_slice(nonce_bytes);
  let plaintext_bytes = cipher.decrypt(nonce, ciphertext).map_err(|error| anyhow!("decryption failed: {error}"))?;

  let plaintext = String::from_utf8(plaintext_bytes)?;

  Ok(plaintext)
}
