use std::path::PathBuf;

use directories::BaseDirs;

fn home_dir() -> PathBuf {
  std::env::var_os("HOME")
    .map(PathBuf::from)
    .or_else(|| BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()))
    .expect("home directory should be available")
}

pub fn blprnt_home() -> PathBuf {
  home_dir().join(".blprnt")
}

pub fn agents_dir() -> PathBuf {
  home_dir().join(".agents")
}

pub fn agents_skills_dir() -> PathBuf {
  agents_dir().join("skills")
}

pub fn blprnt_builtin_skills_dir() -> PathBuf {
  blprnt_home().join("skills").join("builtin")
}

pub fn blprnt_builtin_skills_mirror_dir() -> PathBuf {
  agents_skills_dir().join("blprnt")
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
