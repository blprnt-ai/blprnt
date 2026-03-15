#![warn(unused, unused_crate_dependencies)]

use std::sync::Arc;

use common::paths::BlprntPath;
use common::paths::KEYCHAIN_DIR;
use common::paths::KEYCHAIN_NAME;
use hkdf::Hkdf;
use iota_stronghold::KeyProvider;
use iota_stronghold::Location;
use iota_stronghold::SnapshotPath;
use iota_stronghold::Stronghold;
use iota_stronghold::procedures::Runner;
use lazy_static::lazy_static;
use sha2::Sha256;
use surrealdb::types::Uuid;
use tokio::sync::OnceCell;
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

lazy_static! {
  static ref KEY_VAULT: OnceCell<Arc<StrongholdState>> = OnceCell::new();
  static ref LICENSE_VAULT: OnceCell<Arc<StrongholdState>> = OnceCell::new();
  static ref GENERATE_LICENSE_VAULT: OnceCell<Arc<StrongholdState>> = OnceCell::new();
}

pub async fn set_stronghold_secret(vault: Vault, key: Uuid, value: &str) -> anyhow::Result<()> {
  tracing::trace!("Setting stronghold secret for key: {}", key);
  let item = match vault {
    Vault::Key => "provider credentials have",
  };

  let state = get_state(vault).await;
  let client = state
    .stronghold
    .get_client(CLIENT_ID)
    .map_err(|e| common::errors::vault::VaultError::FailedToGetClient { error: e.to_string() })?;
  let vault = client.vault(b"keychain");
  let store = client.store();

  let location = Location::Generic { vault_path: b"keychain".to_vec(), record_path: key.as_bytes().to_vec() };
  vault.write_secret(location, Zeroizing::new(value.as_bytes().to_vec())).map_err(|e| {
    common::errors::vault::VaultError::FailedToSetSecret { item: item.to_string(), error: e.to_string() }
  })?;
  store.insert(key.as_bytes().to_vec(), vec![], None).map_err(|e| {
    common::errors::vault::VaultError::FailedToSetSecret { item: item.to_string(), error: e.to_string() }
  })?;
  state
    .stronghold
    .commit_with_keyprovider(&state.snapshot, &state.key)
    .map_err(|e| common::errors::vault::VaultError::FailedToCommitSecret { error: e.to_string() })?;

  Ok(())
}

pub async fn get_stronghold_secret(vault: Vault, key: Uuid) -> Option<String> {
  tracing::debug!("Getting stronghold secret for key: {}", key);
  let state = get_state(vault).await;
  let client = state.stronghold.get_client(CLIENT_ID).ok()?;
  let location = Location::Generic { vault_path: b"keychain".to_vec(), record_path: key.as_bytes().to_vec() };

  client.get_guards([location.clone()], |[buf]| Ok(String::from_utf8(buf.borrow().to_vec()).unwrap())).ok()
}

pub async fn delete_stronghold_secret(vault: Vault, key: Uuid) -> anyhow::Result<()> {
  let item = match vault {
    Vault::Key => "provider credentials have",
  };

  let state = get_state(vault).await;
  let client = state
    .stronghold
    .get_client(CLIENT_ID)
    .map_err(|e| common::errors::vault::VaultError::FailedToGetClient { error: e.to_string() })?;
  let vault = client.vault(b"keychain");
  let store = client.store();

  vault.delete_secret(key.as_bytes()).map_err(|e| common::errors::vault::VaultError::FailedToDeleteSecret {
    item:  item.to_string(),
    error: e.to_string(),
  })?;
  store.delete(key.as_bytes()).map_err(|e| common::errors::vault::VaultError::FailedToDeleteSecret {
    item:  item.to_string(),
    error: e.to_string(),
  })?;
  state
    .stronghold
    .commit_with_keyprovider(&state.snapshot, &state.key)
    .map_err(|e| common::errors::vault::VaultError::FailedToCommitSecret { error: e.to_string() })?;

  Ok(())
}

async fn get_state(vault: Vault) -> Arc<StrongholdState> {
  match vault {
    Vault::Key => KEY_VAULT
      .get_or_init(|| async {
        let path = BlprntPath::blprnt_home().join(KEYCHAIN_DIR).join(KEYCHAIN_NAME);

        let snapshot = SnapshotPath::from_path(&path);
        let uid = machine_uid::get().expect("failed to obtain machine UID");
        let hk = Hkdf::<Sha256>::new(None, uid.as_bytes());
        let mut derived = [0u8; 64];
        hk.expand(b"blprnt-vault-stronghold-v1", &mut derived)
          .expect("HKDF expand failed");
        let pass = Zeroizing::new(derived.to_vec());
        let key = KeyProvider::with_passphrase_hashed_blake2b(pass).unwrap();

        let stronghold = Stronghold::default();

        if snapshot.exists() {
          let _ = stronghold.load_snapshot(&key, &snapshot);
          let _ = stronghold.load_client_from_snapshot(CLIENT_ID, &key, &snapshot);
        } else {
          let _ = stronghold.create_client(CLIENT_ID);
        }

        Arc::new(StrongholdState { stronghold, snapshot, key })
      })
      .await
      .clone()
  }
}
