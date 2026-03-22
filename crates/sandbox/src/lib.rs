#![warn(unused, unused_crate_dependencies)]

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use cap_async_std::ambient_authority;
use cap_async_std::fs::Dir;
use cap_async_std::fs::File;
use cap_async_std::fs::OpenOptions;
use once_cell::sync::Lazy;
use shared::errors::SandboxError;
use tokio::sync::RwLock;

pub static SANDBOX: Lazy<Arc<RwLock<SandBox>>> = Lazy::new(|| Arc::new(RwLock::new(SandBox::new())));

// Convenience API
pub fn get_sandbox() -> Arc<RwLock<SandBox>> {
  SANDBOX.clone()
}

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

    let path_display = self.host_path.display().to_string();
    let other_path_display = other.host_path.display().to_string();

    path_display == other_path_display
  }
}

impl Eq for Root {}

#[derive(Clone, Debug)]
pub struct SandBox {
  roots: HashMap<String, Vec<Root>>,
}

impl SandBox {
  fn new() -> Self {
    Self { roots: HashMap::new() }
  }

  pub async fn add_dir(&mut self, name: impl Into<String>, dir: &PathBuf) -> anyhow::Result<()> {
    let name: String = name.into();
    let abs = canonicalize_path(dir)?;
    let dir = Dir::open_ambient_dir(dir, ambient_authority())
      .await
      .map_err(|e| SandboxError::FailedToOpenDirectory { path: dir.display().to_string(), error: e.to_string() })?;

    self.add_tmp_to_root(&name).await;

    let root = Root { dir, host_path: abs.clone() };
    let mut roots = self.roots.get(&name).cloned().unwrap_or_default();
    roots.push(root);

    self.roots.insert(name.clone(), roots);

    tracing::debug!("added sandbox root: {}", abs.display());

    Ok(())
  }

  pub async fn remove_dir(&mut self, name: &str, dir: &Path) -> anyhow::Result<()> {
    let name: String = name.into();
    let abs = canonicalize_path(dir)?;

    let mut roots = self.roots.get(&name).cloned().unwrap_or_default();
    roots.retain(|r| !same_file::is_same_file(&r.host_path, &abs).unwrap_or(false));

    self.roots.insert(name.clone(), roots);

    Ok(())
  }

  async fn add_tmp_to_root(&mut self, name: &str) {
    if self.roots.contains_key(name) {
      return;
    }

    let tmp_dir = std::env::temp_dir();
    let Ok(tmp_dir) = canonicalize_path(&tmp_dir) else {
      tracing::error!("Failed to canonicalize temp dir for sandbox root");
      return;
    };

    let Ok(dir) = Dir::open_ambient_dir(&tmp_dir, ambient_authority()).await else {
      tracing::error!("Failed to open temp dir for sandbox root");
      return;
    };

    let root = Root { dir, host_path: tmp_dir };
    let mut roots = self.roots.get(name).cloned().unwrap_or_default();
    roots.push(root);

    self.roots.insert(name.to_string(), roots);
  }

  pub fn remove_root(&mut self, name: &str) {
    tracing::debug!("removed sandbox root: {}", name);
    self.roots.remove(name);
  }

  pub fn get_dirs(&self, name: &str) -> Vec<Root> {
    tracing::debug!("getting sandbox root: {}", name);
    self.roots.get(name).cloned().unwrap_or_else(|| panic!("[Sandbox - get_dirs] unknown sandbox root: {}", name))
  }

  pub fn is_workspace_in_roots(&self, name: &str, path: &Path) -> bool {
    let roots = self.get_dirs(name);

    let Ok(path) = canonicalize_path(path) else {
      tracing::debug!("error canonicalizing path: {}", path.display());
      return false;
    };

    tracing::debug!("path: {}", path.display());

    roots.iter().any(|root| {
      let path_display = path.display().to_string();
      let root_display = root.host_path.display().to_string();
      let eq = path_display == root_display;

      tracing::debug!("root: {}", root.host_path.display());
      tracing::debug!("eq: {}", eq);

      eq
    })
  }

  pub fn walker(
    &self,
    workspace_root: &PathBuf,
    sandbox_key: &str,
    rel: impl AsRef<Path>,
    max_depth: Option<usize>,
    explicitly_include_hidden: Option<bool>,
  ) -> anyhow::Result<GuardedWalk> {
    let roots = self.roots.get(sandbox_key).ok_or_else(|| SandboxError::UnknownRoot(sandbox_key.to_string()))?;

    if !self.is_workspace_in_roots(sandbox_key, workspace_root) {
      return Err(SandboxError::WorkspaceNotInSandbox {
        name: sandbox_key.to_string(),
        path: workspace_root.display().to_string(),
      })?;
    }

    tracing::debug!("roots: {:#?}", roots);

    let root = roots.iter().find(|root| same_file::is_same_file(&root.host_path, workspace_root).unwrap_or(false));

    tracing::debug!("root: {:?}", root);

    let root = root.ok_or_else(|| SandboxError::RootNotInSandbox {
      name: sandbox_key.to_string(),
      path: workspace_root.display().to_string(),
    })?;
    let start = root.host_path.join(rel.as_ref());

    tracing::debug!("START: {}", start.display());

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

    let walk = walk_builder.build();

    tracing::debug!("walk done");

    Ok(GuardedWalk { walk, allow_prefix: root.host_path.clone() })
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
          let p = dent.path();
          if p.starts_with(&self.allow_prefix) {
            return Some(dent);
          }

          continue;
        }
        Err(_) => continue,
      }
    }
    None
  }
}

pub async fn open_read_only(abs: &Path) -> anyhow::Result<cap_async_std::fs::File> {
  let mut opts = OpenOptions::new();
  opts.read(true);
  let file = cap_async_std::fs::File::open_ambient_with(abs, &opts, ambient_authority())
    .await
    .map_err(|e| SandboxError::FailedToOpenFile { path: abs.display().to_string(), error: e.to_string() })?;

  Ok(file)
}

pub async fn open_write_only(
  sandbox_key: &str,
  workspace_root: &Path,
  abs: &Path,
) -> anyhow::Result<cap_async_std::fs::File> {
  let sandbox = get_sandbox();
  let sandbox = sandbox.read().await;
  if !sandbox.is_workspace_in_roots(sandbox_key, workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: sandbox_key.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let roots = sandbox.get_dirs(sandbox_key);
  let root = roots.iter().find(|root| root.contains(workspace_root)).ok_or_else(|| SandboxError::RootNotInSandbox {
    name: sandbox_key.to_string(),
    path: workspace_root.display().to_string(),
  })?;

  let Some(rel) = rel_in_root(root, abs)? else {
    return Err(SandboxError::RootNotInSandbox { name: sandbox_key.to_string(), path: abs.display().to_string() })?;
  };
  let mut opts = OpenOptions::new();
  opts.write(true).truncate(true);

  let file = root
    .dir
    .open_with(&rel, &opts)
    .await
    .map_err(|e| SandboxError::FailedToOpenWriteOnlyFile { path: rel.display().to_string(), error: e.to_string() })?;

  Ok(file)
}

pub async fn remove_file(sandbox_key: &str, workspace_root: &Path, abs: &Path) -> anyhow::Result<()> {
  let sandbox = get_sandbox();
  let sandbox = sandbox.read().await;
  if !sandbox.is_workspace_in_roots(sandbox_key, workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: sandbox_key.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let roots = sandbox.get_dirs(sandbox_key);
  let root = roots.iter().find(|root| root.contains(workspace_root)).ok_or_else(|| SandboxError::RootNotInSandbox {
    name: sandbox_key.to_string(),
    path: workspace_root.display().to_string(),
  })?;

  let Some(rel) = rel_in_root(root, abs)? else {
    return Err(SandboxError::RootNotInSandbox { name: sandbox_key.to_string(), path: abs.display().to_string() })?;
  };

  root
    .dir
    .remove_file(&rel)
    .await
    .map_err(|e| SandboxError::FailedToRemoveFile { path: rel.display().to_string(), error: e.to_string() })?;

  Ok(())
}

pub async fn create_with_parents(
  sandbox_key: &str,
  workspace_root: &Path,
  abs: &Path,
  opts: &OpenOptions,
) -> anyhow::Result<File> {
  let sandbox = get_sandbox();
  let sandbox = sandbox.read().await;
  if !sandbox.is_workspace_in_roots(sandbox_key, workspace_root) {
    return Err(SandboxError::WorkspaceNotInSandbox {
      name: sandbox_key.to_string(),
      path: abs.display().to_string(),
    })?;
  }

  let roots = sandbox.get_dirs(sandbox_key);
  let root = roots.iter().find(|root| root.contains(workspace_root)).ok_or_else(|| SandboxError::RootNotInSandbox {
    name: sandbox_key.to_string(),
    path: workspace_root.display().to_string(),
  })?;

  tracing::debug!("Creating file: {}", abs.display());
  let base_dir = abs.parent().ok_or_else(|| SandboxError::InvalidFilePath {
    path:  abs.display().to_string(),
    error: "missing parent directory".to_string(),
  })?;
  ensure_parent_dirs(sandbox_key, root, base_dir).await?;

  tracing::debug!("Ensured parent directories: {}", base_dir.display());
  let Some(_) = rel_in_root(root, base_dir)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: sandbox_key.to_string(),
      path: base_dir.display().to_string(),
    })?;
  };

  tracing::debug!("Opening file: {}", abs.display());
  tracing::debug!("Options: {:#?}", opts);

  let rel_file = abs
    .strip_prefix(&root.host_path)
    .map_err(|_| SandboxError::RootNotInSandbox { name: sandbox_key.to_string(), path: abs.display().to_string() })?;

  let file = root
    .dir
    .open_with(rel_file, opts)
    .await
    .map_err(|e| SandboxError::FailedToCreateFile { path: rel_file.display().to_string(), error: e.to_string() })?;

  Ok(file)
}

async fn ensure_parent_dirs(sandbox_key: &str, root: &Root, abs_file: &Path) -> anyhow::Result<()> {
  let Some(rel) = rel_in_root(root, abs_file)? else {
    return Err(SandboxError::RootNotInSandbox {
      name: sandbox_key.to_string(),
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
    } // root reached
  }
  (base, suffix)
}

pub async fn sandbox_test_setup(test_dir: &PathBuf) -> anyhow::Result<()> {
  let sandbox = get_sandbox();
  let mut sandbox = sandbox.write().await;

  std::fs::create_dir_all(test_dir).map_err(|e| anyhow::anyhow!("Failed to create test directory: {}", e))?;
  sandbox.add_dir("test", test_dir).await.unwrap();
  Ok(())
}
