use std::collections::BTreeMap;
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
use seccompiler::BpfProgram;
use seccompiler::SeccompAction;
use seccompiler::SeccompCmpArgLen;
use seccompiler::SeccompCmpOp;
use seccompiler::SeccompCondition;
use seccompiler::SeccompFilter;
use seccompiler::SeccompRule;
use seccompiler::TargetArch;
use seccompiler::apply_filter;
use shared::errors::ToolError;
use shared::sandbox_flags::SandboxFlags;
use tokio::process::Child;
use tokio::process::Command;

pub struct Loki;

impl Loki {
  pub fn exec(
    workspace_root: &PathBuf,
    command: String,
    args: Vec<String>,
    sandbox_flags: SandboxFlags,
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
    cmd.envs(crate::host::env::get_env());

    if !sandbox_flags.is_yolo() {
      let ruleset = Self::build_ruleset(workspace_root)?;
      let prog = if !sandbox_flags.is_network_access() { Some(Self::install_network_filter()?) } else { None };

      unsafe {
        cmd.pre_exec(move || {
          let _ = ruleset.try_clone().unwrap().restrict_self();

          if let Some(prog) = &prog {
            apply_filter(&prog).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
          } else {
            Ok(())
          }
        });
      }
    }

    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);

    cmd.spawn().map_err(|e| ToolError::SpawnFailed(e.to_string()).into())
  }

  fn build_ruleset(workspace_root: &PathBuf) -> anyhow::Result<RulesetCreated> {
    let abi = ABI::V5;
    let access_rw = AccessFs::from_all(abi);
    let access_ro = AccessFs::from_read(abi);

    let ruleset = Ruleset::default()
      .set_compatibility(CompatLevel::BestEffort)
      .handle_access(access_rw)?
      .create()?
      .add_rules(landlock::path_beneath_rules(&["/"], access_ro))?
      .add_rules(landlock::path_beneath_rules(&["/dev/null"], access_rw))?
      .add_rules(landlock::path_beneath_rules(&[workspace_root.as_path()], access_rw))?
      .add_rules(landlock::path_beneath_rules(&[std::env::temp_dir().as_path()], access_rw))?
      .set_no_new_privs(true);

    Ok(ruleset)
  }

  fn install_network_filter() -> anyhow::Result<BpfProgram> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    let mut deny_syscall = |nr: i64| {
      rules.insert(nr, vec![]);
    };

    deny_syscall(libc::SYS_connect);
    deny_syscall(libc::SYS_accept);
    deny_syscall(libc::SYS_accept4);
    deny_syscall(libc::SYS_bind);
    deny_syscall(libc::SYS_listen);
    deny_syscall(libc::SYS_getpeername);
    deny_syscall(libc::SYS_getsockname);
    deny_syscall(libc::SYS_shutdown);
    deny_syscall(libc::SYS_sendto);
    deny_syscall(libc::SYS_sendmsg);
    deny_syscall(libc::SYS_sendmmsg);
    deny_syscall(libc::SYS_recvmsg);
    deny_syscall(libc::SYS_recvmmsg);
    deny_syscall(libc::SYS_getsockopt);
    deny_syscall(libc::SYS_setsockopt);
    deny_syscall(libc::SYS_ptrace);

    let unix_only_rule = SeccompRule::new(vec![SeccompCondition::new(
      0,
      SeccompCmpArgLen::Dword,
      SeccompCmpOp::Ne,
      libc::AF_UNIX as u64,
    )?])?;

    rules.insert(libc::SYS_socket, vec![unix_only_rule.clone()]);
    rules.insert(libc::SYS_socketpair, vec![unix_only_rule]);

    let filter = SeccompFilter::new(
      rules,
      SeccompAction::Allow,
      SeccompAction::Errno(libc::EPERM as u32),
      if cfg!(target_arch = "x86_64") {
        TargetArch::x86_64
      } else if cfg!(target_arch = "aarch64") {
        TargetArch::aarch64
      } else {
        unimplemented!("unsupported architecture for seccomp filter");
      },
    )?;

    let prog: BpfProgram = filter.try_into()?;

    Ok(prog)
  }
}
