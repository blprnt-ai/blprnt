use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use sysinfo::Pid;
use sysinfo::Process;
use sysinfo::ProcessRefreshKind;
use sysinfo::ProcessesToUpdate;
use sysinfo::System;
use sysinfo::UpdateKind;

fn port_is_free(port: u16) -> bool {
  TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn wait_for_port_free(port: u16, timeout: Duration) -> bool {
  let start = Instant::now();
  while start.elapsed() < timeout {
    if port_is_free(port) {
      return true;
    }
    std::thread::sleep(Duration::from_millis(50));
  }
  port_is_free(port)
}

fn read_pid(lockfile: &Path) -> Option<u32> {
  let contents = std::fs::read_to_string(lockfile).ok()?;
  contents.trim().parse::<u32>().ok()
}

fn write_pid(lockfile: &Path, pid: u32) -> std::io::Result<()> {
  if let Some(parent) = lockfile.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(lockfile, format!("{pid}\n"))
}

fn remove_lockfile(lockfile: &Path) {
  let _ = std::fs::remove_file(lockfile);
}

fn looks_like_surreal(process: &Process) -> bool {
  let name = process.name().to_string_lossy().to_lowercase();
  if name.contains("surreal") {
    return true;
  }

  let Some(exe) = process.exe() else {
    return false;
  };
  let Some(file_name) = exe.file_name() else {
    return false;
  };
  file_name.to_string_lossy().to_lowercase().contains("surreal")
}

fn cmd_has_bind_arg(process: &Process, port: u16) -> Option<bool> {
  let bind_arg = format!("127.0.0.1:{port}");
  let cmd = process.cmd();
  if cmd.is_empty() {
    return None;
  }
  Some(cmd.iter().any(|arg| arg.to_string_lossy() == bind_arg))
}

fn refresh_process(system: &mut System, pid: Pid) {
  system.refresh_processes_specifics(
    ProcessesToUpdate::Some(&[pid]),
    true,
    ProcessRefreshKind::nothing().without_tasks().with_cmd(UpdateKind::Always).with_exe(UpdateKind::Always),
  );
}

fn refresh_all_processes(system: &mut System) {
  system.refresh_processes_specifics(
    ProcessesToUpdate::All,
    true,
    ProcessRefreshKind::nothing().without_tasks().with_cmd(UpdateKind::Always).with_exe(UpdateKind::Always),
  );
}

fn kill_surreal_pid_if_matches_port(system: &mut System, pid: Pid, port: u16, label: &str) -> bool {
  refresh_process(system, pid);
  let Some(process) = system.process(pid) else {
    return false;
  };

  if !looks_like_surreal(process) {
    return false;
  }

  // If cmdline is available, require a match for the configured bind arg. If cmdline isn't
  // available (e.g. Windows without elevated privileges), fall back to just the name/exe check.
  if let Some(matches) = cmd_has_bind_arg(process, port)
    && !matches
  {
    return false;
  }

  tracing::warn!("Surreal {}: killing existing surreal pid={} on port {}", label, pid.as_u32(), port);

  let _ = process.kill_with(sysinfo::Signal::Term);
  if wait_for_port_free(port, Duration::from_secs(2)) {
    return true;
  }

  let _ = process.kill();
  let _ = wait_for_port_free(port, Duration::from_secs(3));
  true
}

fn kill_surreal_by_cmdline_port(system: &mut System, port: u16, label: &str) -> usize {
  let bind_arg = format!("127.0.0.1:{port}");
  refresh_all_processes(system);

  let pids = system
    .processes()
    .iter()
    .filter_map(|(pid, process)| {
      if !looks_like_surreal(process) {
        return None;
      }
      if process.cmd().iter().any(|arg| arg.to_string_lossy() == bind_arg) { Some(*pid) } else { None }
    })
    .collect::<Vec<_>>();

  for pid in &pids {
    if let Some(process) = system.process(*pid) {
      tracing::warn!(
        "Surreal {}: killing surreal pid={} found via cmdline match for port {}",
        label,
        (*pid).as_u32(),
        port
      );
      let _ = process.kill();
    }
  }

  if !pids.is_empty() {
    let _ = wait_for_port_free(port, Duration::from_secs(3));
  }

  pids.len()
}

/// Ensures the given port is free to bind before spawning Surreal.
///
/// Uses a PID lockfile so we only kill the Surreal process we previously spawned.
/// If the lockfile is missing (e.g. first run after an update), falls back to killing
/// any Surreal process whose cmdline explicitly binds `127.0.0.1:<port>`.
pub fn ensure_surreal_port_is_free(port: u16, lockfile: &Path, label: &str) {
  if port_is_free(port) {
    remove_lockfile(lockfile);
    return;
  }

  let mut system = System::new();

  if let Some(pid) = read_pid(lockfile) {
    let pid = Pid::from_u32(pid);
    if kill_surreal_pid_if_matches_port(&mut system, pid, port, label) && port_is_free(port) {
      remove_lockfile(lockfile);
      return;
    }
  } else {
    // Stale/invalid lockfile. Remove it so we can rewrite it after spawn.
    remove_lockfile(lockfile);
  }

  // Fallback: find a Surreal process explicitly bound to this port and kill it.
  let _ = kill_surreal_by_cmdline_port(&mut system, port, label);

  if port_is_free(port) {
    remove_lockfile(lockfile);
  } else {
    tracing::error!("Surreal {}: port {} is still in use; surreal spawn may fail", label, port);
  }
}

pub fn write_surreal_pid_lock(lockfile: &Path, pid: u32) {
  if let Err(err) = write_pid(lockfile, pid) {
    tracing::warn!("Surreal: failed to write pid lockfile at {:?}: {:?}", lockfile, err);
  }
}

pub fn clear_surreal_pid_lock(lockfile: &Path) {
  remove_lockfile(lockfile);
}
