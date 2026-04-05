use std::path::PathBuf;
use std::process::ExitStatus;
use std::time::Duration;

use anyhow::Result;
use sandbox::RunSandbox;
use shared::errors::ToolError;
use shared::tools::config::ToolRuntimeConfig;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::process::Child as TokioChild;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "windows")]
use crate::host::baldr::Baldr;
#[cfg(target_os = "linux")]
use crate::host::loki::Loki;
#[cfg(target_os = "macos")]
use crate::host::thor::Thor;

const SIGKILL_CODE: i32 = 9;
const TIMEOUT_CODE: i32 = 64;
const EXIT_CODE_SIGNAL_BASE: i32 = 128;
const READ_CHUNK_SIZE: usize = 8192; // bytes per read
const AGGREGATE_BUFFER_INITIAL_CAPACITY: usize = 8 * 1024; // 8 KiB

pub struct Child;

impl Child {
  /// Spawn a command and wait for completion.
  ///
  /// This uses pipes (not PTY) intentionally - most CLI tools detect `!isatty()`
  /// and use non-interactive defaults. Combined with CI=true and other env vars
  /// set in `get_env()`, this causes tools like npm, cargo, etc. to skip prompts.
  pub async fn spawn(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    timeout: Option<u64>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
    cancel_token: Option<CancellationToken>,
  ) -> Result<(Vec<u8>, Vec<u8>, ExitStatus)> {
    #[cfg(not(target_os = "windows"))]
    let mut child = Self::get_child(workspace_root, command.clone(), args.clone(), runtime_config.clone(), sandbox)?;
    #[cfg(target_os = "windows")]
    let mut child = Self::get_child(workspace_root, command, args, runtime_config.clone(), sandbox)?;

    let stdout_reader = child.stdout.take().unwrap();
    let stderr_reader = child.stderr.take().unwrap();

    let stdout_handle = tokio::spawn(read_capped(BufReader::new(stdout_reader)));
    let stderr_handle = tokio::spawn(read_capped(BufReader::new(stderr_reader)));

    let (exit_code, timed_out, cancelled) = if let Some(timeout) = timeout {
      tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(timeout), child.wait()) => {
          match result {
            Ok(status_result) => {
              let exit_status = status_result.map_err(|e| ToolError::SpawnFailed(format!("failed to get exit status: {}", e)))?;
              (exit_status, false, false)
            }
            Err(_) => {
              kill_child(&mut child).await?;
              (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + TIMEOUT_CODE), true, false)
            }
          }
        }
        _ = tokio::signal::ctrl_c() => {
          kill_child(&mut child).await?;
          (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false, false)
        }
        _ = wait_for_cancel(cancel_token.clone()), if cancel_token.is_some() => {
          kill_child(&mut child).await?;
          (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false, true)
        }
      }
    } else {
      tokio::select! {
        result = child.wait() => {
          match result {
            Ok(exit_status) => (exit_status, false, false),
            Err(_) => (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false, false),
          }
        }
        _ = wait_for_cancel(cancel_token.clone()), if cancel_token.is_some() => {
          kill_child(&mut child).await?;
          (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false, true)
        }
      }
    };

    if timed_out {
      return Err(ToolError::CommandTimeout.into());
    }
    if cancelled {
      return Err(ToolError::SpawnFailed("command cancelled".into()).into());
    }

    let stdout = stdout_handle.await.map_err(|_e| ToolError::ProcessOutputFailed("failed to read stdout".into()))??;
    let stderr = stderr_handle.await.map_err(|_e| ToolError::ProcessOutputFailed("failed to read stderr".into()))??;

    #[cfg(target_os = "macos")]
    if let Some(error) = macos_sandbox_required_error(&stderr, exit_code.success()) {
      return Err(error);
    }

    Ok((stdout, stderr, exit_code))
  }

  #[cfg(target_os = "macos")]
  fn get_child(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
  ) -> Result<TokioChild> {
    Thor::exec(workspace_root, command, args, runtime_config, sandbox)
  }

  #[cfg(target_os = "linux")]
  fn get_child(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
  ) -> Result<TokioChild> {
    Loki::exec(workspace_root, command, args, runtime_config, sandbox)
  }

  #[cfg(target_os = "windows")]
  fn get_child(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    _sandbox: std::sync::Arc<RunSandbox>,
  ) -> Result<TokioChild> {
    Baldr::exec(workspace_root, command, args, runtime_config)
  }
}

async fn wait_for_cancel(cancel_token: Option<CancellationToken>) {
  if let Some(cancel_token) = cancel_token {
    cancel_token.cancelled().await;
  }
}

async fn kill_child(child: &mut TokioChild) -> Result<()> {
  child.start_kill().map_err(|e| ToolError::SpawnFailed(format!("failed to kill process: {}", e)))?;
  let _ = child.wait().await;
  Ok(())
}

#[cfg(target_os = "macos")]
fn macos_sandbox_required_error(stderr: &[u8], success: bool) -> Option<anyhow::Error> {
  if success {
    return None;
  }

  let stderr_text = String::from_utf8_lossy(stderr);
  if is_macos_sandbox_failure_text(&stderr_text) {
    Some(
      ToolError::SpawnFailed(format!(
        "macOS shell execution requires sandboxing; sandbox-exec failed instead of running unsandboxed: {}",
        stderr_text.trim()
      ))
      .into(),
    )
  } else {
    None
  }
}

#[cfg(target_os = "macos")]
fn is_macos_sandbox_failure_text(stderr_text: &str) -> bool {
  stderr_text.contains("sandbox_apply: Operation not permitted") || stderr_text.contains("sandbox-exec")
}

#[cfg(unix)]
fn synthetic_exit_status(code: i32) -> ExitStatus {
  use std::os::unix::process::ExitStatusExt;
  std::process::ExitStatus::from_raw(code)
}

#[cfg(windows)]
fn synthetic_exit_status(code: i32) -> ExitStatus {
  use std::os::windows::process::ExitStatusExt;
  #[expect(clippy::unwrap_used)]
  std::process::ExitStatus::from_raw(code.try_into().unwrap())
}

async fn read_capped<R: AsyncRead + Unpin + Send + 'static>(mut reader: R) -> Result<Vec<u8>> {
  let mut buf = Vec::with_capacity(AGGREGATE_BUFFER_INITIAL_CAPACITY);
  let mut tmp = [0u8; READ_CHUNK_SIZE];

  // No caps: append all bytes

  loop {
    let n =
      reader.read(&mut tmp).await.map_err(|e| ToolError::ProcessOutputFailed(format!("failed to read: {}", e)))?;
    if n == 0 {
      break;
    }

    append_all(&mut buf, &tmp[..n]);
  }

  Ok(buf)
}

#[inline]
fn append_all(dst: &mut Vec<u8>, src: &[u8]) {
  dst.extend_from_slice(src);
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
  use super::is_macos_sandbox_failure_text;
  use super::macos_sandbox_required_error;

  #[test]
  fn detects_seatbelt_failure_text() {
    assert!(is_macos_sandbox_failure_text("sandbox-exec: sandbox_apply: Operation not permitted"));
    assert!(is_macos_sandbox_failure_text("sandbox-exec: seatbelt failure"));
    assert!(!is_macos_sandbox_failure_text("plain command failure"));
  }

  #[test]
  fn converts_sandbox_failure_into_explicit_error() {
    let error = macos_sandbox_required_error(b"sandbox-exec: sandbox_apply: Operation not permitted\n", false)
      .expect("sandbox failure should return an explicit error");
    let message = error.to_string();

    assert!(message.contains("requires sandboxing"));
    assert!(message.contains("instead of running unsandboxed"));
    assert!(message.contains("sandbox_apply: Operation not permitted"));
  }

  #[test]
  fn ignores_successful_exit_even_if_stderr_mentions_sandbox() {
    assert!(macos_sandbox_required_error(b"sandbox-exec warning", true).is_none());
  }
}
