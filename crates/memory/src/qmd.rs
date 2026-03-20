#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

use crate::store;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/**
 * dir structure:
 *
 * memories/
 *  <employee_id>/
 *    .learnings/
 *      ERRORS.md
 *    life/
 *      archives/
 *      people/<name>/
 *        summary.md
 *        items.yaml
 *      projects/<name>/
 *        summary.md
 *        items.yaml
 *      resources/<topic>/
 *        summary.md
 *        items.yaml
 *    memory/<date>.md
 *    AGENTS.md
 *    HEARTBEAT.md
 *    MEMORY.md
 *    SOUL.md
 *    TOOLS.md
 *
 */

pub fn init_new_employee(
  employee_id: &EmployeeId,
  agents: &str,
  heartbeat: &str,
  soul: &str,
  tools: &str,
) -> MemoryResult<()> {
  let employee_id = employee_id.uuid().to_string();
  let employee_dir = Path::new(&employee_id);

  store::ensure_dir(employee_dir)?;

  store::write(&employee_dir.join("AGENTS.md"), agents)?;
  store::write(&employee_dir.join("HEARTBEAT.md"), heartbeat)?;
  store::write(&employee_dir.join("SOUL.md"), soul)?;
  store::write(&employee_dir.join("TOOLS.md"), tools)?;

  ensure_qmd()?;

  init_collection(&employee_id)?;

  Ok(())
}

fn ensure_qmd() -> MemoryResult<()> {
  let output = Command::new("qmd").arg("--version").output();

  if output.is_err() || !output.unwrap().status.success() {
    install_qmd()?;
  }

  Ok(())
}

fn install_qmd() -> MemoryResult<()> {
  let output = command("npx", &["i", "-g", "@tobilu/qmd"]).output();

  if output.is_err() || !output.unwrap().status.success() { Err(MemoryError::QmdInstallationFailed) } else { Ok(()) }
}

fn init_collection(employee_id: &str) -> MemoryResult<()> {
  let collection_path = store::memories_root().join(employee_id);
  let output = command("qmd", &["collection", "add", &collection_path.to_string_lossy().to_string()]).output();

  if output.is_err() || !output.unwrap().status.success() {
    Err(MemoryError::QmdCollectionInitializationFailed)
  } else {
    Ok(())
  }
}

fn command(command: &str, args: &[&str]) -> Command {
  let mut cmd = Command::new(command);
  cmd.args(args);

  #[cfg(windows)]
  cmd.creation_flags(CREATE_NO_WINDOW);

  cmd
}
