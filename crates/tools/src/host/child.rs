use std::path::PathBuf;
use std::process::ExitStatus;
use std::time::Duration;

use anyhow::Result;
use sandbox::RunSandbox;
use shared::errors::ToolError;
#[cfg(not(target_os = "windows"))]
use shared::sandbox_flags::SandboxFlags;
use shared::tools::config::ToolRuntimeConfig;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::process::Child as TokioChild;

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
    #[cfg(not(target_os = "windows"))] sandbox_flags: SandboxFlags,
  ) -> Result<(Vec<u8>, Vec<u8>, ExitStatus)> {
    #[cfg(not(target_os = "windows"))]
    let mut child = Self::get_child(workspace_root, command, args, runtime_config.clone(), sandbox, sandbox_flags)?;
    #[cfg(target_os = "windows")]
    let mut child = Self::get_child(workspace_root, command, args, runtime_config.clone(), sandbox)?;

    let stdout_reader = child.stdout.take().unwrap();
    let stderr_reader = child.stderr.take().unwrap();

    let stdout_handle = tokio::spawn(read_capped(BufReader::new(stdout_reader)));
    let stderr_handle = tokio::spawn(read_capped(BufReader::new(stderr_reader)));

    let (exit_code, timed_out) = if let Some(timeout) = timeout {
      tokio::select! {
          result = tokio::time::timeout(Duration::from_secs(timeout), child.wait()) => {
              match result {
                  Ok(status_result) => {
                      let exit_status = status_result.map_err(|e| ToolError::SpawnFailed(format!("failed to get exit status: {}", e)))?;
                      (exit_status, false)
                  }
                  Err(_) => {
                      // timeout
                      child.start_kill().map_err(|e| ToolError::SpawnFailed(format!("failed to kill process: {}", e)))?;
                      // Debatable whether `child.wait().await` should be called here.
                      (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + TIMEOUT_CODE), true)
                  }
              }
          }
          _ = tokio::signal::ctrl_c() => {
              child.start_kill().map_err(|e| ToolError::SpawnFailed(format!("failed to kill process: {}", e)))?;
              (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false)
          }
      }
    } else {
      match child.wait().await {
        Ok(exit_status) => (exit_status, false),
        Err(_) => (synthetic_exit_status(EXIT_CODE_SIGNAL_BASE + SIGKILL_CODE), false),
      }
    };

    if timed_out {
      return Err(ToolError::CommandTimeout.into());
    }

    let stdout = stdout_handle.await.map_err(|_e| ToolError::ProcessOutputFailed("failed to read stdout".into()))??;
    let stderr = stderr_handle.await.map_err(|_e| ToolError::ProcessOutputFailed("failed to read stderr".into()))??;

    Ok((stdout, stderr, exit_code))
  }

  #[cfg(target_os = "macos")]
  fn get_child(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
    sandbox_flags: SandboxFlags,
  ) -> Result<TokioChild> {
    Thor::exec(workspace_root, command, args, runtime_config, sandbox, sandbox_flags)
  }

  #[cfg(target_os = "linux")]
  fn get_child(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
    sandbox_flags: SandboxFlags,
  ) -> Result<TokioChild> {
    Loki::exec(workspace_root, command, args, runtime_config, sandbox, sandbox_flags)
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
