use std::path::PathBuf;
use std::process::Stdio;

use anyhow::Result;
use landlock::ABI;
use landlock::Access;
use landlock::AccessFs;
use landlock::CompatLevel;
use landlock::Compatible;
use landlock::Ruleset;
use landlock::RulesetAttr;
use landlock::RulesetCreated;
use landlock::RulesetCreatedAttr;
use sandbox::RunSandbox;
use shared::errors::ToolError;
use shared::tools::config::ToolRuntimeConfig;
use tokio::process::Child;
use tokio::process::Command;

pub struct Loki;

impl Loki {
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

    let mut cmd = Command::new("/bin/bash");
    let args = vec!["-c".to_string(), args.join(" ")];

    cmd.args(args);
    cmd.current_dir(workspace_root);
    cmd.envs(crate::host::env::get_env_with_runtime(&runtime_config));

    let ruleset = Self::build_ruleset(&sandbox)?;
    unsafe {
      cmd.pre_exec(move || {
        let _ = ruleset.try_clone().unwrap().restrict_self();
        Ok(())
      });
    }

    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);

    cmd.spawn().map_err(|e| ToolError::SpawnFailed(e.to_string()).into())
  }

  fn build_ruleset(sandbox: &RunSandbox) -> anyhow::Result<RulesetCreated> {
    let abi = ABI::V5;
    let access_rw = AccessFs::from_all(abi);
    let access_ro = AccessFs::from_read(abi);

    let ruleset = Ruleset::default()
      .set_compatibility(CompatLevel::BestEffort)
      .handle_access(access_rw)?
      .create()?
      .add_rules(landlock::path_beneath_rules(&["/"], access_ro))?
      .add_rules(landlock::path_beneath_rules(&["/dev/null"], access_rw))?
      .add_rules(landlock::path_beneath_rules(&[std::env::temp_dir().as_path()], access_rw))?
      .set_no_new_privs(true);

    let ruleset = sandbox
      .host_paths()
      .into_iter()
      .try_fold(ruleset, |ruleset, path| ruleset.add_rules(landlock::path_beneath_rules(&[path.as_path()], access_rw)))?;

    Ok(ruleset)
  }
}
