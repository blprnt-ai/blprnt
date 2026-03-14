use std::io::ErrorKind;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use schemars::JsonSchema;

use crate::paths::BlprntPath;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BunRuntimeCommandState {
  Available,
  Missing,
  InvocationFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct BunRuntimeCommandStatus {
  pub command:          String,
  pub detected_version: Option<String>,
  pub state:            BunRuntimeCommandState,
  pub error:            Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct BunRuntimeStatus {
  /// `bun` as resolved from PATH.
  pub bun:                 BunRuntimeCommandStatus,
  /// `bun` at `~/.local/bin/bun` (installed by blprnt). This is informational only.
  pub user_local_bun:      BunRuntimeCommandStatus,
  pub install_target_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct BunRuntimeInstallResult {
  pub status:         BunRuntimeStatus,
  pub path_help_snip: String,
}

fn user_local_bun_path() -> PathBuf {
  let home = std::env::var("HOME").unwrap_or_else(|_| "".to_string());
  if home.is_empty() {
    return PathBuf::from(".local/bin/bun");
  }
  PathBuf::from(home).join(".local").join("bin").join("bun")
}

fn detect_bun(command: &str, is_path_command: bool) -> BunRuntimeCommandStatus {
  let mut cmd = Command::new(command);
  cmd.arg("--version");
  #[cfg(windows)]
  cmd.creation_flags(CREATE_NO_WINDOW);

  match cmd.output() {
    Ok(output) if output.status.success() => {
      let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
      let version = if stdout.is_empty() { None } else { Some(stdout) };
      BunRuntimeCommandStatus {
        command:          command.to_string(),
        detected_version: version,
        state:            BunRuntimeCommandState::Available,
        error:            None,
      }
    }
    Ok(output) => {
      let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
      let err =
        if stderr.is_empty() { Some(format!("Command exited with status {}", output.status)) } else { Some(stderr) };
      BunRuntimeCommandStatus {
        command:          command.to_string(),
        detected_version: None,
        state:            BunRuntimeCommandState::InvocationFailed,
        error:            err,
      }
    }
    Err(error) if is_path_command && error.kind() == ErrorKind::NotFound => BunRuntimeCommandStatus {
      command:          command.to_string(),
      detected_version: None,
      state:            BunRuntimeCommandState::Missing,
      error:            None,
    },
    Err(error) => BunRuntimeCommandStatus {
      command:          command.to_string(),
      detected_version: None,
      state:            BunRuntimeCommandState::InvocationFailed,
      error:            Some(error.to_string()),
    },
  }
}

pub fn load_bun_runtime_status() -> BunRuntimeStatus {
  let bun = detect_bun("bun", true);
  let user_local = user_local_bun_path();
  let user_local_bun = if user_local.exists() {
    detect_bun(user_local.to_string_lossy().to_string().as_ref(), false)
  } else {
    BunRuntimeCommandStatus {
      command:          user_local.to_string_lossy().to_string(),
      detected_version: None,
      state:            BunRuntimeCommandState::Missing,
      error:            None,
    }
  };

  BunRuntimeStatus { bun, user_local_bun, install_target_path: user_local.to_string_lossy().to_string() }
}

pub async fn bun_runtime_install(overwrite: bool) -> Result<BunRuntimeInstallResult> {
  if std::env::consts::OS != "macos" && std::env::consts::OS != "linux" {
    return Err(anyhow::anyhow!("Bun install is only supported on macOS and Linux platforms"));
  }

  let source = BlprntPath::app_resources().join("bun");
  if !source.exists() {
    return Err(anyhow::anyhow!("Bundled Bun not found at {}", source.to_string_lossy()));
  }

  let dest = user_local_bun_path();
  if dest.exists() && !overwrite {
    let status = load_bun_runtime_status();
    return Ok(BunRuntimeInstallResult {
      status,
      path_help_snip: r#"export PATH="$HOME/.local/bin:$PATH""#.to_string(),
    });
  }

  if let Some(parent) = dest.parent() {
    std::fs::create_dir_all(parent)
      .map_err(|e| anyhow::anyhow!("Failed to create {}: {}", parent.to_string_lossy(), e))?;
  }

  std::fs::copy(&source, &dest)
    .map_err(|e| anyhow::anyhow!("Failed to copy bun to {}: {}", dest.to_string_lossy(), e))?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&dest)
      .map_err(|e| anyhow::anyhow!("Failed to stat {}: {}", dest.to_string_lossy(), e))?
      .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&dest, perms)
      .map_err(|e| anyhow::anyhow!("Failed to chmod {}: {}", dest.to_string_lossy(), e))?;
  }

  let status = load_bun_runtime_status();

  Ok(BunRuntimeInstallResult { status, path_help_snip: r#"export PATH="$HOME/.local/bin:$PATH""#.to_string() })
}
