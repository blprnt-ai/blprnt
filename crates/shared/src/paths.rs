use std::path::PathBuf;

use directories::BaseDirs;

pub fn blprnt_home() -> PathBuf {
  let base_dirs = BaseDirs::new().unwrap();
  base_dirs.home_dir().join(".blprnt")
}
