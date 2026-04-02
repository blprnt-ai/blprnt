use std::path::PathBuf;

use directories::BaseDirs;

const BLPRNT_HOME_ENV: &str = "BLPRNT_HOME";
const MEMORY_BASE_DIR_ENV: &str = "BLPRNT_MEMORY_BASE_DIR";

fn home_dir() -> PathBuf {
  std::env::var_os(BLPRNT_HOME_ENV)
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
    .or_else(|| BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()))
    .expect("home directory should be available")
}

pub fn blprnt_home() -> PathBuf {
  home_dir().join(".blprnt")
}

fn memory_blprnt_home() -> PathBuf {
  match std::env::var_os(MEMORY_BASE_DIR_ENV) {
    Some(path) => PathBuf::from(path).join(".blprnt"),
    None => blprnt_home(),
  }
}

pub fn agents_dir() -> PathBuf {
  home_dir().join(".agents")
}

pub fn agents_skills_dir() -> PathBuf {
  agents_dir().join("skills")
}

pub fn blprnt_builtin_skills_dir() -> PathBuf {
  blprnt_home().join("skills")
}

pub fn blprnt_builtin_skills_mirror_dir() -> PathBuf {
  agents_skills_dir()
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

pub fn employee_home(employee_id: &str) -> PathBuf {
  memory_blprnt_home().join("employees").join(employee_id)
}

pub fn project_home(project_id: &str) -> PathBuf {
  memory_blprnt_home().join("projects").join(project_id)
}

pub fn employee_homes_dir() -> PathBuf {
  memory_blprnt_home().join("employees")
}

pub fn project_homes_dir() -> PathBuf {
  memory_blprnt_home().join("projects")
}

#[cfg(test)]
mod tests {
  use super::*;

  struct EnvGuard {
    key:      &'static str,
    previous: Option<std::ffi::OsString>,
  }

  impl EnvGuard {
    fn set_os(key: &'static str, value: &std::path::Path) -> Self {
      let previous = std::env::var_os(key);
      unsafe { std::env::set_var(key, value) };
      Self { key, previous }
    }

    fn remove(key: &'static str) -> Self {
      let previous = std::env::var_os(key);
      unsafe { std::env::remove_var(key) };
      Self { key, previous }
    }
  }

  impl Drop for EnvGuard {
    fn drop(&mut self) {
      match &self.previous {
        Some(value) => unsafe { std::env::set_var(self.key, value) },
        None => unsafe { std::env::remove_var(self.key) },
      }
    }
  }

  #[test]
  fn blprnt_home_prefers_blprnt_home_env_var() {
    let _home_guard = EnvGuard::set_os("HOME", std::path::Path::new("/tmp/actual-home"));
    let _blprnt_home_guard = EnvGuard::set_os("BLPRNT_HOME", std::path::Path::new("/tmp/override-home"));

    assert_eq!(blprnt_home(), PathBuf::from("/tmp/override-home").join(".blprnt"));
  }

  #[test]
  fn blprnt_home_falls_back_to_home_env_var() {
    let _blprnt_home_guard = EnvGuard::remove("BLPRNT_HOME");
    let _home_guard = EnvGuard::set_os("HOME", std::path::Path::new("/tmp/actual-home"));

    assert_eq!(blprnt_home(), PathBuf::from("/tmp/actual-home").join(".blprnt"));
  }

  #[test]
  fn employee_home_defaults_to_blprnt_home_when_memory_base_dir_is_unset() {
    let _memory_base_dir_guard = EnvGuard::remove("BLPRNT_MEMORY_BASE_DIR");
    let _blprnt_home_guard = EnvGuard::set_os("BLPRNT_HOME", std::path::Path::new("/tmp/runtime-home"));

    assert_eq!(
      employee_home("employee-123"),
      PathBuf::from("/tmp/runtime-home").join(".blprnt/employees/employee-123")
    );
  }

  #[test]
  fn project_home_uses_memory_base_dir_override_when_present() {
    let _memory_base_dir_guard = EnvGuard::set_os("BLPRNT_MEMORY_BASE_DIR", std::path::Path::new("/tmp/test-home"));
    let _blprnt_home_guard = EnvGuard::set_os("BLPRNT_HOME", std::path::Path::new("/tmp/runtime-home"));

    assert_eq!(project_home("project-123"), PathBuf::from("/tmp/test-home").join(".blprnt/projects/project-123"));
  }
}
