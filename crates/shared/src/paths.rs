use std::path::PathBuf;

use directories::BaseDirs;

pub fn blprnt_home() -> PathBuf {
  let base_dirs = BaseDirs::new().unwrap();
  base_dirs.home_dir().join(".blprnt")
}

pub fn executable_dir() -> Option<PathBuf> {
  std::env::current_exe().ok()?.parent().map(|path| path.to_path_buf())
}

pub fn bundled_tools_dir() -> Option<PathBuf> {
  let tools_dir = executable_dir()?.join("tools");
  tools_dir.is_dir().then_some(tools_dir)
}

pub fn bundled_rg_path() -> Option<PathBuf> {
  let file_name = if cfg!(target_os = "windows") { "rg.exe" } else { "rg" };
  let rg_path = bundled_tools_dir()?.join(file_name);
  rg_path.is_file().then_some(rg_path)
}
