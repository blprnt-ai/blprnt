use std::sync::LazyLock;

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
///
/// ## Platform-specific behaviour of `machine_uid::get()`
///
/// | Platform  | Source                    | Entropy   | Notes                                          |
/// |-----------|---------------------------|-----------|-------------------------------------------------|
/// | Linux     | `/etc/machine-id`         | ~128 bits | Regenerates on OS reinstall                     |
/// | macOS     | `IOPlatformUUID`          | ~122 bits | Stable across reinstalls                        |
/// | Windows   | `MachineGuid` (registry)  | ~122 bits | Can be cloned via disk images / VM snapshots    |
/// | illumos   | `gethostid(3C)`           | ~32 bits  | **Critically low — brute-forceable in seconds** |
///
/// On any platform, the UID may change after OS reinstall or VM migration,
/// making previously encrypted data unrecoverable. This is acceptable for
/// a local-first desktop app where the data is meant to stay on-device.
fn derive_key() -> Result<[u8; 32]> {
  let uid = machine_uid::get()
    .map_err(|e| anyhow!("failed to obtain machine UID: {e}\n\
      hint: on Linux, ensure /etc/machine-id or /var/lib/dbus/machine-id exists; \
      in containers, mount the host machine-id or set BLPRNT_MACHINE_UID"))?;
  let hk = Hkdf::<Sha256>::new(None, uid.as_bytes());
  let mut key = [0u8; 32];
  hk.expand(b"blprnt-quick-encrypt-v1", &mut key)
    .map_err(|e| anyhow!("HKDF expand failed: {e}"))?;
  Ok(key)
}

static DERIVED_KEY: LazyLock<Result<[u8; 32], String>> = LazyLock::new(|| {
  derive_key().map_err(|e| e.to_string())
});

fn cached_key() -> Result<[u8; 32]> {
  DERIVED_KEY.clone().map_err(|e| anyhow!("{e}"))
}

const LEGACY_KEY: &[u8; 32] = b"4ccd22dfd5df6501780eb1dfb67b6818";

fn try_decrypt_with_key(key_bytes: &[u8; 32], nonce_bytes: &[u8], ciphertext: &[u8]) -> Result<String> {
  let cipher_key = Key::<Aes256Gcm>::from_slice(key_bytes);
  let cipher = Aes256Gcm::new(cipher_key);
  let nonce = Nonce::from_slice(nonce_bytes);
  let plaintext_bytes = cipher.decrypt(nonce, ciphertext).map_err(|error| anyhow!("decryption failed: {error}"))?;
  Ok(String::from_utf8(plaintext_bytes)?)
}

pub fn encrypt_string(plaintext: &str) -> Result<String> {
  let key_bytes = cached_key()?;
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

/// Decrypts data, transparently migrating from the legacy hardcoded key.
///
/// Tries the new HKDF-derived key first. If that fails, falls back to the
/// old hardcoded key. On successful legacy decryption the caller gets the
/// plaintext back and can re-encrypt it with `encrypt_string` to complete
/// the migration.
pub fn decrypt_string(encoded_data: &str) -> Result<String> {
  let combined_bytes = STANDARD.decode(encoded_data)?;
  if combined_bytes.len() < NONCE_LEN {
    return Err(anyhow!("ciphertext too short"));
  }

  let (nonce_bytes, ciphertext) = combined_bytes.split_at(NONCE_LEN);

  let new_key = cached_key()?;
  if let Ok(plaintext) = try_decrypt_with_key(&new_key, nonce_bytes, ciphertext) {
    return Ok(plaintext);
  }

  tracing::warn!("decryption with derived key failed, attempting legacy key migration");
  try_decrypt_with_key(LEGACY_KEY, nonce_bytes, ciphertext)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encrypt_decrypt_round_trip() {
    let plaintext = "hello, blprnt!";
    let encrypted = encrypt_string(plaintext).expect("encryption should succeed");
    assert_ne!(encrypted, plaintext, "ciphertext should differ from plaintext");
    let decrypted = decrypt_string(&encrypted).expect("decryption should succeed");
    assert_eq!(decrypted, plaintext);
  }

  #[test]
  fn encrypt_decrypt_empty_string() {
    let encrypted = encrypt_string("").expect("encrypting empty string should succeed");
    let decrypted = decrypt_string(&encrypted).expect("decrypting empty string should succeed");
    assert_eq!(decrypted, "");
  }

  #[test]
  fn encrypt_decrypt_unicode() {
    let plaintext = "こんにちは世界 🌍 émojis & spëcial chars";
    let encrypted = encrypt_string(plaintext).expect("encryption should succeed");
    let decrypted = decrypt_string(&encrypted).expect("decryption should succeed");
    assert_eq!(decrypted, plaintext);
  }

  #[test]
  fn derived_key_is_deterministic() {
    let key1 = cached_key().expect("first key derivation should succeed");
    let key2 = cached_key().expect("second key derivation should succeed");
    assert_eq!(key1, key2, "derived key must be stable across calls");
  }

  #[test]
  fn derived_key_is_32_bytes() {
    let key = cached_key().expect("key derivation should succeed");
    assert_eq!(key.len(), 32);
  }

  #[test]
  fn decrypt_invalid_base64_fails() {
    assert!(decrypt_string("not-valid-base64!!!").is_err());
  }

  #[test]
  fn decrypt_too_short_ciphertext_fails() {
    let short = STANDARD.encode([0u8; 4]);
    assert!(decrypt_string(&short).is_err());
  }

  #[test]
  fn each_encryption_produces_different_ciphertext() {
    let plaintext = "determinism check";
    let a = encrypt_string(plaintext).unwrap();
    let b = encrypt_string(plaintext).unwrap();
    assert_ne!(a, b, "random nonce should make each ciphertext unique");
  }
}
