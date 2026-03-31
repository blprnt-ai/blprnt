use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Result;
use sandbox::RunSandbox;
use shared::errors::ToolError;
use shared::tools::config::ToolRuntimeConfig;
use tokio::process::Child;
use tokio::process::Command;

pub struct Thor;

const MACOS_SEATBELT_BASE_POLICY: &str = include_str!("sandbox_base_policy.sbpl");
const MACOS_PATH_TO_SANDBOX_EXECUTABLE: &str = "/usr/bin/sandbox-exec";

impl Thor {
  pub fn exec(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    runtime_config: ToolRuntimeConfig,
    sandbox: std::sync::Arc<RunSandbox>,
  ) -> Result<Child> {
    let mut args = args.clone();
    if args.first().map(|a| a == "-c") == Some(true) {
      args.remove(0);
    }

    if command != "sh" && command != "/bin/sh" && command != "bash" && command != "/bin/bash" {
      args.insert(0, command);
    }

    let full_command = vec!["/bin/bash".to_string(), "-c".to_string(), args.join(" ")];

    let args = Self::build_args(full_command, workspace_root, &sandbox);
    let mut cmd = Command::new(MACOS_PATH_TO_SANDBOX_EXECUTABLE);

    cmd
      .args(args)
      .current_dir(workspace_root)
      .envs(crate::host::env::get_env_with_runtime(&runtime_config))
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .kill_on_drop(true);

    cmd.spawn().map_err(|e| ToolError::SpawnFailed(format!("[Thor] {}", e)).into())
  }

  fn build_args(
    command: Vec<String>,
    workspace_root: &Path,
    sandbox: &RunSandbox,
  ) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let write_policy = Self::build_write_policy(&home, workspace_root, sandbox);

    let read_policy = "(allow file-read*)";
    let network_policy = "(allow network-outbound)\n(allow network-inbound)\n(allow system-socket)";

    let full_policy = format!("{MACOS_SEATBELT_BASE_POLICY}\n{read_policy}\n{write_policy}\n{network_policy}");

    let mut backseat_args = vec!["-p".to_string(), full_policy];
    // backseat_args.extend(write_args);
    // backseat_args.push("--".to_string());
    backseat_args.extend(command);

    backseat_args
  }

  fn build_write_policy(home: &str, workspace_root: &Path, sandbox: &RunSandbox) -> String {
    let workspace = workspace_root.to_string_lossy().to_string();
    let paths = [
      // System temp
      "/private/tmp",
      "/tmp",
      "/var/folders",
      "/private/var/folders",
      // Homebrew
      "/opt/homebrew",
      "/usr/local",
      // Rust
      &format!("{}/.cargo", home),
      &format!("{}/.rustup", home),
      // Node/JS ecosystem
      &format!("{}/.npm", home),
      &format!("{}/.yarn", home),
      &format!("{}/.pnpm", home),
      &format!("{}/.bun", home),
      &format!("{}/.nvm", home),
      &format!("{}/.fnm", home),
      &format!("{}/.volta", home),
      // Python
      &format!("{}/.pyenv", home),
      &format!("{}/.pip", home),
      &format!("{}/.conda", home),
      &format!("{}/.virtualenvs", home),
      &format!("{}/Library/Python", home),
      // Go
      &format!("{}/go", home),
      &format!("{}/.go", home),
      // Ruby
      &format!("{}/.rbenv", home),
      &format!("{}/.rvm", home),
      &format!("{}/.gem", home),
      &format!("{}/.bundle", home),
      // Java/JVM
      &format!("{}/.m2", home),
      &format!("{}/.gradle", home),
      &format!("{}/.sdkman", home),
      // Other runtimes
      &format!("{}/.deno", home),
      // Version managers
      &format!("{}/.asdf", home),
      &format!("{}/.mise", home),
      &format!("{}/.proto", home),
      // General config/cache
      &format!("{}/.config", home),
      &format!("{}/.local", home),
      &format!("{}/.cache", home),
      &format!("{}/Library/Preferences", home),
      &format!("{}/Library/Caches", home),
      &format!("{}/Library/Logs", home),
      &format!("{}/Library/Application Support", home),
      // Git/auth (optional - security consideration)
      &format!("{}/.gnupg", home),
      // Workspace
      &workspace,
      &format!("{}/", workspace),
    ];

    let mut paths = paths.to_vec();

    let tmp_dir = std::env::temp_dir().to_string_lossy().to_string();
    if !paths.contains(&tmp_dir.as_str()) {
      paths.push(tmp_dir.as_str());
    }

    let tmp_dir = std::env::var_os("TMPDIR").map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    if !paths.contains(&tmp_dir.as_str()) {
      paths.push(tmp_dir.as_str());
    }

    let sandbox_roots = sandbox.host_paths();
    let sandbox_root_strings = sandbox_roots.iter().map(|path| path.to_string_lossy().to_string()).collect::<Vec<_>>();
    for root in &sandbox_root_strings {
      if !paths.contains(&root.as_str()) {
        paths.push(root.as_str());
      }
    }

    let subpaths: String = paths.iter().map(|p| format!(r#"(subpath "{}")"#, p)).collect::<Vec<_>>().join(" ");

    // npx create-next-app is a cunt and it checks if the workspace PARENT is writeable
    // so we need to allow the parent directory to be writeable, but narrow scope to literal
    // this is a hack, but it works for now
    // This grants the harness write access to the parent dir. It can create/delete sibling dirs but not inside of them.
    let mut literals = vec![format!(
      r#"(literal "{}")"#,
      workspace_root.parent().unwrap_or(Path::new("/")).to_string_lossy()
    )];
    for root in &sandbox_roots {
      let parent = root.parent().unwrap_or(Path::new("/"));
      let literal = format!(r#"(literal "{}")"#, parent.to_string_lossy());
      if !literals.contains(&literal) {
        literals.push(literal);
      }
    }

    // Use file-write* wildcard to allow all write operations including:
    // - file-write-create, file-write-data, file-write-unlink, file-write-times
    // - file-write-mode, file-write-flags, file-write-owner, file-write-setugid
    // This is necessary for operations like copyfile(2) which need full write access.
    format!(r#"(allow file-write* {} {subpaths})"#, literals.join(" "))
  }
}
