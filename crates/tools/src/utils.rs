use std::path::PathBuf;

pub fn get_workspace_root(working_directories: &[PathBuf], workspace_index: Option<u8>) -> PathBuf {
  let workspace_index = workspace_index.unwrap_or(0) as usize;
  let workspace_index = if working_directories.len() > workspace_index { workspace_index } else { 0 };

  working_directories.get(workspace_index).unwrap().clone()
}
