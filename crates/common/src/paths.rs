use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;

use anyhow::Result;
use directories::BaseDirs;

use crate::memory::MemoryPathInfo;

pub struct BlprntPath {
  pub home:          PathBuf,
  pub blprnt_home:   PathBuf,
  pub app_resources: PathBuf,
}

static BLPRNT_PATHS: OnceLock<Arc<BlprntPath>> = OnceLock::new();

pub static KEYCHAIN_NAME: &str = "keychain.v2";
pub static DATA_DIR: &str = "data";
pub static KEYCHAIN_DIR: &str = "keychain";
pub const QMD_ROOT_DIR: &str = "qmd";

static BLPRNT_HOME_NAME: &str = "ai.blprnt";

impl BlprntPath {
  pub fn home() -> PathBuf {
    BLPRNT_PATHS.get_or_init(|| Arc::new(BlprntPath::new().expect("Failed to get blprnt home"))).home.clone()
  }

  pub fn blprnt_home() -> PathBuf {
    BLPRNT_PATHS.get_or_init(|| Arc::new(BlprntPath::new().expect("Failed to get blprnt home"))).blprnt_home.clone()
  }

  pub fn app_resources() -> PathBuf {
    BLPRNT_PATHS.get_or_init(|| Arc::new(BlprntPath::new().expect("Failed to get blprnt home"))).app_resources.clone()
  }

  pub fn memories_root() -> PathBuf {
    Self::blprnt_home().join(crate::memory::MemoryContract::ROOT_DIR)
  }

  pub fn memory_path_for_date(date: chrono::NaiveDate) -> MemoryPathInfo {
    MemoryPathInfo::for_date(Self::memories_root(), date)
  }

  pub fn today_memory_path() -> MemoryPathInfo {
    Self::memory_path_for_date(crate::memory::local_today())
  }

  pub fn memory_summary_path() -> crate::memory::MemorySummaryPathInfo {
    crate::memory::MemorySummaryPathInfo::rolling(Self::memories_root())
  }

  fn new() -> Result<Self> {
    #[cfg(not(feature = "testing"))]
    let app_resources = {
      use tauri::Manager;
      use tauri::path::BaseDirectory;

      use crate::blprnt::Blprnt;

      Blprnt::handle().path().resolve("", BaseDirectory::Resource)?
    };
    #[cfg(feature = "testing")]
    let app_resources = PathBuf::from("target/debug/resources");

    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Failed to determine base directories"))?;
    let blprnt_home = base_dirs.data_dir().join(BLPRNT_HOME_NAME);
    Self::ensure_dir_exists(&blprnt_home);

    let keychain_dir = blprnt_home.join(KEYCHAIN_DIR);
    let db_dir = blprnt_home.join(DATA_DIR);
    Self::ensure_dir_exists(&keychain_dir);
    Self::ensure_dir_exists(&db_dir);

    let home = base_dirs.home_dir().to_path_buf();

    Ok(Self { home, blprnt_home, app_resources })
  }

  fn ensure_dir_exists(path: &PathBuf) {
    if !path.exists() {
      let _ = fs::create_dir_all(path);
    }
  }
}
