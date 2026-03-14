use std::collections::HashMap;
use std::sync::OnceLock;

static ENV_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Get environment variables with user shell PATH merged in (cached).
/// Also sets environment variables that signal non-interactive mode to common tools.
pub fn get_env() -> &'static HashMap<String, String> {
  ENV_CACHE.get_or_init(|| {
    #[cfg(target_os = "windows")]
    let mut env = std::env::vars().collect::<HashMap<String, String>>();
    #[cfg(not(target_os = "windows"))]
    let mut env = std::env::vars().collect::<HashMap<String, String>>();

    #[cfg(target_os = "macos")]
    if let Some(shell_path) = load_macos_shell_path() {
      env.insert("PATH".to_string(), shell_path);
    }

    #[cfg(target_os = "linux")]
    if let Some(shell_path) = load_linux_shell_path() {
      env.insert("PATH".to_string(), shell_path);
    }

    // Set environment variables that signal non-interactive mode to common tools.
    // These cause tools to use sensible defaults instead of prompting.

    // CI=true is widely respected by npm, yarn, pnpm, cargo, etc.
    env.insert("CI".to_string(), "true".to_string());

    // npm-specific: auto-yes for prompts
    env.insert("npm_config_yes".to_string(), "true".to_string());

    // Disable interactive prompts in various tools
    env.insert("DEBIAN_FRONTEND".to_string(), "noninteractive".to_string());

    // Cargo: don't prompt
    env.insert("CARGO_TERM_COLOR".to_string(), "never".to_string());

    // Python/pip: don't prompt
    env.insert("PIP_DISABLE_PIP_VERSION_CHECK".to_string(), "1".to_string());

    // Git: don't prompt for credentials or other input
    env.insert("GIT_TERMINAL_PROMPT".to_string(), "0".to_string());

    // Homebrew: don't prompt
    env.insert("HOMEBREW_NO_AUTO_UPDATE".to_string(), "1".to_string());
    env.insert("NONINTERACTIVE".to_string(), "1".to_string());

    env
  })
}

#[cfg(target_os = "macos")]
fn load_macos_shell_path() -> Option<String> {
  use std::env;
  use std::process::Command;

  let home = env::var("HOME").ok()?;

  // Respect user's SHELL env var first
  if let Ok(shell) = env::var("SHELL") {
    if shell.contains("bash") {
      // For bash, explicitly source profile files
      let bash_cmd = r#"[ -f ~/.bash_profile ] && source ~/.bash_profile;
         [ -f ~/.bashrc ] && source ~/.bashrc;
         echo $PATH"#;

      if let Ok(output) = Command::new(&shell).args(["-c", bash_cmd]).output()
        && output.status.success()
      {
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim();
        if !path.is_empty() {
          return Some(path.to_string());
        }
      }
    } else if shell.contains("zsh") {
      // For zsh, source zshrc and zprofile
      let zsh_cmd = r#"[ -f ~/.zshrc ] && source ~/.zshrc; \
         [ -f ~/.zprofile ] && source ~/.zprofile; \
         echo $PATH"#;

      if let Ok(output) = Command::new(&shell).args(["-c", zsh_cmd]).output()
        && output.status.success()
      {
        let path = String::from_utf8(output.stdout).ok()?;
        let path = path.trim();
        if !path.is_empty() {
          return Some(path.to_string());
        }
      }
    }

    // Generic fallback for other shells: use login shell
    if let Ok(output) = Command::new(&shell).args(["-l", "-c", "echo $PATH"]).output()
      && output.status.success()
    {
      let path = String::from_utf8(output.stdout).ok()?;
      let path = path.trim();
      if !path.is_empty() {
        return Some(path.to_string());
      }
    }
  }

  // Fallback: try bash with explicit profile sourcing
  let bash_cmd = format!(
    "[ -f {home}/.bash_profile ] && source {home}/.bash_profile; \
     [ -f {home}/.bashrc ] && source {home}/.bashrc; \
     echo $PATH"
  );

  if let Ok(output) = Command::new("/bin/bash").args(["-c", &bash_cmd]).output()
    && output.status.success()
  {
    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.trim();
    if !path.is_empty() {
      return Some(path.to_string());
    }
  }

  None
}

#[cfg(target_os = "linux")]
fn load_linux_shell_path() -> Option<String> {
  use std::process::Command;

  // Try to load PATH from user's login shell
  let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

  let output = Command::new(&shell).args(["-l", "-c", "echo $PATH"]).output().ok()?;

  if output.status.success() {
    let path = String::from_utf8(output.stdout).ok()?;
    let path = path.trim();
    if !path.is_empty() {
      return Some(path.to_string());
    }
  }

  None
}
