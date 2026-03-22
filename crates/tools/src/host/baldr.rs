use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Result;
use base64::Engine;
use shared::errors::ToolError;
use tokio::process::Child;
use tokio::process::Command;

pub struct Baldr;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

impl Baldr {
  /// Windows execution strategy adapted from codex-rs:
  /// - Prefer argv-style native process execution when we can infer a program + args.
  /// - Fall back to PowerShell script execution for cmdlets/expressions.
  pub fn exec(cwd: &PathBuf, command: String, args: Vec<String>) -> Result<Child> {
    let normalized = Self::normalize_command(cwd, command, args);
    let effective_cwd = &normalized.cwd;
    let effective_command = &normalized.command;
    let effective_args = &normalized.args;
    let env_overrides = &normalized.env_overrides;

    if let Some((program, program_args)) = Self::direct_argv(&effective_command, &effective_args) {
      match Self::spawn_process(effective_cwd, &program, &program_args, env_overrides) {
        Ok(child) => return Ok(child),
        Err(err) if err.kind() == ErrorKind::NotFound => {
          // Not an external executable (likely a PowerShell cmdlet/expression) -> fallback below.
        }
        Err(err) => return Err(ToolError::SpawnFailed(err.to_string()).into()),
      }
    }

    let script = Self::build_script(&effective_command, &effective_args);
    let (pwsh, pwsh_args) = Self::build_powershell_command(script);
    Self::spawn_process(effective_cwd, &pwsh, &pwsh_args, env_overrides)
      .map_err(|e| ToolError::SpawnFailed(e.to_string()).into())
  }

  fn normalize_command(cwd: &PathBuf, command: String, args: Vec<String>) -> NormalizedCommand {
    if !args.is_empty() {
      if let Some(script) = Self::extract_powershell_script_arg(&command, &args) {
        // Unwrap powershell wrappers like `powershell -c "<script>"` so the
        // inner command can use argv-first execution and avoid quote garbling.
        return Self::normalize_command(cwd, script, vec![]);
      }

      return NormalizedCommand { cwd: cwd.clone(), command, args, env_overrides: HashMap::new() };
    }

    let statements = Self::split_unquoted_statements(&command);
    if statements.len() <= 1 {
      return NormalizedCommand { cwd: cwd.clone(), command, args, env_overrides: HashMap::new() };
    }

    let mut effective_cwd = cwd.clone();
    let mut env_overrides = HashMap::new();
    let mut idx = 0usize;

    while idx < statements.len() {
      let stmt = statements[idx].trim();
      if stmt.is_empty() {
        idx += 1;
        continue;
      }

      if let Some((key, value)) = Self::parse_env_assignment(stmt) {
        env_overrides.insert(key, value);
        idx += 1;
        continue;
      }

      if let Some(cd_target) = Self::parse_cd_statement(stmt) {
        let mut next_cwd = PathBuf::from(cd_target);
        if !next_cwd.is_absolute() {
          next_cwd = effective_cwd.join(next_cwd);
        }
        effective_cwd = next_cwd;
        idx += 1;
        continue;
      }

      break;
    }

    if idx > 0 && idx < statements.len() {
      let tail = statements[idx..].join(" ; ");
      return NormalizedCommand { cwd: effective_cwd, command: tail, args, env_overrides };
    }

    NormalizedCommand { cwd: cwd.clone(), command, args, env_overrides: HashMap::new() }
  }

  fn spawn_process(
    cwd: &PathBuf,
    program: &str,
    args: &[String],
    env_overrides: &HashMap<String, String>,
  ) -> std::io::Result<Child> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.current_dir(cwd);
    cmd.envs(crate::host::env::get_env());
    if !env_overrides.is_empty() {
      cmd.envs(env_overrides);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.spawn()
  }

  fn direct_argv(command: &str, args: &[String]) -> Option<(String, Vec<String>)> {
    let command = command.trim();
    if command.is_empty() {
      return None;
    }

    if !args.is_empty() {
      return Some((command.to_string(), args.to_vec()));
    }

    // Models often send command-only payloads with escaped double quotes (`\"`).
    // Normalize before tokenizing so native argv execution can preserve intent.
    let normalized = command.replace("\\\"", "\"");
    if Self::has_unquoted_shell_operators(&normalized) {
      return None;
    }
    let parsed = Self::split_windows_command_line(&normalized);
    let (program, tail) = parsed.split_first()?;

    Some((program.clone(), tail.to_vec()))
  }

  fn extract_powershell_script_arg(command: &str, args: &[String]) -> Option<String> {
    if !Self::is_powershell_executable(command) || args.is_empty() {
      return None;
    }

    let mut idx = 0usize;
    while idx < args.len() {
      let flag = &args[idx];
      let lower = flag.to_ascii_lowercase();
      match lower.as_str() {
        // benign flags with no value
        "-nologo" | "-noprofile" | "-noninteractive" | "-mta" | "-sta" => {
          idx += 1;
        }
        // benign flags with one value
        "-executionpolicy" | "/executionpolicy" => {
          if idx + 1 >= args.len() {
            return None;
          }
          idx += 2;
        }
        // script passed as a separate next token
        "-command" | "/command" | "-c" => {
          let script = args.get(idx + 1)?.clone();
          if idx + 2 != args.len() {
            return None;
          }
          return Some(script);
        }
        // script passed as part of the same token
        _ if lower.starts_with("-command:") || lower.starts_with("/command:") => {
          if idx + 1 != args.len() {
            return None;
          }
          let (_, script) = flag.split_once(':')?;
          return Some(script.to_string());
        }
        _ => return None,
      }
    }

    None
  }

  fn is_powershell_executable(command: &str) -> bool {
    let executable_name =
      Path::new(command).file_name().and_then(|s| s.to_str()).unwrap_or(command).to_ascii_lowercase();

    matches!(executable_name.as_str(), "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe")
  }

  fn split_unquoted_statements(command: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut rest = command.trim();

    while !rest.is_empty() {
      if let Some(idx) = Self::find_first_unquoted(rest, ';') {
        let (left, right_with_sep) = rest.split_at(idx);
        let left = left.trim();
        if !left.is_empty() {
          statements.push(left.to_string());
        }
        rest = right_with_sep[1..].trim_start();
      } else {
        let tail = rest.trim();
        if !tail.is_empty() {
          statements.push(tail.to_string());
        }
        break;
      }
    }

    statements
  }

  fn parse_cd_statement(statement: &str) -> Option<String> {
    let left_tokens = Self::split_windows_command_line(statement.trim());
    if left_tokens.is_empty() || !left_tokens[0].eq_ignore_ascii_case("cd") {
      return None;
    }

    // Support `cd <path>` and `cd /d <path>` forms.
    let path_idx = if left_tokens.get(1).is_some_and(|t| t.eq_ignore_ascii_case("/d")) { 2 } else { 1 };
    let cd_target = left_tokens.get(path_idx)?;
    if left_tokens.len() != path_idx + 1 {
      return None;
    }

    Some(cd_target.clone())
  }

  fn parse_env_assignment(statement: &str) -> Option<(String, String)> {
    let trimmed = statement.trim();
    if trimmed.len() < 6 {
      return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("$env:") {
      return None;
    }

    let body = &trimmed[5..];
    let eq_idx = Self::find_first_unquoted(body, '=')?;
    let key = body[..eq_idx].trim();
    let value = body[eq_idx + 1..].trim();
    if key.is_empty() {
      return None;
    }

    let value = Self::unquote_literal(value);
    Some((key.to_string(), value))
  }

  fn unquote_literal(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
      value[1..value.len() - 1].replace("''", "'")
    } else if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
      value[1..value.len() - 1].replace("`\"", "\"").replace("``", "`")
    } else {
      value.to_string()
    }
  }

  fn build_script(command: &str, args: &[String]) -> String {
    let mut script = String::from("$ProgressPreference='SilentlyContinue'; ");
    if args.is_empty() {
      // Preserve raw behavior for PowerShell cmdlets/expressions.
      script.push_str(command.trim());
      return script;
    }

    let cmd = Self::escape_ps_single(command);
    let arg_list: Vec<String> = args.iter().map(|a| Self::render_ps_arg(a)).collect();
    script.push_str(&format!("& {cmd} {}", arg_list.join(" ")));

    script
  }

  fn has_unquoted_shell_operators(command: &str) -> bool {
    Self::find_first_unquoted(command, '|').is_some()
      || Self::find_first_unquoted(command, '&').is_some()
      || Self::find_first_unquoted(command, ';').is_some()
      || Self::find_first_unquoted(command, '<').is_some()
      || Self::find_first_unquoted(command, '>').is_some()
  }

  fn find_first_unquoted(command: &str, needle: char) -> Option<usize> {
    let chars: Vec<char> = command.chars().collect();
    let mut in_single = false;
    let mut in_double = false;

    for (i, ch) in chars.iter().enumerate() {
      let ch = *ch;
      if ch == '\'' && !in_double {
        in_single = !in_single;
        continue;
      }

      if ch == '"' && !in_single {
        let mut backslashes = 0;
        let mut j = i;
        while j > 0 && chars[j - 1] == '\\' {
          backslashes += 1;
          j -= 1;
        }
        if backslashes % 2 == 0 {
          in_double = !in_double;
          continue;
        }
      }

      if !in_single && !in_double && ch == needle {
        return Some(i);
      }
    }

    None
  }

  /// Split a command line using Windows-style quote/backslash rules.
  /// This is intentionally focused on shell-tool command ingestion (not full cmd.exe parsing).
  fn split_windows_command_line(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
      while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
      }
      if i >= chars.len() {
        break;
      }

      let mut arg = String::new();
      let mut in_quotes = false;
      let mut saw_any = false;

      loop {
        let mut backslashes = 0usize;
        while i < chars.len() && chars[i] == '\\' {
          backslashes += 1;
          i += 1;
        }

        if i < chars.len() && chars[i] == '"' {
          arg.push_str(&"\\".repeat(backslashes / 2));

          if backslashes % 2 == 0 {
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
              arg.push('"');
              i += 2;
            } else {
              in_quotes = !in_quotes;
              i += 1;
            }
          } else {
            arg.push('"');
            i += 1;
          }

          saw_any = true;
          continue;
        }

        if backslashes > 0 {
          arg.push_str(&"\\".repeat(backslashes));
          saw_any = true;
        }

        if i >= chars.len() || (!in_quotes && chars[i].is_whitespace()) {
          break;
        }

        arg.push(chars[i]);
        saw_any = true;
        i += 1;
      }

      if saw_any {
        out.push(arg);
      }

      while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
      }
    }

    out
  }

  fn render_ps_arg(arg: &str) -> String {
    if Self::is_parameter_token(arg) { arg.to_string() } else { Self::escape_ps_single(arg) }
  }

  fn is_parameter_token(arg: &str) -> bool {
    // Keep cmdlet/exe flag-like tokens unquoted: -Path, -Recurse, --yes, -Name:Value.
    // Do not treat numeric values (e.g. -1) as parameter tokens.
    if arg.len() < 2 || !arg.starts_with('-') || arg.chars().any(char::is_whitespace) {
      return false;
    }

    let trimmed = arg.trim_start_matches('-');
    let first = trimmed.chars().next();
    matches!(first, Some(c) if c.is_ascii_alphabetic())
  }

  fn escape_ps_single(s: &str) -> String {
    let mut r = String::from("'");
    for c in s.chars() {
      if c == '\'' {
        r.push_str("''");
      } else {
        r.push(c);
      }
    }
    r.push('\'');
    r
  }

  fn build_powershell_command(script: String) -> (String, Vec<String>) {
    let encoded = Self::base64_utf16le(&script);
    (
      "powershell.exe".into(),
      vec![
        "-NoLogo".into(),
        "-NonInteractive".into(),
        "-NoProfile".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-EncodedCommand".into(),
        encoded,
      ],
    )
  }

  fn base64_utf16le(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len() * 2);
    for u in s.encode_utf16() {
      bytes.extend_from_slice(&u.to_le_bytes());
    }

    base64::engine::general_purpose::STANDARD.encode(bytes)
  }
}

struct NormalizedCommand {
  cwd:           PathBuf,
  command:       String,
  args:          Vec<String>,
  env_overrides: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use shared::tools::ToolUseResponse;
  use shared::tools::ToolUseResponseData;
  use shared::tools::host::ShellArgs;

  use super::Baldr;

  #[test]
  fn build_script_keeps_raw_command_when_args_empty() {
    let script =
      Baldr::build_script("Get-ChildItem -Path backend -Recurse -File -Include *.db *.sqlite *.sqlite3", &[]);
    assert_eq!(
      script,
      "$ProgressPreference='SilentlyContinue'; Get-ChildItem -Path backend -Recurse -File -Include *.db *.sqlite *.sqlite3"
    );
  }

  #[test]
  fn build_script_invokes_command_with_args_when_present() {
    let args = vec![
      "-Path".to_string(),
      "backend".to_string(),
      "-Include".to_string(),
      "*.db".to_string(),
      "*.sqlite".to_string(),
      "*.sqlite3".to_string(),
    ];

    let script = Baldr::build_script("Get-ChildItem", &args);
    assert_eq!(
      script,
      "$ProgressPreference='SilentlyContinue'; & 'Get-ChildItem' -Path 'backend' -Include '*.db' '*.sqlite' '*.sqlite3'"
    );
  }

  #[test]
  fn direct_argv_parses_command_only_payload_with_escaped_quotes() {
    let (program, args) =
      Baldr::direct_argv("python -c \\\"print('ok')\\\"", &[]).expect("direct argv should be inferred");
    assert_eq!(program, "python");
    assert_eq!(args, vec!["-c".to_string(), "print('ok')".to_string()]);
  }

  #[test]
  fn extract_powershell_script_arg_supports_common_flags() {
    let args = vec!["-NoProfile".to_string(), "-c".to_string(), "python -c \"print('ok')\"".to_string()];
    let script =
      Baldr::extract_powershell_script_arg("powershell", &args).expect("should unwrap powershell script argument");
    assert_eq!(script, "python -c \"print('ok')\"");
  }

  #[tokio::test]
  async fn build_script_invokes_command_with_args_when_present_on_windows() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-shell").expect("create temp dir");
    let workdir = temp.path().join("root");
    let spaced = workdir.join("space dir");
    std::fs::create_dir_all(&spaced).expect("create test dir");
    std::fs::write(spaced.join("marker.txt"), b"ok").expect("create marker file");

    let tool = ShellTool {
      args: ShellArgs {
        command: "Get-ChildItem".to_string(),
        args:    vec!["-Path".to_string(), spaced.to_string_lossy().to_string(), "-Name".to_string()],
        timeout: Some(30),
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    println!("payload: {:?}", payload);

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(
      !payload.stderr.contains("Preparing modules for first use."),
      "unexpected progress noise in stderr: {}",
      payload.stderr
    );
    assert!(payload.stdout.contains("marker.txt"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn command_only_payload_with_escaped_quotes_executes_without_garbling() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-command-only").expect("create temp dir");
    let workdir = temp.path().join("root");
    std::fs::create_dir_all(&workdir).expect("create test dir");

    let tool = ShellTool {
      args: ShellArgs {
        command: "powershell -NoProfile -Command \\\"Write-Output ok\\\"".to_string(),
        args:    vec![],
        timeout: Some(30),
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("ok"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn command_only_payload_with_cd_prefix_and_escaped_quotes_executes_without_garbling() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-command-only-cd").expect("create temp dir");
    let workdir = temp.path().join("root");
    let spaced = workdir.join("space dir");
    std::fs::create_dir_all(&spaced).expect("create test dir");

    let command = format!("cd \"{}\" ; python -c \\\"print('ok')\\\"", spaced.to_string_lossy());

    let tool = ShellTool { args: ShellArgs { command, args: vec![], timeout: Some(30) } };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("ok"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn command_only_payload_with_env_and_cd_prefix_executes_without_garbling() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-command-only-env-cd").expect("create temp dir");
    let workdir = temp.path().join("root");
    let spaced = workdir.join("space dir");
    std::fs::create_dir_all(&spaced).expect("create test dir");

    let command = format!(
      "$env:MY_VAR=123 ; cd \"{}\" ; python -c \\\"import os; print(os.getenv('MY_VAR')); print('ok')\\\"",
      spaced.to_string_lossy()
    );

    let tool = ShellTool { args: ShellArgs { command, args: vec![], timeout: Some(30) } };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("123"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
    assert!(payload.stdout.contains("ok"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn permutation_cd_then_env_then_cmdlets_applies_prefixes() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-permute-cd-env").expect("create temp dir");
    let workdir = temp.path().join("root");
    let target = workdir.join("permute one");
    std::fs::create_dir_all(&target).expect("create test dir");
    std::fs::write(target.join("marker.txt"), b"ok").expect("create marker file");

    let command = format!(
      "cd \"{}\" ; $env:MY_VAR=123 ; Write-Output $env:MY_VAR ; Get-ChildItem -Path . -Name",
      target.to_string_lossy()
    );

    let tool = ShellTool { args: ShellArgs { command, args: vec![], timeout: Some(30) } };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("123"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
    assert!(payload.stdout.contains("marker.txt"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn permutation_env_then_cd_then_cmdlets_applies_prefixes() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-permute-env-cd").expect("create temp dir");
    let workdir = temp.path().join("root");
    let target = workdir.join("permute two");
    std::fs::create_dir_all(&target).expect("create test dir");
    std::fs::write(target.join("marker.txt"), b"ok").expect("create marker file");

    let command = format!(
      "$env:MY_VAR=123 ; cd \"{}\" ; Write-Output $env:MY_VAR ; Get-ChildItem -Path . -Name",
      target.to_string_lossy()
    );

    let tool = ShellTool { args: ShellArgs { command, args: vec![], timeout: Some(30) } };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("123"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
    assert!(payload.stdout.contains("marker.txt"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }

  #[tokio::test]
  async fn powershell_wrapper_python_c_executes_without_garbling() {
    use persistence::prelude::SurrealId;
    use shared::agent::AgentKind;
    use shared::sandbox_flags::SandboxFlags;

    use crate::Tool;
    use crate::host::shell::ShellTool;
    use crate::tool_use::ToolUseContext;

    let temp = tempdir::TempDir::new("baldr-powershell-python").expect("create temp dir");
    let workdir = temp.path().join("root");
    let target = workdir.join("python target");
    std::fs::create_dir_all(&target).expect("create test dir");
    std::fs::write(target.join("marker.txt"), b"ok").expect("create marker file");

    let command = "powershell".to_string();

    let tool = ShellTool {
      args: ShellArgs {
        command,
        args: vec![
          "-NoProfile".to_string(),
          "-c".to_string(),
          "python -c \"print('BEGIN'); print('state=\\\"COMPLETED\\\"'); print('END')\"".to_string(),
        ],
        timeout: Some(30),
      },
    };

    let sandbox = sandbox::get_sandbox();
    let mut sandbox = sandbox.write().await;
    sandbox.add_dir("test", &workdir).await.expect("sandbox add dir");

    let context = ToolUseContext::new(
      SurrealId::default(),
      None,
      SurrealId::default(),
      AgentKind::Crew,
      vec![PathBuf::from(&workdir)],
      SandboxFlags::default(),
      "test".to_string(),
      false,
    )
    .await;

    let result = tool.run(context).await.expect("shell tool run");
    let payload = match result {
      ToolUseResponse::Success(success) => match success.data {
        ToolUseResponseData::Shell(payload) => payload,
        other => panic!("unexpected payload type: {:?}", other),
      },
      failure => panic!("expected success response, got: {:?}", failure),
    };

    assert_eq!(payload.exit_code, 0, "stderr: {}", payload.stderr);
    assert!(payload.stdout.contains("BEGIN"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
    assert!(payload.stdout.contains("COMPLETED"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
    assert!(payload.stdout.contains("END"), "stdout: {}, stderr: {}", payload.stdout, payload.stderr);
  }
}
