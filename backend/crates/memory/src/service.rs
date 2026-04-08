use std::cmp::Ordering;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use persistence::prelude::DbId;
use persistence::prelude::EmployeeId;
use persistence::prelude::ProjectId;
use persistence::prelude::ProjectRepository;
use shared::errors::DatabaseError;
use shared::errors::MemoryError;
use shared::errors::MemoryResult;

use crate::MemoryListResult;
use crate::ProjectPlanListItem;
use crate::ProjectPlanReadResult;
use crate::ProjectPlansListResult;
use crate::MemoryReadResult;
use crate::MemorySearchResult;
use crate::MemorySearchResultItem;
use crate::MemoryTreeNode;
use crate::qmd;

const MEMORY_DIRECTORY: &str = "memory";
const LIFE_DIRECTORY: &str = "life";
const PLANS_DIRECTORY: &str = "plans";
#[derive(Clone, Debug)]
pub struct ProjectMemoryService {
  inner: ScopedMemoryService,
}

#[derive(Clone, Debug)]
pub struct EmployeeMemoryService {
  inner: ScopedMemoryService,
}

#[derive(Clone, Debug)]
pub struct ProjectPlansService {
  root: PathBuf,
}

impl ProjectMemoryService {
  pub async fn new(project_id: ProjectId) -> MemoryResult<Self> {
    Ok(Self { inner: ScopedMemoryService::new(MemoryScope::Project(project_id)).await? })
  }

  pub async fn list(&self) -> MemoryResult<MemoryListResult> {
    self.inner.list().await
  }

  pub async fn read(&self, path: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.read(path).await
  }

  pub async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.inner.search(query, limit).await
  }
}

impl EmployeeMemoryService {
  pub async fn new(employee_id: EmployeeId) -> MemoryResult<Self> {
    Ok(Self { inner: ScopedMemoryService::new(MemoryScope::Employee(employee_id)).await? })
  }

  pub async fn list(&self) -> MemoryResult<MemoryListResult> {
    self.inner.list().await
  }

  pub async fn read(&self, path: &str) -> MemoryResult<MemoryReadResult> {
    self.inner.read(path).await
  }

  pub async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.inner.search(query, limit).await
  }
}

impl ProjectPlansService {
  pub async fn new(project_id: ProjectId) -> MemoryResult<Self> {
    ensure_project_exists(&project_id).await?;
    let root = project_plans_root(&project_id)?;
    fs::create_dir_all(&root)?;
    Ok(Self { root })
  }

  pub async fn list(&self) -> MemoryResult<ProjectPlansListResult> {
    let mut plans = Vec::new();
    list_project_plan_items(&self.root, Path::new(""), &mut plans)?;
    plans.sort_by(compare_project_plan_items);
    Ok(ProjectPlansListResult { plans })
  }

  pub async fn read(&self, path: &str) -> MemoryResult<ProjectPlanReadResult> {
    let absolute_path = resolve_relative_path_within_root(&self.root, path)?;
    let bytes = fs::read(&absolute_path)?;
    let mime_type = infer_plan_mime_type(&absolute_path, &bytes);
    let is_previewable = is_previewable_plan_mime_type(&mime_type);
    let content = if is_previewable { Some(String::from_utf8_lossy(&bytes).into_owned()) } else { None };

    Ok(ProjectPlanReadResult { path: path.to_string(), mime_type, is_previewable, content })
  }
}

#[derive(Clone, Debug)]
struct ScopedMemoryService {
  root:             PathBuf,
  collection_names: Vec<String>,
  qmd_roots:        Vec<PathBuf>,
  scope:            MemoryScope,
}

impl ScopedMemoryService {
  async fn new(scope: MemoryScope) -> MemoryResult<Self> {
    scope.ensure_exists().await?;

    let root = scope.root()?;
    fs::create_dir_all(&root)?;
    let qmd_roots = scope.qmd_roots()?;
    for qmd_root in &qmd_roots {
      fs::create_dir_all(qmd_root)?;
    }

    let collection_names = scope.collection_names();
    for (collection_name, qmd_root) in collection_names.iter().zip(qmd_roots.iter()) {
      qmd::ensure_collection(collection_name, qmd_root).await?;
    }

    Ok(Self { root, collection_names, qmd_roots, scope })
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

  async fn search(&self, query: &str, limit: Option<usize>) -> MemoryResult<MemorySearchResult> {
    self.sync_qmd().await?;

    let mut memories = Vec::new();
    for collection_name in &self.collection_names {
      memories.extend(
        qmd::search_collection(collection_name, query, limit).await?.into_iter().map(|item| MemorySearchResultItem {
          path:    relative_search_result_path(&item.file, &self.root),
          title:   item.title,
          content: item.body,
          score:   item.score as f64,
        }),
      );
    }

    memories.sort_by(|left, right| {
      right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| right.title.cmp(&left.title))
        .then_with(|| right.content.cmp(&left.content))
    });
    if let Some(limit) = limit {
      memories.truncate(limit);
    }

    Ok(MemorySearchResult { memories })
  }

  fn resolve_path(&self, path: &str) -> MemoryResult<PathBuf> {
    validate_relative_markdown_path(path)?;
    Ok(self.root.join(path))
  }

  async fn sync_qmd(&self) -> MemoryResult<()> {
    for (collection_name, qmd_root) in self.collection_names.iter().zip(self.qmd_roots.iter()) {
      qmd::ensure_collection(collection_name, qmd_root).await?;
      qmd::sync_collection(collection_name).await?;
    }
    Ok(())
  }
}

fn relative_search_result_path(file_path: &str, root: &Path) -> Option<String> {
  if let Some(virtual_path) = ::qmd::parse_virtual_path(file_path) {
    return Some(virtual_path.path);
  }

  let path = Path::new(file_path);
  path.strip_prefix(root).ok().map(|relative| relative.to_string_lossy().replace('\\', "/"))
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

  fn qmd_roots(&self) -> MemoryResult<Vec<PathBuf>> {
    match self {
      MemoryScope::Employee(employee_id) => {
        Ok(vec![employee_memory_root(employee_id)?, employee_life_root(employee_id)?])
      }
      MemoryScope::Project(project_id) => Ok(vec![project_memory_root(project_id)?]),
    }
  }

  fn collection_names(&self) -> Vec<String> {
    match self {
      MemoryScope::Employee(employee_id) => {
        vec![qmd::employee_memory_collection_name(employee_id), qmd::employee_life_collection_name(employee_id)]
      }
      MemoryScope::Project(project_id) => vec![qmd::project_collection_name(project_id)],
    }
  }

  fn scope_root_alias(&self) -> &'static str {
    match self {
      MemoryScope::Employee(_) => "$AGENT_HOME/memory",
      MemoryScope::Project(_) => "$PROJECT_HOME/memory",
    }
  }
}

pub fn employee_memory_root(employee_id: &EmployeeId) -> MemoryResult<PathBuf> {
  Ok(employee_scope_root(employee_id)?.join(MEMORY_DIRECTORY))
}

pub fn employee_life_root(employee_id: &EmployeeId) -> MemoryResult<PathBuf> {
  Ok(employee_scope_root(employee_id)?.join(LIFE_DIRECTORY))
}

pub fn project_memory_root(project_id: &ProjectId) -> MemoryResult<PathBuf> {
  Ok(project_scope_root(project_id)?.join(MEMORY_DIRECTORY))
}

pub fn project_plans_root(project_id: &ProjectId) -> MemoryResult<PathBuf> {
  Ok(project_scope_root(project_id)?.join(PLANS_DIRECTORY))
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

fn validate_relative_path(path: &str) -> MemoryResult<()> {
  let candidate = PathBuf::from(path);
  if candidate.as_os_str().is_empty() {
    return Err(MemoryError::InvalidPath("plan path must not be empty".to_string()).into());
  }
  if candidate.is_absolute()
    || candidate
      .components()
      .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
  {
    return Err(MemoryError::InvalidPath(path.to_string()).into());
  }

  Ok(())
}

fn resolve_relative_path_within_root(root: &Path, path: &str) -> MemoryResult<PathBuf> {
  validate_relative_path(path)?;
  Ok(root.join(path))
}

fn employee_scope_root(employee_id: &EmployeeId) -> MemoryResult<PathBuf> {
  Ok(shared::paths::employee_home(&employee_id.uuid().to_string()))
}

fn project_scope_root(project_id: &ProjectId) -> MemoryResult<PathBuf> {
  Ok(shared::paths::project_home(&project_id.uuid().to_string()))
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

fn list_project_plan_items(root: &Path, relative_path: &Path, plans: &mut Vec<ProjectPlanListItem>) -> MemoryResult<()> {
  let directory = root.join(relative_path);
  if !directory.exists() {
    return Ok(());
  }

  for entry in fs::read_dir(&directory)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    let name = entry.file_name().to_string_lossy().into_owned();
    let child_relative_path = relative_path.join(&name);

    if file_type.is_dir() {
      list_project_plan_items(root, &child_relative_path, plans)?;
      continue;
    }

    if !file_type.is_file() {
      continue;
    }

    let absolute_path = entry.path();
    let relative_path = child_relative_path.to_string_lossy().replace('\\', "/");
    let metadata = fs::metadata(&absolute_path)?;
    let updated_at = metadata.modified().map(DateTime::<Utc>::from).unwrap_or_else(|_| Utc::now()).to_rfc3339();
    let filename = absolute_path
      .file_name()
      .and_then(|value| value.to_str())
      .map(str::to_string)
      .unwrap_or_else(|| relative_path.clone());

    let (title, is_superseded) = match fs::read_to_string(&absolute_path) {
      Ok(content) => derive_plan_metadata(&content, &filename),
      Err(_) => (filename.clone(), false),
    };

    plans.push(ProjectPlanListItem { path: relative_path, title, filename, updated_at, is_superseded });
  }

  Ok(())
}

fn compare_project_plan_items(left: &ProjectPlanListItem, right: &ProjectPlanListItem) -> Ordering {
  left
    .is_superseded
    .cmp(&right.is_superseded)
    .then_with(|| right.updated_at.cmp(&left.updated_at))
    .then_with(|| left.path.cmp(&right.path))
}

fn derive_plan_metadata(content: &str, filename: &str) -> (String, bool) {
  let (frontmatter, body) = split_frontmatter(content);
  let title = frontmatter
    .as_ref()
    .and_then(extract_frontmatter_title)
    .or_else(|| extract_markdown_title(body))
    .unwrap_or_else(|| filename.to_string());
  let is_superseded = frontmatter.as_ref().is_some_and(frontmatter_marks_superseded) || body_marks_superseded(body);

  (title, is_superseded)
}

fn split_frontmatter(content: &str) -> (Option<serde_yaml::Value>, &str) {
  if !(content.starts_with("---\n") || content.starts_with("---\r\n")) {
    return (None, content);
  }

  let delimiter_len = if content.starts_with("---\r\n") { 5 } else { 4 };
  let remainder = &content[delimiter_len..];
  if let Some(index) = remainder.find("\n---\n") {
    let yaml = &remainder[..index + 1];
    let body = &remainder[index + 5..];
    return (serde_yaml::from_str::<serde_yaml::Value>(yaml).ok(), body);
  }
  if let Some(index) = remainder.find("\n---\r\n") {
    let yaml = &remainder[..index + 1];
    let body = &remainder[index + 6..];
    return (serde_yaml::from_str::<serde_yaml::Value>(yaml).ok(), body);
  }

  (None, content)
}

fn extract_frontmatter_title(frontmatter: &serde_yaml::Value) -> Option<String> {
  frontmatter.get("title")?.as_str().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn frontmatter_marks_superseded(frontmatter: &serde_yaml::Value) -> bool {
  frontmatter.get("superseded").and_then(serde_yaml::Value::as_bool).unwrap_or(false)
    || frontmatter
      .get("status")
      .and_then(serde_yaml::Value::as_str)
      .is_some_and(|value| value.trim().eq_ignore_ascii_case("superseded"))
}

fn extract_markdown_title(content: &str) -> Option<String> {
  content
    .lines()
    .map(str::trim)
    .find(|line| line.starts_with("# "))
    .map(|line| line.trim_start_matches("# ").trim().to_string())
    .filter(|line| !line.is_empty())
}

fn body_marks_superseded(content: &str) -> bool {
  content.lines().map(str::trim).filter(|line| !line.is_empty()).take(12).any(|line| {
    let normalized = line.to_ascii_lowercase();
    (normalized.starts_with("**status:**") || normalized.starts_with("status:")) && normalized.contains("superseded")
  })
}

fn infer_plan_mime_type(path: &Path, bytes: &[u8]) -> String {
  let extension = path.extension().and_then(|extension| extension.to_str()).unwrap_or_default();
  if extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown") {
    return "text/markdown".to_string();
  }
  if std::str::from_utf8(bytes).is_ok() {
    return "text/plain".to_string();
  }

  "application/octet-stream".to_string()
}

fn is_previewable_plan_mime_type(mime_type: &str) -> bool {
  matches!(mime_type, "text/markdown" | "text/plain")
}
