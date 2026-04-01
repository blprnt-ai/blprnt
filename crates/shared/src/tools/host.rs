use surrealdb_types::SurrealValue;

use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[cfg_attr(
  not(target_os = "windows"),
  schemars(
    title = "shell",
    description = "Executes shell commands via a `/bin/bash -c` wrapper. Use this for one-off commands where final stdout/stderr is sufficient. Do not wrap commands as `bash -c` for normal use; host execution already applies shell wrapping when needed. This is a one-off tool and should not be used for long-running processes."
  )
)]
#[cfg_attr(
  target_os = "windows",
  schemars(
    title = "shell",
    description = "Executes shell commands on Windows. Use this for one-off commands where final stdout/stderr is sufficient.\n\nWindows contract:\n- Set `command` to the target executable or cmdlet (for example `python`, `git`, `Get-ChildItem`).\n- Pass each argument as a separate token in `args`.\n- Do not wrap commands as `powershell -c` or `pwsh -c` for normal use; host execution already applies PowerShell wrapping when needed.\n\nExamples:\n- Native executable: `{ \"command\": \"python\", \"args\": [\"-c\", \"print('ok')\"], \"timeout\": 30 }`\n- PowerShell cmdlet: `{ \"command\": \"Get-ChildItem\", \"args\": [\"-Path\", \".\", \"-Name\"], \"timeout\": 30 }`\n- Prefer direct command over wrapper: use `{\"command\":\"python\",...}` instead of `{\"command\":\"powershell\",\"args\":[\"-c\",\"python ...\"]}`. This is a one-off tool and should not be used for long-running processes."
  )
)]
pub struct ShellArgs {
  #[cfg_attr(
    target_os = "windows",
    schemars(description = "Executable or command name to run (do not include `powershell`/`pwsh` wrapper prefixes).")
  )]
  pub command: String,
  #[schemars(default)]
  #[cfg_attr(
    target_os = "windows",
    schemars(description = "Arguments passed to `command` as separate argv tokens, in order.")
  )]
  pub args:    Vec<String>,
  #[schemars(default)]
  #[cfg_attr(target_os = "windows", schemars(description = "Optional timeout in seconds."))]
  pub timeout: Option<u64>,

  #[schemars(default)]
  #[schemars(
    description = "Optional zero-based workspace index to use. If not provided, the first workspace will be used."
  )]
  pub workspace_index: Option<u8>,
}

#[derive(
  Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue, ts_rs::TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct ShellPayload {
  pub stdout:    String,
  pub stderr:    String,
  pub exit_code: i32,
}

impl From<ShellPayload> for ToolUseResponseData {
  fn from(payload: ShellPayload) -> Self {
    Self::Shell(payload)
  }
}
