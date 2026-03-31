#![warn(unused, unused_crate_dependencies)]

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use cap_async_std::ambient_authority;
use cap_async_std::fs::Dir;
use cap_async_std::fs::File;
use cap_async_std::fs::OpenOptions;
use shared::errors::SandboxError;

const RUN_SANDBOX_NAME: &str = "run";

#[derive(Clone, Debug)]
pub struct Root {
  pub dir:       Dir,
  pub host_path: PathBuf,
}

impl Root {
  pub fn contains(&self, candidate: &Path) -> bool {
    let Ok(root) = canonicalize_path(&self.host_path) else {
      return false;
    };

    let (cur, _) = peel_to_existing(candidate);
    let Ok(cur_c) = canonicalize_path(&cur) else {
      return false;
    };

    let mut probe = cur_c.clone();
    loop {
      if same_file::is_same_file(&probe, &root).unwrap_or(false) {
        return true;
      }

      if !probe.pop() {
        return false;
      }
    }
  }
}

impl PartialEq for Root {
  fn eq(&self, other: &Self) -> bool {
    if self.host_path.eq(&other.host_path) {
      return true;
    }

    self.host_path.display().to_string() == other.host_path.display().to_string()
  }
}

impl Eq for Root {}

#[derive(Clone, Debug)]
pub struct RunSandbox {
  roots: Vec<Root>,
}

impl RunSandbox {
  pub async fn new(roots: &[PathBuf]) -> anyhow::Result<Self> {
    let mut sandbox = Self { roots: Vec::new() };
    sandbox.add_tmp_root().await;

    for root in roots {
      if root.exists() {
        sandbox.add_dir(root).await?;
      }
    }

    Ok(sandbox)
  }

  pub fn host_paths(&self) -> Vec<PathBuf> {
    self.roots.iter().map(|root| root.host_path.clone()).collect()
  }

  pub fn contains_workspace(&self, path: &Path) -> bool {
    let Ok(path) = canonicalize_path(path) else {
      tracing::debug!("error canonicalizing path: {}", path.display());
      return false;
    };

    self.roots.iter().any(|root| {
      let eq = path.display().to_string() == root.host_path.display().to_string();
      tracing::debug!("root: {}", root.host_path.display());
      tracing::debug!("eq: {}", eq);
      eq
    })
  }

  pub fn walker(
    &self,
    workspace_root: &PathBuf,
    rel: impl AsRef<Path>,
    max_depth: Option<usize>,
    explicitly_include_hidden: Option<bool>,
  ) -> anyhow::Result<GuardedWalk> {
    if !self.contains_workspace(workspace_root) {
      return Err(SandboxError::WorkspaceNotInSandbox {
        name: RUN_SANDBOX_NAME.to_string(),
        path: workspace_root.display().to_string(),
      })?;
    }

    let root = self.root_for_workspace(workspace_root)?;
    let start = root.host_path.join(rel.as_ref());

    let mut walk_builder = &mut ignore::WalkBuilder::new(start);

    if PathBuf::from(workspace_root).join(".blprntignore").exists() {
      walk_builder = walk_builder.add_custom_ignore_filename(".blprntignore");
    }

    if let Some(max_depth) = max_depth {
      walk_builder = walk_builder.max_depth(Some(max_depth));
    }

    if let Some(explicitly_include_hidden) = explicitly_include_hidden {
      walk_builder = walk_builder.hidden(!explicitly_include_hidden);
    }

    Ok(GuardedWalk { walk: walk_builder.build(), allow_prefix: root.host_path.clone() })
  }

  fn root_for_workspace(&self, workspace_root: &Path) -> anyhow::Result<&Root> {
    self.roots.iter().find(|root| root.contains(workspace_root)).ok_or_else(|| {
      SandboxError::RootNotInSandbox {
        name: RUN_SANDBOX_NAME.to_string(),
        path: workspace_root.display().to_string(),
      }
      .into()
    })
  }

  async fn add_dir(&mut self, dir: &PathBuf) -> anyhow::Result<()> {
    let abs = canonicalize_path(dir)?;
    let dir = Dir::open_ambient_dir(dir, ambient_authority())
      .await
      .map_err(|e| SandboxError::FailedToOpenDirectory { path: dir.display().to_string(), error: e.to_string() })?;

    self.roots.push(Root { dir, host_path: abs.clone() });
    tracing::debug!("added sandbox root: {}", abs.display());
    Ok(())
  }

  async fn add_tmp_root(&mut self) {
    let tmp_dir = std::env::temp_dir();
    let Ok(tmp_dir) = canonicalize_path(&tmp_dir) else {
      tracing::error!("failed to canonicalize temp dir for sandbox root");
      return;
    };

    let Ok(dir) = Dir::open_ambient_dir(&tmp_dir, ambient_authority()).await else {
      tracing::error!("failed to open temp dir for sandbox root");
      return;
    };

    self.roots.push(Root { dir, host_path: tmp_dir });
  }
}

pub struct GuardedWalk {
  walk:         ignore::Walk,
  allow_prefix: PathBuf,
}

impl Iterator for GuardedWalk {
  type Item = ignore::DirEntry;

  fn next(&mut self) -> Option<Self::Item> {
    for res in self.walk.by_ref() {
      match res {
        Ok(dent) => {
          if dent.path().starts_with(&self.allow_prefix) {
            return Some(dent);
          }
        }
        Err(_) => continue,
      }
    }
    None
  }
}

pub async fn open_read_only(abs: &Path) -> anyhow::Result<File> {
  let mut opts = OpenOptions::new();
  opts.read(true);
  File::open_ambient_with(abs, &opts, ambient_authority())
    .await
    .map_err(|e| SandboxError::FailedToOpenFile { path: abs.display().to_string(), error: e.to_string() }.into())
}

pub async fn open_write_only(sandbox: &RunSandbox, workspace_root: &Path, abs: &Path) -> anyhow::Result<File> {
  if !sandbox.contains_workspace(workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let root = sandbox.root_for_workspace(workspace_root)?;

  let Some(rel) = rel_in_root(root, abs)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs.display().to_string(),
    })?;
  };

  let mut opts = OpenOptions::new();
  opts.write(true).truncate(true);

  root
    .dir
    .open_with(&rel, &opts)
    .await
    .map_err(|e| SandboxError::FailedToOpenWriteOnlyFile { path: rel.display().to_string(), error: e.to_string() }.into())
}

pub async fn remove_file(sandbox: &RunSandbox, workspace_root: &Path, abs: &Path) -> anyhow::Result<()> {
  if !sandbox.contains_workspace(workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let root = sandbox.root_for_workspace(workspace_root)?;

  let Some(rel) = rel_in_root(root, abs)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs.display().to_string(),
    })?;
  };

  root
    .dir
    .remove_file(&rel)
    .await
    .map_err(|e| SandboxError::FailedToRemoveFile { path: rel.display().to_string(), error: e.to_string() }.into())
}

pub async fn create_with_parents(
  sandbox: &RunSandbox,
  workspace_root: &Path,
  abs: &Path,
  opts: &OpenOptions,
) -> anyhow::Result<File> {
  if !sandbox.contains_workspace(workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let root = sandbox.root_for_workspace(workspace_root)?;
  let base_dir = abs.parent().ok_or_else(|| SandboxError::InvalidFilePath {
    path:  abs.display().to_string(),
    error: "missing parent directory".to_string(),
  })?;

  ensure_parent_dirs(root, base_dir)?;

  let Some(_) = rel_in_root(root, base_dir)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: base_dir.display().to_string(),
    })?;
  };

  let rel_file = abs
    .strip_prefix(&root.host_path)
    .map_err(|_| SandboxError::RootNotInSandbox { name: RUN_SANDBOX_NAME.to_string(), path: abs.display().to_string() })?;

  root
    .dir
    .open_with(rel_file, opts)
    .await
    .map_err(|e| SandboxError::FailedToCreateFile { path: rel_file.display().to_string(), error: e.to_string() }.into())
}

fn ensure_parent_dirs(root: &Root, abs_file: &Path) -> anyhow::Result<()> {
  let Some(rel) = rel_in_root(root, abs_file)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: RUN_SANDBOX_NAME.to_string(),
      path: abs_file.display().to_string(),
    })?;
  };

  std::fs::create_dir_all(abs_file).map_err(|e| SandboxError::FailedToCreateParentDirectories {
    path:  rel.display().to_string(),
    error: e.to_string(),
  })?;

  Ok(())
}

fn rel_in_root(root: &Root, abs: &Path) -> anyhow::Result<Option<PathBuf>> {
  let root_c = canonicalize_path(&root.host_path)?;
  let (existing, tail) = peel_to_existing(abs);
  let existing_c = canonicalize_path(&existing)?;

  let mut anc = existing_c.clone();
  while !same_file::is_same_file(&anc, &root_c).unwrap_or(false) {
    if !anc.pop() {
      return Ok(None);
    }
  }
  let rel_existing = existing_c.strip_prefix(&anc).unwrap_or_else(|_| Path::new(""));

  if rel_existing.is_dir() { Ok(Some(rel_existing.join(tail))) } else { Ok(Some(rel_existing.to_path_buf())) }
}

fn canonicalize_path(path: &Path) -> anyhow::Result<PathBuf> {
  dunce::canonicalize(path)
    .map_err(|e| SandboxError::InvalidFilePath { path: path.display().to_string(), error: e.to_string() })?;

  Ok(path.to_path_buf())
}

fn peel_to_existing(path: &Path) -> (PathBuf, PathBuf) {
  let mut base = path.to_path_buf();
  let mut suffix = PathBuf::new();
  while !base.exists() {
    if let Some(name) = base.file_name() {
      suffix = PathBuf::from(name).join(suffix);
    }
    if !base.pop() {
      break;
    }
  }
  (base, suffix)
}

pub async fn sandbox_test_setup(test_dir: &PathBuf) -> anyhow::Result<Arc<RunSandbox>> {
  std::fs::create_dir_all(test_dir).map_err(|e| anyhow::anyhow!("failed to create test directory: {}", e))?;
  Ok(Arc::new(RunSandbox::new(std::slice::from_ref(test_dir)).await?))
}
