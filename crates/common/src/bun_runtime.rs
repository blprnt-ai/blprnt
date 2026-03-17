use std::io::ErrorKind;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use schemars::JsonSchema;

use crate::memory::managed_qmd_display_path;
use crate::memory::resolve_existing_managed_qmd_path;
use crate::memory::QmdMemoryReadiness;
use crate::memory::QmdMemoryReadinessState;
use crate::paths::BlprntPath;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const WINDOWS_CANONICAL_MANAGED_BUN_DIR: &str = "ai.blprnt";
const WINDOWS_LEGACY_MANAGED_BUN_DIR: &str = "blprnt";
const UNIX_PATH_HELP_SNIP: &str = r#"export PATH="$HOME/.local/bin:$PATH""#;

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
  /// blprnt-managed Bun installation, including legacy Windows installs when detected.
  pub user_local_bun:      BunRuntimeCommandStatus,
  pub install_target_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct BunRuntimeInstallResult {
  pub status:         BunRuntimeStatus,
  pub path_help_snip: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JsRuntimeKind {
  Bun,
  Node,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JsRuntimeSource {
  Path,
  Managed,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JsRuntimeRecommendedActionType {
  None,
  InstallManaged,
  AddToPath,
  ManualInstall,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct JsRuntimeCommandStatus {
  pub command:          String,
  pub detected_version: Option<String>,
  pub state:            BunRuntimeCommandState,
  pub error:            Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct ActiveJsRuntime {
  pub kind:    JsRuntimeKind,
  pub source:  JsRuntimeSource,
  pub command: String,
  pub version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct JsRuntimeRecommendedAction {
  pub r#type: JsRuntimeRecommendedActionType,
  pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct JsRuntimeHealthStatus {
  pub runtime_on_path:      JsRuntimeCommandStatus,
  pub managed_runtime:      JsRuntimeCommandStatus,
  pub managed_runtime_path: String,
  pub active_runtime:       Option<ActiveJsRuntime>,
  pub install_supported:    bool,
  pub path_help_snip:       Option<String>,
  pub qmd_readiness:        QmdMemoryReadiness,
  pub recommended_action:   JsRuntimeRecommendedAction,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type, JsonSchema)]
pub struct JsRuntimeInstallResult {
  pub status:         JsRuntimeHealthStatus,
  pub path_help_snip: Option<String>,
}

fn bundled_bun_resource_name_for_os(os: &str) -> &'static str {
  if os == "windows" { "bun.exe" } else { "bun" }
}

fn bundled_bun_resource_path() -> PathBuf {
  BlprntPath::app_resources().join(bundled_bun_resource_name_for_os(std::env::consts::OS))
}

fn home_dir() -> PathBuf {
  std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn local_app_data_dir() -> Option<PathBuf> {
  std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

fn managed_bun_install_target_for_os(
  os: &str,
  home: &std::path::Path,
  local_app_data: Option<&std::path::Path>,
) -> PathBuf {
  if os == "windows" {
    let local_app_data = local_app_data.map(PathBuf::from).unwrap_or_else(|| home.join("AppData").join("Local"));
    return local_app_data.join(WINDOWS_CANONICAL_MANAGED_BUN_DIR).join("runtime").join("bun").join("bun.exe");
  }

  if home.as_os_str().is_empty() {
    return PathBuf::from(".local").join("bin").join("bun");
  }

  home.join(".local").join("bin").join("bun")
}

fn managed_bun_probe_paths_for_os(
  os: &str,
  home: &std::path::Path,
  local_app_data: Option<&std::path::Path>,
) -> Vec<PathBuf> {
  let install_target = managed_bun_install_target_for_os(os, home, local_app_data);
  if os != "windows" {
    return vec![install_target];
  }

  let legacy_target = local_app_data
    .map(PathBuf::from)
    .unwrap_or_else(|| home.join("AppData").join("Local"))
    .join(WINDOWS_LEGACY_MANAGED_BUN_DIR)
    .join("bun")
    .join("bun.exe");

  if legacy_target == install_target {
    return vec![install_target];
  }

  vec![install_target, legacy_target]
}

fn managed_bun_install_target() -> PathBuf {
  let home = home_dir();
  let local_app_data = local_app_data_dir();
  managed_bun_install_target_for_os(std::env::consts::OS, &home, local_app_data.as_deref())
}

fn managed_bun_probe_paths() -> Vec<PathBuf> {
  let home = home_dir();
  let local_app_data = local_app_data_dir();
  managed_bun_probe_paths_for_os(std::env::consts::OS, &home, local_app_data.as_deref())
}

fn qmd_path() -> PathBuf {
  managed_qmd_display_path()
}

fn install_supported_for_os(os: &str) -> bool {
  matches!(os, "macos" | "linux" | "windows")
}

fn install_supported() -> bool {
  install_supported_for_os(std::env::consts::OS)
}

fn path_help_snip_for_os(os: &str) -> Option<String> {
  if matches!(os, "macos" | "linux") { Some(UNIX_PATH_HELP_SNIP.to_string()) } else { None }
}

fn path_help_snip() -> Option<String> {
  path_help_snip_for_os(std::env::consts::OS)
}

fn detect_managed_bun() -> BunRuntimeCommandStatus {
  let install_target = managed_bun_install_target();

  for candidate in managed_bun_probe_paths() {
    if candidate.exists() {
      return detect_bun(candidate.to_string_lossy().as_ref(), false);
    }
  }

  BunRuntimeCommandStatus {
    command:          install_target.to_string_lossy().to_string(),
    detected_version: None,
    state:            BunRuntimeCommandState::Missing,
    error:            None,
  }
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

fn detect_node(command: &str, is_path_command: bool) -> BunRuntimeCommandStatus {
  detect_bun(command, is_path_command)
}

fn to_js_runtime_command_status(status: BunRuntimeCommandStatus) -> JsRuntimeCommandStatus {
  JsRuntimeCommandStatus {
    command:          status.command,
    detected_version: status.detected_version,
    state:            status.state,
    error:            status.error,
  }
}

fn preferred_path_runtime(
  bun_status: BunRuntimeCommandStatus,
  node_status: BunRuntimeCommandStatus,
) -> (JsRuntimeCommandStatus, Option<JsRuntimeKind>) {
  if bun_status.state == BunRuntimeCommandState::Available {
    return (to_js_runtime_command_status(bun_status), Some(JsRuntimeKind::Bun));
  }

  if node_status.state == BunRuntimeCommandState::Available {
    return (to_js_runtime_command_status(node_status), Some(JsRuntimeKind::Node));
  }

  if bun_status.state == BunRuntimeCommandState::InvocationFailed {
    return (to_js_runtime_command_status(bun_status), Some(JsRuntimeKind::Bun));
  }

  if node_status.state == BunRuntimeCommandState::InvocationFailed {
    return (to_js_runtime_command_status(node_status), Some(JsRuntimeKind::Node));
  }

  (to_js_runtime_command_status(bun_status), None)
}

fn active_runtime(
  runtime_on_path: &JsRuntimeCommandStatus,
  runtime_on_path_kind: Option<JsRuntimeKind>,
  managed_runtime: &JsRuntimeCommandStatus,
) -> Option<ActiveJsRuntime> {
  if runtime_on_path.state == BunRuntimeCommandState::Available {
    return Some(ActiveJsRuntime {
      kind:    runtime_on_path_kind.unwrap_or(JsRuntimeKind::Bun),
      source:  JsRuntimeSource::Path,
      command: runtime_on_path.command.clone(),
      version: runtime_on_path.detected_version.clone().unwrap_or_else(|| "unknown".to_string()),
    });
  }

  if managed_runtime.state == BunRuntimeCommandState::Available {
    return Some(ActiveJsRuntime {
      kind:    JsRuntimeKind::Bun,
      source:  JsRuntimeSource::Managed,
      command: managed_runtime.command.clone(),
      version: managed_runtime.detected_version.clone().unwrap_or_else(|| "unknown".to_string()),
    });
  }

  None
}

fn qmd_version_suffix(status: &BunRuntimeCommandStatus) -> String {
  status.detected_version.as_ref().map(|version| format!(" ({version})")).unwrap_or_default()
}

fn detect_qmd_readiness(active_runtime: Option<&ActiveJsRuntime>) -> QmdMemoryReadiness {
  let qmd_on_path = detect_bun("qmd", true);
  if qmd_on_path.state == BunRuntimeCommandState::Available {
    return QmdMemoryReadiness {
      state:  QmdMemoryReadinessState::Ready,
      detail: format!("QMD is available on PATH{}", qmd_version_suffix(&qmd_on_path)),
    };
  }

  if qmd_on_path.state == BunRuntimeCommandState::InvocationFailed {
    return QmdMemoryReadiness {
      state:  QmdMemoryReadinessState::QmdUnavailable,
      detail: qmd_on_path.error.unwrap_or_else(|| "QMD could not be invoked from PATH.".to_string()),
    };
  }

  let managed_qmd_path = resolve_existing_managed_qmd_path().unwrap_or_else(qmd_path);
  let managed_qmd = if managed_qmd_path.exists() {
    detect_bun(managed_qmd_path.to_string_lossy().as_ref(), false)
  } else {
    BunRuntimeCommandStatus {
      command:          managed_qmd_path.to_string_lossy().to_string(),
      detected_version: None,
      state:            BunRuntimeCommandState::Missing,
      error:            None,
    }
  };

  match managed_qmd.state {
    BunRuntimeCommandState::Available => QmdMemoryReadiness {
      state:  QmdMemoryReadinessState::Ready,
      detail: format!("QMD is available at {}{}", managed_qmd.command, qmd_version_suffix(&managed_qmd)),
    },
    BunRuntimeCommandState::Missing => {
      if active_runtime.is_none() {
        return QmdMemoryReadiness {
          state:  QmdMemoryReadinessState::RuntimeMissing,
          detail: "No JavaScript runtime is currently available for QMD.".to_string(),
        };
      }

      QmdMemoryReadiness {
        state:  QmdMemoryReadinessState::QmdMissingFromPath,
        detail: format!("QMD was not found on PATH or at {}", managed_qmd.command),
      }
    }
    BunRuntimeCommandState::InvocationFailed => QmdMemoryReadiness {
      state:  QmdMemoryReadinessState::QmdUnavailable,
      detail: managed_qmd.error.unwrap_or_else(|| "QMD could not be invoked.".to_string()),
    },
  }
}

fn recommended_action(
  runtime_on_path: &JsRuntimeCommandStatus,
  managed_runtime: &JsRuntimeCommandStatus,
  qmd_readiness: &QmdMemoryReadiness,
) -> JsRuntimeRecommendedAction {
  if managed_runtime.state == BunRuntimeCommandState::Available
    && runtime_on_path.state != BunRuntimeCommandState::Available
  {
    return JsRuntimeRecommendedAction {
      r#type: JsRuntimeRecommendedActionType::AddToPath,
      detail: "Add the managed runtime to PATH and relaunch blprnt.".to_string(),
    };
  }

  let runtime_available = runtime_on_path.state == BunRuntimeCommandState::Available
    || managed_runtime.state == BunRuntimeCommandState::Available;

  if !runtime_available {
    if install_supported() {
      return JsRuntimeRecommendedAction {
        r#type: JsRuntimeRecommendedActionType::InstallManaged,
        detail: "Install the managed runtime to restore JavaScript workflows.".to_string(),
      };
    }

    return JsRuntimeRecommendedAction {
      r#type: JsRuntimeRecommendedActionType::ManualInstall,
      detail: "Install Bun manually, then relaunch blprnt.".to_string(),
    };
  }

  if qmd_readiness.state != QmdMemoryReadinessState::Ready {
    return JsRuntimeRecommendedAction {
      r#type: JsRuntimeRecommendedActionType::None,
      detail: "Refresh after relaunching blprnt to re-check QMD readiness.".to_string(),
    };
  }

  JsRuntimeRecommendedAction { r#type: JsRuntimeRecommendedActionType::None, detail: "Runtime is ready.".to_string() }
}

fn selected_runtime_command(status: &BunRuntimeStatus) -> Option<String> {
  if status.bun.state == BunRuntimeCommandState::Available {
    return Some(status.bun.command.clone());
  }

  if status.user_local_bun.state == BunRuntimeCommandState::Available {
    return Some(status.user_local_bun.command.clone());
  }

  None
}

fn ensure_qmd_installed(runtime_command: &str) -> Result<()> {
  let qmd_on_path = detect_bun("qmd", true);
  if qmd_on_path.state == BunRuntimeCommandState::Available || resolve_existing_managed_qmd_path().is_some() {
    return Ok(());
  }

  let mut cmd = Command::new(runtime_command);
  cmd.arg("install").arg("-g").arg("@tobilu/qmd");

  #[cfg(windows)]
  cmd.creation_flags(CREATE_NO_WINDOW);

  let output = cmd.output().map_err(|error| anyhow::anyhow!("Failed to install QMD: {error}"))?;
  if output.status.success() {
    return Ok(());
  }

  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
  let detail = if !stderr.is_empty() {
    stderr
  } else if !stdout.is_empty() {
    stdout
  } else {
    format!("Command exited with status {}", output.status)
  };

  Err(anyhow::anyhow!("Failed to install QMD: {detail}"))
}

pub fn load_bun_runtime_status() -> BunRuntimeStatus {
  let bun = detect_bun("bun", true);
  let install_target = managed_bun_install_target();
  let user_local_bun = detect_managed_bun();

  BunRuntimeStatus { bun, user_local_bun, install_target_path: install_target.to_string_lossy().to_string() }
}

pub async fn bun_runtime_install(overwrite: bool) -> Result<BunRuntimeInstallResult> {
  if !install_supported() {
    return Err(anyhow::anyhow!("Bun install is only supported on macOS, Linux, and Windows platforms"));
  }

  let source = bundled_bun_resource_path();
  if !source.exists() {
    return Err(anyhow::anyhow!("Bundled Bun not found at {}", source.to_string_lossy()));
  }

  let dest = managed_bun_install_target();
  if dest.exists() && !overwrite {
    let status = load_bun_runtime_status();
    return Ok(BunRuntimeInstallResult {
      status,
      path_help_snip: path_help_snip().unwrap_or_default(),
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

  if let Some(runtime_command) = selected_runtime_command(&status) {
    let _ = ensure_qmd_installed(&runtime_command);
  }

  Ok(BunRuntimeInstallResult {
    status,
    path_help_snip: path_help_snip().unwrap_or_default(),
  })
}

pub fn load_js_runtime_health_status() -> JsRuntimeHealthStatus {
  let bun_status = load_bun_runtime_status();
  let (runtime_on_path, runtime_on_path_kind) =
    preferred_path_runtime(detect_bun("bun", true), detect_node("node", true));
  let managed_runtime = to_js_runtime_command_status(bun_status.user_local_bun.clone());
  let active_runtime = active_runtime(&runtime_on_path, runtime_on_path_kind, &managed_runtime);
  let qmd_readiness = detect_qmd_readiness(active_runtime.as_ref());
  let recommended_action = recommended_action(&runtime_on_path, &managed_runtime, &qmd_readiness);

  JsRuntimeHealthStatus {
    runtime_on_path,
    managed_runtime,
    managed_runtime_path: bun_status.install_target_path,
    active_runtime,
    install_supported: install_supported(),
    path_help_snip: path_help_snip(),
    qmd_readiness,
    recommended_action,
  }
}

pub async fn js_runtime_install_managed(overwrite: bool) -> Result<JsRuntimeInstallResult> {
  let bun_install = bun_runtime_install(overwrite).await?;

  Ok(JsRuntimeInstallResult {
    status:         load_js_runtime_health_status(),
    path_help_snip: path_help_snip().filter(|_| !bun_install.path_help_snip.is_empty()),
  })
}

pub async fn ensure_qmd_runtime_ready() -> Result<JsRuntimeHealthStatus> {
  let status = load_js_runtime_health_status();
  if status.qmd_readiness.state == QmdMemoryReadinessState::Ready {
    return Ok(status);
  }

  if matches!(
    status.qmd_readiness.state,
    QmdMemoryReadinessState::QmdUnavailable | QmdMemoryReadinessState::RuntimeUnsupported
  ) {
    return Err(anyhow::anyhow!("QMD is not ready: {}", status.qmd_readiness.detail));
  }

  let mut bun_status = load_bun_runtime_status();
  let runtime_command = match selected_runtime_command(&bun_status) {
    Some(command) => command,
    None => {
      bun_runtime_install(false).await?;
      bun_status = load_bun_runtime_status();
      selected_runtime_command(&bun_status)
        .ok_or_else(|| anyhow::anyhow!("Managed Bun was not available after installation"))?
    }
  };

  ensure_qmd_installed(&runtime_command)?;

  let status = load_js_runtime_health_status();
  if status.qmd_readiness.state == QmdMemoryReadinessState::Ready {
    return Ok(status);
  }

  Err(anyhow::anyhow!("QMD is not ready after bootstrap: {}", status.qmd_readiness.detail))
}

#[cfg(test)]
mod tests {
  use std::path::Path;
  use std::path::PathBuf;

  use super::BunRuntimeCommandState;
  use super::BunRuntimeCommandStatus;
  use super::BunRuntimeStatus;
  use super::JsRuntimeKind;
  use super::bundled_bun_resource_name_for_os;
  use super::install_supported_for_os;
  use super::managed_bun_install_target_for_os;
  use super::managed_bun_probe_paths_for_os;
  use super::path_help_snip_for_os;
  use super::preferred_path_runtime;
  use super::selected_runtime_command;

  fn status(command: &str, state: BunRuntimeCommandState) -> BunRuntimeCommandStatus {
    BunRuntimeCommandStatus {
      command: command.to_string(),
      detected_version: Some("1.0.0".to_string()),
      state,
      error: None,
    }
  }

  #[test]
  fn unix_targets_keep_user_local_bun_path() {
    let home = Path::new("/Users/tester");

    assert_eq!(managed_bun_install_target_for_os("macos", home, None), PathBuf::from("/Users/tester/.local/bin/bun"));
    assert_eq!(managed_bun_install_target_for_os("linux", home, None), PathBuf::from("/Users/tester/.local/bin/bun"));
    assert_eq!(
      managed_bun_probe_paths_for_os("linux", home, None),
      vec![PathBuf::from("/Users/tester/.local/bin/bun")]
    );
  }

  #[test]
  fn windows_targets_use_bundle_id_runtime_dir_and_probe_legacy_location() {
    let home = Path::new("C:/Users/tester");
    let local_app_data = Path::new("C:/Users/tester/AppData/Local");

    assert_eq!(
      managed_bun_install_target_for_os("windows", home, Some(local_app_data)),
      PathBuf::from("C:/Users/tester/AppData/Local/ai.blprnt/runtime/bun/bun.exe")
    );
    assert_eq!(
      managed_bun_probe_paths_for_os("windows", home, Some(local_app_data)),
      vec![
        PathBuf::from("C:/Users/tester/AppData/Local/ai.blprnt/runtime/bun/bun.exe"),
        PathBuf::from("C:/Users/tester/AppData/Local/blprnt/bun/bun.exe")
      ]
    );
  }

  #[test]
  fn bundled_bun_resource_name_matches_platform_conventions() {
    assert_eq!(bundled_bun_resource_name_for_os("macos"), "bun");
    assert_eq!(bundled_bun_resource_name_for_os("linux"), "bun");
    assert_eq!(bundled_bun_resource_name_for_os("windows"), "bun.exe");
  }

  #[test]
  fn managed_install_supports_windows() {
    assert!(install_supported_for_os("macos"));
    assert!(install_supported_for_os("linux"));
    assert!(install_supported_for_os("windows"));
  }

  #[test]
  fn windows_does_not_expose_unix_path_help() {
    assert_eq!(path_help_snip_for_os("macos"), Some(r#"export PATH="$HOME/.local/bin:$PATH""#.to_string()));
    assert_eq!(path_help_snip_for_os("linux"), Some(r#"export PATH="$HOME/.local/bin:$PATH""#.to_string()));
    assert_eq!(path_help_snip_for_os("windows"), None);
  }

  #[test]
  fn preferred_path_runtime_uses_bun_before_node() {
    let (runtime, kind) = preferred_path_runtime(
      status("bun", BunRuntimeCommandState::Available),
      status("node", BunRuntimeCommandState::Available),
    );

    assert_eq!(runtime.command, "bun");
    assert_eq!(kind, Some(JsRuntimeKind::Bun));
  }

  #[test]
  fn preferred_path_runtime_falls_back_to_node() {
    let (runtime, kind) = preferred_path_runtime(
      status("bun", BunRuntimeCommandState::Missing),
      status("node", BunRuntimeCommandState::Available),
    );

    assert_eq!(runtime.command, "node");
    assert_eq!(kind, Some(JsRuntimeKind::Node));
  }

  #[test]
  fn selected_runtime_command_uses_path_bun_then_managed_bun() {
    let path_bun = status("bun", BunRuntimeCommandState::Available);
    let managed_bun = status("/Users/tester/.local/bin/bun", BunRuntimeCommandState::Available);

    let with_path_bun = BunRuntimeStatus {
      bun:                 path_bun.clone(),
      user_local_bun:      managed_bun.clone(),
      install_target_path: managed_bun.command.clone(),
    };
    assert_eq!(selected_runtime_command(&with_path_bun), Some("bun".to_string()));

    let without_path_bun = BunRuntimeStatus {
      bun:                 status("bun", BunRuntimeCommandState::Missing),
      user_local_bun:      managed_bun.clone(),
      install_target_path: managed_bun.command,
    };
    assert_eq!(selected_runtime_command(&without_path_bun), Some("/Users/tester/.local/bin/bun".to_string()));
  }
}
