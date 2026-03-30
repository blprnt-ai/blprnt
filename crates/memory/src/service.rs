use std::cmp::Ordering;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use chrono::Local;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectRepository;
use shared::errors::DatabaseError;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

use crate::MemoryListResult;
use crate::MemoryReadResult;
use crate::MemorySearchResult;
use crate::MemorySearchResultItem;
use crate::MemoryTreeNode;
use crate::MemoryWriteResult;
use crate::MemoryWriteStatus;
use crate::qmd;

const MEMORY_DIR: &str = "memories";
const MEMORY_BASE_DIR_ENV: &str = "BLPRNT_MEMORY_BASE_DIR";
const EMPLOYEES_DIRECTORY: &str = "employees";
const PROJECTS_DIRECTORY: &str = "projects";
const EMPLOYEE_NOTES_DIRECTORY: &str = "memory";
const PROJECT_SUMMARY_FILE: &str = "SUMMARY.md";
const AGENT_HOME_ALIAS: &str = "$AGENT_HOME";
const PROJECT_HOME_ALIAS: &str = "$PROJECT_HOME";

#[derive(Clone, Debug)]
pub struct ProjectMemoryService {
  inner: ScopedMemoryService,
}

#[derive(Clone, Debug)]
pub struct EmployeeMemoryService {
  inner: ScopedMemoryService,
}

impl ProjectMemoryService {
  pub async fn new(project_id: ProjectId) -> MemoryResult<Self> {
    Ok(Self { inner: ScopedMemoryService::new(MemoryScope::Project(project_id)).await? })
  }

  pub async fn create(&self, content: &str) -> MemoryResult<MemoryWriteResult> {
    self.inner.create(None, content).await
  }

  pub async fn create_at(&self, path: &str, content: &str) -> MemoryResult<MemoryWriteResult> {
    self.inner.create(Some(path), content).await
  }

  pub async fn list(&self) -> MemoryResult<MemoryListResult> {
    self.inner.list().await
  }

  pub async fn read(&self, path: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.read(path).await
  }

  pub async fn update(&self, path: &str, content: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.update(path, content).await
  }

  pub async fn delete(&self, path: &str) -> MemoryResult<()> {
    self.inner.delete(path).await
  }

  pub async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.inner.search(query, limit).await
  }
}

impl EmployeeMemoryService {
  pub async fn new(employee_id: EmployeeId) -> MemoryResult<Self> {
    Ok(Self { inner: ScopedMemoryService::new(MemoryScope::Employee(employee_id)).await? })
  }

  pub async fn create(&self, content: &str) -> MemoryResult<MemoryWriteResult> {
    self.inner.create(None, content).await
  }

  pub async fn create_at(&self, path: &str, content: &str) -> MemoryResult<MemoryWriteResult> {
    self.inner.create(Some(path), content).await
  }

  pub async fn list(&self) -> MemoryResult<MemoryListResult> {
    self.inner.list().await
  }

  pub async fn read(&self, path: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.read(path).await
  }

  pub async fn update(&self, path: &str, content: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.update(path, content).await
  }

  pub async fn delete(&self, path: &str) -> MemoryResult<()> {
    self.inner.delete(path).await
  }

  pub async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.inner.search(query, limit).await
  }
}

#[derive(Clone, Debug)]
struct ScopedMemoryService {
  root:            PathBuf,
  collection_name: String,
  scope:           MemoryScope,
}

impl ScopedMemoryService {
  async fn new(scope: MemoryScope) -> MemoryResult<Self> {
    scope.ensure_exists().await?;

    let root = scope.root()?;
    fs::create_dir_all(&root)?;

    let collection_name = scope.collection_name();
    qmd::ensure_collection(&collection_name, &root).await?;

    Ok(Self { root, collection_name, scope })
  }

  async fn create(&self, path: Option<&str>, content: &str) -> MemoryResult<MemoryWriteResult> {
    let date = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let path = path.map(str::to_string).unwrap_or_else(|| self.scope.default_create_path(&date));
    let absolute_path = self.resolve_path(&path)?;

    if let Some(parent) = absolute_path.parent() {
      fs::create_dir_all(parent)?;
    }

    let next_content = match fs::read_to_string(&absolute_path) {
      Ok(existing) if !existing.trim().is_empty() && !content.trim().is_empty() => {
        format!("{existing}\n\n{content}")
      }
      Ok(existing) => format!("{existing}{content}"),
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => content.to_string(),
      Err(error) => return Err(error.into()),
    };

    fs::write(&absolute_path, next_content)?;
    self.sync_qmd().await?;

    Ok(MemoryWriteResult { status: MemoryWriteStatus::Written, path, date })
  }

  async fn list(&self) -> MemoryResult<MemoryListResult> {
    Ok(MemoryListResult {
      root_path: self.scope.scope_root_alias().to_string(),
      nodes:     list_memory_tree(&self.root, Path::new(""))?,
    })
  }

  async fn read(&self, path: &str) -> MemoryResult<MemoryReadResult> {
    let absolute_path = self.resolve_path(path)?;
    let content = fs::read_to_string(&absolute_path)?;

    Ok(MemoryReadResult { path: path.to_string(), content })
  }

  async fn update(&self, path: &str, content: &str) -> MemoryResult<MemoryReadResult> {
    let absolute_path = self.resolve_path(path)?;
    if let Some(parent) = absolute_path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(&absolute_path, content)?;
    self.sync_qmd().await?;

    Ok(MemoryReadResult { path: path.to_string(), content: content.to_string() })
  }

  async fn delete(&self, path: &str) -> MemoryResult<()> {
    let absolute_path = self.resolve_path(path)?;
    fs::remove_file(absolute_path)?;
    self.sync_qmd().await?;

    Ok(())
  }

  async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.sync_qmd().await?;

    let memories = qmd::search_collection(&self.collection_name, query, limit)
      .await?
      .into_iter()
      .map(|item| MemorySearchResultItem { title: item.title, content: item.body, score: item.score as f64 })
      .collect();

    Ok(MemorySearchResult { memories })
  }

  fn resolve_path(&self, path: &str) -> MemoryResult<PathBuf> {
    validate_relative_markdown_path(path)?;
    Ok(self.root.join(path))
  }

  async fn sync_qmd(&self) -> MemoryResult<()> {
    qmd::ensure_collection(&self.collection_name, &self.root).await?;
    qmd::sync_collection(&self.collection_name).await?;
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub(crate) enum MemoryScope {
  Employee(EmployeeId),
  Project(ProjectId),
}

impl MemoryScope {
  async fn ensure_exists(&self) -> MemoryResult<()> {
    match self {
      MemoryScope::Employee(_) => Ok(()),
      MemoryScope::Project(project_id) => ensure_project_exists(project_id).await,
    }
  }

  fn root(&self) -> MemoryResult<PathBuf> {
    match self {
      MemoryScope::Employee(employee_id) => employee_memory_root(employee_id),
      MemoryScope::Project(project_id) => project_memory_root(project_id),
    }
  }

  fn collection_name(&self) -> String {
    match self {
      MemoryScope::Employee(employee_id) => qmd::employee_collection_name(employee_id),
      MemoryScope::Project(project_id) => qmd::project_collection_name(project_id),
    }
  }

  fn default_create_path(&self, date: &str) -> String {
    match self {
      MemoryScope::Employee(_) => format!("{EMPLOYEE_NOTES_DIRECTORY}/{date}.md"),
      // Project create targets the top-level summary file to match the approved project memory layout.
      MemoryScope::Project(_) => PROJECT_SUMMARY_FILE.to_string(),
    }
  }

  fn scope_root_alias(&self) -> &'static str {
    match self {
      MemoryScope::Employee(_) => AGENT_HOME_ALIAS,
      MemoryScope::Project(_) => PROJECT_HOME_ALIAS,
    }
  }
}

pub fn employee_memory_root(employee_id: &EmployeeId) -> MemoryResult<PathBuf> {
  Ok(memory_scope_root(EMPLOYEES_DIRECTORY)?.join(employee_id.uuid().to_string()))
}

pub fn project_memory_root(project_id: &ProjectId) -> MemoryResult<PathBuf> {
  Ok(memory_scope_root(PROJECTS_DIRECTORY)?.join(project_id.uuid().to_string()))
}

async fn ensure_project_exists(project_id: &ProjectId) -> MemoryResult<()> {
  match ProjectRepository::get(project_id.clone()).await {
    Ok(_) => Ok(()),
    Err(DatabaseError::NotFound { .. }) => Err(MemoryError::ProjectNotFound(project_id.uuid().to_string()).into()),
    Err(error) => Err(MemoryError::ProjectLookupFailed(error.to_string()).into()),
  }
}

fn validate_relative_markdown_path(path: &str) -> MemoryResult<()> {
  let candidate = PathBuf::from(path);
  if candidate.as_os_str().is_empty() {
    return Err(MemoryError::InvalidPath("memory path must not be empty".to_string()).into());
  }
  if candidate.is_absolute()
    || candidate
      .components()
      .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
  {
    return Err(MemoryError::InvalidPath(path.to_string()).into());
  }
  if !is_markdown_file(&candidate) {
    return Err(MemoryError::InvalidPath(path.to_string()).into());
  }

  Ok(())
}

fn memory_scope_root(scope_directory: &str) -> MemoryResult<PathBuf> {
  let base_dir = match std::env::var_os(MEMORY_BASE_DIR_ENV) {
    Some(path) => PathBuf::from(path),
    None => std::env::current_dir()?,
  };

  Ok(base_dir.join(MEMORY_DIR).join(scope_directory))
}

fn list_memory_tree(root: &Path, relative_path: &Path) -> MemoryResult<Vec<MemoryTreeNode>> {
  let directory = root.join(relative_path);
  if !directory.exists() {
    return Ok(Vec::new());
  }

  let mut nodes = Vec::new();

  for entry in fs::read_dir(&directory)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let name = entry.file_name().to_string_lossy().into_owned();
    let child_relative_path = relative_path.join(&name);
    let child_path = child_relative_path.to_string_lossy().into_owned();

    if file_type.is_dir() {
      let children = list_memory_tree(root, &child_relative_path)?;
      if !children.is_empty() {
        nodes.push(MemoryTreeNode::Directory { name, path: child_path, children });
      }
      continue;
    }

    if file_type.is_file() && is_markdown_file(entry.path().as_path()) {
      nodes.push(MemoryTreeNode::File { name, path: child_path });
    }
  }

  nodes.sort_by(compare_memory_tree_nodes);
  Ok(nodes)
}

fn is_markdown_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

fn compare_memory_tree_nodes(left: &MemoryTreeNode, right: &MemoryTreeNode) -> Ordering {
  memory_tree_node_kind_rank(left)
    .cmp(&memory_tree_node_kind_rank(right))
    .then_with(|| compare_memory_tree_node_name_desc(memory_tree_node_name(left), memory_tree_node_name(right)))
}

fn memory_tree_node_kind_rank(node: &MemoryTreeNode) -> u8 {
  match node {
    MemoryTreeNode::Directory { .. } => 0,
    MemoryTreeNode::File { .. } => 1,
  }
}

fn compare_memory_tree_node_name_desc(left: &str, right: &str) -> Ordering {
  compare_date_file_names_desc(left, right)
    .then_with(|| compare_numeric_segment_desc(left, right))
    .then_with(|| right.to_ascii_lowercase().cmp(&left.to_ascii_lowercase()))
    .then_with(|| right.cmp(left))
}

fn compare_numeric_segment_desc(left: &str, right: &str) -> Ordering {
  match (parse_numeric_name(left), parse_numeric_name(right)) {
    (Some(left), Some(right)) => right.cmp(&left),
    _ => Ordering::Equal,
  }
}

fn parse_numeric_name(name: &str) -> Option<u32> {
  let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
  if stem.bytes().all(|byte| byte.is_ascii_digit()) { stem.parse().ok() } else { None }
}

fn compare_date_file_names_desc(left: &str, right: &str) -> Ordering {
  match (parse_date_file_name(left), parse_date_file_name(right)) {
    (Some(left), Some(right)) => right.cmp(&left),
    _ => Ordering::Equal,
  }
}

fn parse_date_file_name(name: &str) -> Option<(u32, u32, u32)> {
  let stem = name.rsplit_once('.')?.0;
  let mut parts = stem.split('-');
  let year = parts.next()?.parse().ok()?;
  let month = parts.next()?.parse().ok()?;
  let day = parts.next()?.parse().ok()?;
  if parts.next().is_some() {
    return None;
  }
  Some((year, month, day))
}

fn memory_tree_node_name(node: &MemoryTreeNode) -> &str {
  match node {
    MemoryTreeNode::Directory { name, .. } | MemoryTreeNode::File { name, .. } => name,
  }
}
