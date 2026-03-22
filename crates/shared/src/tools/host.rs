use surrealdb_types::SurrealValue;

use crate::tools::ToolSpec;
use crate::tools::ToolUseResponseData;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[cfg_attr(
  not(target_os = "windows"),
  schemars(
    title = "shell",
    description = "Executes shell commands via a `/bin/bash -c` wrapper. Use this for one-off commands where final stdout/stderr is sufficient. Do not wrap commands as `bash -c` for normal use; host execution already applies shell wrapping when needed. If you need incremental output, session state across calls, interactivity, or a long-running process, use `terminal` instead. This is a one-off tool and should not be used for long-running processes."
  )
)]
#[cfg_attr(
  target_os = "windows",
  schemars(
    title = "shell",
    description = "Executes shell commands on Windows. Use this for one-off commands where final stdout/stderr is sufficient.\n\nWindows contract:\n- Set `command` to the target executable or cmdlet (for example `python`, `git`, `Get-ChildItem`).\n- Pass each argument as a separate token in `args`.\n- Do not wrap commands as `powershell -c` or `pwsh -c` for normal use; host execution already applies PowerShell wrapping when needed.\n- If you need incremental output, session state across calls, interactivity, or a long-running process, use `terminal` instead.\n\nExamples:\n- Native executable: `{ \"command\": \"python\", \"args\": [\"-c\", \"print('ok')\"], \"timeout\": 30 }`\n- PowerShell cmdlet: `{ \"command\": \"Get-ChildItem\", \"args\": [\"-Path\", \".\", \"-Name\"], \"timeout\": 30 }`\n- Prefer direct command over wrapper: use `{\"command\":\"python\",...}` instead of `{\"command\":\"powershell\",\"args\":[\"-c\",\"python ...\"]}`. This is a one-off tool and should not be used for long-running processes."
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[schemars(
  title = "terminal",
  description = r#"Creates and interacts with a persistent, stateful terminal session.

This tool differs from `shell`:
- `shell` executes a command and returns stdout/stderr once.
- `terminal` creates a long-lived interactive session.
- The terminal maintains screen state across calls.
- The terminal can be written to incrementally and polled for screen snapshots.

Selection guidance:
- Use `terminal` instead of `shell` when you need incremental output, polling, interactivity, or state that persists across calls.
- Use `terminal` for long-running commands such as dev servers, watchers, REPLs, or commands that may prompt for follow-up input.
- Use `shell` only for short one-off commands where final stdout/stderr is enough.

Lifecycle contract:

1. `open`
   - Creates a new interactive shell session.
   - Returns a unique `terminal_id`.
   - No `terminal_id` should be provided for this action.

2. `write`
   - Sends raw text input into the terminal as if typed by a user.
   - `terminal_id` is required.
   - The command will execute asynchronously.

3. `snapshot`
   - Returns the current visible terminal screen buffer.
   - `terminal_id` is required.
   - If `timeout` is provided, the system may wait up to N seconds for new output before returning.
   - Snapshot includes only the visible grid, not full scrollback history.

4. `close`
   - Gracefully terminates the session.
   - Returns exit code if available.
   - `terminal_id` is required.

Behavior guarantees:

- Each terminal session is isolated.
- Output is incremental; snapshot reflects parsed terminal state.
- The snapshot represents the authoritative backend screen state.
- ANSI escape sequences are interpreted by the backend emulator.
- The terminal may continue running between calls.

Polling semantics:

- The recommended pattern for long-running commands:
  1. `open` with optional input
  2. `snapshot` with timeout
  3. Repeat until desired condition or completion
  4. `close` if the terminal is no longer needed

Platform behavior:

- On Unix systems, the terminal runs under the user's default shell.
- On Windows, the terminal runs under the system PowerShell host.
- Commands should not be wrapped manually in `bash -c` or `powershell -c`.
- When using `open` with input, the input will be sent to the terminal and the terminal will execute the command, returning on the first snapshot change.
- When using `open` without input, the terminal will open and return immediately with no snapshot.

Anti-patterns:

- Do not attempt to execute multiple unrelated commands without newline separation.
- Do not reuse `terminal_id` after `close`.
- Do not assume snapshot includes full scrollback history.

This tool is designed for interactive workflows,
long-running processes, and incremental output inspection.
"#
)]
pub struct TerminalArgs {
  #[schemars(description = "Operation to perform on the terminal session.")]
  pub action: TerminalAction,

  #[schemars(description = "Optional terminal session ID. Required for all actions except `open`.")]
  pub terminal_id: Option<String>,

  #[schemars(description = "Input text to write to the terminal (used with `write` and `open`).")]
  #[schemars(default)]
  pub input: Option<String>,

  #[schemars(default)]
  #[schemars(description = "Optional timeout in seconds (used with `snapshot` to wait for changes).")]
  pub timeout: Option<u64>,

  #[schemars(default)]
  #[schemars(
    description = "Optional zero-based workspace index to use. If not provided, the first workspace will be used."
  )]
  pub workspace_index: Option<u8>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum TerminalAction {
  Open,
  Write,
  Snapshot,
  Close,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TerminalPayload {
  pub terminal_id: String,

  /// Present for snapshot responses.
  pub snapshot: Option<TerminalSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct TerminalSnapshot {
  pub rows:  usize,
  pub cols:  usize,
  pub lines: Vec<String>,
}

impl From<TerminalPayload> for ToolUseResponseData {
  fn from(payload: TerminalPayload) -> Self {
    Self::Terminal(payload)
  }
}

impl TerminalArgs {
  pub fn schema() -> Vec<ToolSpec> {
    let schema = schemars::schema_for!(TerminalArgs);
    let json = serde_json::to_value(&schema).expect("[TerminalArgs] schema is required");

    let params = serde_json::json!({
      "type": "object",
      "properties": json.get("properties").expect("[TerminalArgs] properties is required"),
      "required": json.get("required").expect("[TerminalArgs] required is required"),
    });

    let name = schema.get("title").expect("[TerminalArgs] title is required").clone();
    let description = schema.get("description").expect("[TerminalArgs] description is required").clone();

    vec![ToolSpec { name, description, params }]
  }
}
