#![warn(unused, unused_crate_dependencies)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

use tokio as _;

use hkdf::Hkdf;
use iota_stronghold::KeyProvider;
use iota_stronghold::Location;
use iota_stronghold::SnapshotPath;
use iota_stronghold::Stronghold;
use iota_stronghold::procedures::Runner;
use sha2::Sha256;
use shared::errors::VaultError;
use uuid::Uuid;
use zeroize::Zeroizing;

const CLIENT_ID: &str = "app";

pub enum Vault {
  Key,
}

struct StrongholdState {
  stronghold: Stronghold,
  snapshot:   SnapshotPath,
  key:        KeyProvider,
}

static VAULT_STATES: LazyLock<Mutex<HashMap<String, Arc<StrongholdState>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn set_stronghold_secret(vault: Vault, key: Uuid, value: &str) -> anyhow::Result<()> {
  tracing::trace!("Setting stronghold secret for key: {}", key);
  let item = match vault {
    Vault::Key => "provider credentials have",
  };

  let state = get_state(vault).await?;
  let client =
    state.stronghold.get_client(CLIENT_ID).map_err(|e| VaultError::FailedToGetClient { error: e.to_string() })?;
  let vault = client.vault(b"keychain");
  let store = client.store();

  let location = Location::Generic { vault_path: b"keychain".to_vec(), record_path: key.as_bytes().to_vec() };
  vault
    .write_secret(location, Zeroizing::new(value.as_bytes().to_vec()))
    .map_err(|e| VaultError::FailedToSetSecret { item: item.to_string(), error: e.to_string() })?;
  store
    .insert(key.as_bytes().to_vec(), vec![], None)
    .map_err(|e| VaultError::FailedToSetSecret { item: item.to_string(), error: e.to_string() })?;
  state
    .stronghold
    .commit_with_keyprovider(&state.snapshot, &state.key)
    .map_err(|e| VaultError::FailedToCommitSecret { error: e.to_string() })?;

  Ok(())
}

pub async fn get_stronghold_secret(vault: Vault, key: Uuid) -> Option<String> {
  tracing::debug!("Getting stronghold secret for key: {}", key);
  let state = match get_state(vault).await {
    Ok(state) => state,
    Err(error) => {
      tracing::warn!(?error, "Failed to initialize stronghold state while reading secret");
      return None;
    },
  };
  let client = state.stronghold.get_client(CLIENT_ID).ok()?;
  let location = Location::Generic { vault_path: b"keychain".to_vec(), record_path: key.as_bytes().to_vec() };

  let bytes = client
    .get_guards([location.clone()], |[buf]| Ok(buf.borrow().to_vec()))
    .map_err(|error| {
      tracing::warn!(?error, key = %key, "Failed to read stronghold secret bytes");
      error
    })
    .ok()?;

  decode_secret_bytes(&bytes)
    .map_err(|error| {
      tracing::warn!(?error, key = %key, "Failed to decode stronghold secret");
      error
    })
    .ok()
}

pub async fn delete_stronghold_secret(vault: Vault, key: Uuid) -> anyhow::Result<()> {
  let item = match vault {
    Vault::Key => "provider credentials have",
  };

  let state = get_state(vault).await?;
  let client =
    state.stronghold.get_client(CLIENT_ID).map_err(|e| VaultError::FailedToGetClient { error: e.to_string() })?;
  let vault = client.vault(b"keychain");
  let store = client.store();

  vault
    .delete_secret(key.as_bytes())
    .map_err(|e| VaultError::FailedToDeleteSecret { item: item.to_string(), error: e.to_string() })?;
  store
    .delete(key.as_bytes())
    .map_err(|e| VaultError::FailedToDeleteSecret { item: item.to_string(), error: e.to_string() })?;
  state
    .stronghold
    .commit_with_keyprovider(&state.snapshot, &state.key)
    .map_err(|e| VaultError::FailedToCommitSecret { error: e.to_string() })?;

  Ok(())
}

async fn get_state(vault: Vault) -> anyhow::Result<Arc<StrongholdState>> {
  let path = match vault {
    Vault::Key => shared::paths::blprnt_home().join(".keychain"),
  };
  let cache_key = path.to_string_lossy().to_string();

  if let Some(state) = VAULT_STATES
    .lock()
    .map_err(|_| VaultError::FailedToLockState)?
    .get(&cache_key)
    .cloned()
  {
    return Ok(state);
  }

  let state = build_state_for_path(&path)?;
  VAULT_STATES
    .lock()
    .map_err(|_| VaultError::FailedToLockState)?
    .insert(cache_key, state.clone());
  Ok(state)
}

fn build_state_for_path(path: &std::path::Path) -> anyhow::Result<Arc<StrongholdState>> {
  let snapshot = SnapshotPath::from_path(&path);
  let uid = machine_uid::get().map_err(|error| VaultError::FailedToGetMachineUid { error: error.to_string() })?;
  let key = derive_key_provider_from_uid(uid.as_bytes())?;

  let stronghold = Stronghold::default();

  if snapshot.exists() {
    let _ = stronghold.load_snapshot(&key, &snapshot);
    let _ = stronghold.load_client_from_snapshot(CLIENT_ID, &key, &snapshot);
  } else {
    stronghold
      .create_client(CLIENT_ID)
      .map_err(|error| VaultError::FailedToCreateClient { error: error.to_string() })?;
  }

  Ok(Arc::new(StrongholdState { stronghold, snapshot, key }))
}

fn derive_key_provider_from_uid(uid: &[u8]) -> anyhow::Result<KeyProvider> {
  if uid.is_empty() {
    return Err(VaultError::FailedToDeriveKeyMaterial { error: "machine UID was empty".to_string() }.into());
  }

  let hk = Hkdf::<Sha256>::new(None, uid);
  let mut derived = [0u8; 64];
  hk.expand(b"blprnt-vault-stronghold-v1", &mut derived)
    .map_err(|error| VaultError::FailedToDeriveKeyMaterial { error: error.to_string() })?;
  let pass = Zeroizing::new(derived.to_vec());
  KeyProvider::with_passphrase_hashed_blake2b(pass)
    .map_err(|error| VaultError::FailedToCreateKeyProvider { error: error.to_string() }.into())
}

fn decode_secret_bytes(bytes: &[u8]) -> anyhow::Result<String> {
  String::from_utf8(bytes.to_vec()).map_err(|error| VaultError::FailedToDecodeSecret { error: error.to_string() }.into())
}

#[cfg(test)]
mod tests {
  use super::*;

  use tempfile::tempdir;

  #[test]
  fn decode_secret_bytes_returns_error_for_invalid_utf8() {
    let error = decode_secret_bytes(&[0xff, 0xfe]).expect_err("invalid utf-8 should fail");
    let vault_error = error.downcast_ref::<VaultError>().expect("error should downcast to VaultError");
    assert!(matches!(vault_error, VaultError::FailedToDecodeSecret { .. }));
  }

  #[test]
  fn derive_key_provider_from_uid_returns_error_for_empty_uid() {
    let error = derive_key_provider_from_uid(&[]).expect_err("empty machine uid should fail");
    let vault_error = error.downcast_ref::<VaultError>().expect("error should downcast to VaultError");
    assert!(matches!(vault_error, VaultError::FailedToDeriveKeyMaterial { .. }));
  }

  #[tokio::test]
  async fn get_stronghold_secret_returns_none_for_invalid_utf8_secret_bytes() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join(".keychain");
    let snapshot = SnapshotPath::from_path(&path);
    let key = derive_key_provider_from_uid(b"vault-test-machine-uid").expect("derive key");
    let stronghold = Stronghold::default();
    let client = stronghold.create_client(CLIENT_ID).expect("create client");
    let vault = client.vault(b"keychain");
    let store = client.store();
    let secret_id = Uuid::new_v4();
    let location = Location::Generic { vault_path: b"keychain".to_vec(), record_path: secret_id.as_bytes().to_vec() };

    vault
      .write_secret(location, Zeroizing::new(vec![0xff, 0xfe, 0xfd]))
      .expect("write invalid bytes");
    store.insert(secret_id.as_bytes().to_vec(), vec![], None).expect("insert store record");
    stronghold.commit_with_keyprovider(&snapshot, &key).expect("commit snapshot");

    let previous = std::env::var_os("BLPRNT_HOME");
    unsafe { std::env::set_var("BLPRNT_HOME", temp.path()) };

    let secret = get_stronghold_secret(Vault::Key, secret_id).await;

    match previous {
      Some(value) => unsafe { std::env::set_var("BLPRNT_HOME", value) },
      None => unsafe { std::env::remove_var("BLPRNT_HOME") },
    }

    assert!(secret.is_none());
  }
}
