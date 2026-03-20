use std::path::Path;
use std::path::PathBuf;

use shared::errors::MemoryResult;
use shared::paths;

const MEMORY_DIR: &str = "memories";

pub fn memories_root() -> PathBuf {
  paths::blprnt_home().join(MEMORY_DIR)
}

pub fn write(path: &Path, content: &str) -> MemoryResult<()> {
  ensure_no_backward_directory_traversal(path)?;

  let path = memories_root().join(path);

  std::fs::write(path, content)?;

  Ok(())
}

pub fn read(path: &Path) -> MemoryResult<String> {
  ensure_no_backward_directory_traversal(path)?;

  let path = memories_root().join(path);
  let content = std::fs::read_to_string(&path)?;

  Ok(content)
}

pub fn append(path: &Path, content: &str) -> MemoryResult<()> {
  ensure_no_backward_directory_traversal(path)?;

  let previous_content = read(path)?;

  write(path, &format!("{}\n\n{}", previous_content, content))
}

pub fn ensure_dir(path: &Path) -> MemoryResult<()> {
  ensure_no_backward_directory_traversal(path)?;

  let path = memories_root().join(path);

  std::fs::create_dir_all(&path)?;

  Ok(())
}

pub fn list_dirs(path: &Path) -> MemoryResult<Vec<String>> {
  ensure_no_backward_directory_traversal(path)?;

  let path = memories_root().join(path);

  let dirs = std::fs::read_dir(&path)?
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.path().is_dir())
    .filter_map(|entry| entry.path().file_name().map(|name| name.to_string_lossy().to_string()))
    .collect();

  Ok(dirs)
}

fn ensure_no_backward_directory_traversal(path: &Path) -> MemoryResult<()> {
  if path.to_string_lossy().contains("..") {
    Err(shared::errors::MemoryError::InvalidPath(path.to_string_lossy().to_string()))
  } else {
    Ok(())
  }
}
