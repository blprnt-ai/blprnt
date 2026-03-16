use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use common::errors::IntoTauriResult;
use common::errors::TauriResult;
use common::memory::ManagedMemoryStore;
use common::memory::MemorySearchRequest;
use common::memory::MemorySearchResult;
use common::memory::MemoryWriteResult;
use common::memory::local_today;
use common::paths::BlprntPath;
use persistence::prelude::SurrealId;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryCreateRequest {
  pub project_id: String,
  pub content:    String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryReadRequest {
  pub project_id: String,
  pub path:       String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryListRequest {
  pub project_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryTreeNode {
  Directory { name: String, path: String, children: Vec<MemoryTreeNode> },
  File { name: String, path: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryListResult {
  pub root_path: String,
  pub nodes:     Vec<MemoryTreeNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryReadResult {
  pub path:    String,
  pub content: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemorySearchCommandRequest {
  pub project_id: String,
  pub query:      String,
  pub limit:      Option<usize>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryUpdateRequest {
  pub project_id: String,
  pub path:       String,
  pub content:    String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct MemoryDeleteRequest {
  pub project_id: String,
  pub path:       String,
}

#[tauri::command]
#[specta::specta]
pub async fn memory_create(request: MemoryCreateRequest) -> TauriResult<MemoryWriteResult> {
  let memory_root = project_memory_root(&request.project_id)?;
  let store = ManagedMemoryStore::new(memory_root);

  store.append_entry_for_date(local_today(), &request.content).map_err(anyhow::Error::from).into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn memory_read(request: MemoryReadRequest) -> TauriResult<MemoryReadResult> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  let content = fs::read_to_string(&path).map_err(anyhow::Error::from).into_tauri()?;

  Ok(MemoryReadResult { path: request.path, content })
}

#[tauri::command]
#[specta::specta]
pub async fn memory_list(request: MemoryListRequest) -> TauriResult<MemoryListResult> {
  let root = project_memory_root(&request.project_id)?;
  let nodes = list_memory_tree(&root, Path::new(""))?;

  Ok(MemoryListResult { root_path: String::new(), nodes })
}

#[tauri::command]
#[specta::specta]
pub async fn memory_search(request: MemorySearchCommandRequest) -> TauriResult<MemorySearchResult> {
  let project_id = SurrealId::try_from(request.project_id).map_err(anyhow::Error::from).into_tauri()?;

  common::memory::QmdMemorySearchService::new(project_id.key().to_string())
    .search(&MemorySearchRequest { query: request.query, limit: request.limit }, None)
    .await
    .map_err(anyhow::Error::from)
    .into_tauri()
}

#[tauri::command]
#[specta::specta]
pub async fn memory_update(request: MemoryUpdateRequest) -> TauriResult<MemoryReadResult> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(anyhow::Error::from).into_tauri()?;
  }
  fs::write(&path, &request.content).map_err(anyhow::Error::from).into_tauri()?;

  Ok(MemoryReadResult { path: request.path, content: request.content })
}

#[tauri::command]
#[specta::specta]
pub async fn memory_delete(request: MemoryDeleteRequest) -> TauriResult<()> {
  let path = resolve_memory_path(&request.project_id, &request.path)?;
  fs::remove_file(path).map_err(anyhow::Error::from).into_tauri()
}

fn project_memory_root(project_id: &str) -> TauriResult<PathBuf> {
  let project_id = SurrealId::try_from(project_id).map_err(anyhow::Error::from).into_tauri()?;
  Ok(BlprntPath::memories_root().join(project_id.key().to_string()))
}

fn list_memory_tree(root: &Path, relative_path: &Path) -> TauriResult<Vec<MemoryTreeNode>> {
  let directory = root.join(relative_path);
  if !directory.exists() {
    return Ok(Vec::new());
  }

  let mut nodes = Vec::new();

  for entry in fs::read_dir(&directory).map_err(anyhow::Error::from).into_tauri()? {
    let entry = entry.map_err(anyhow::Error::from).into_tauri()?;
    let file_type = entry.file_type().map_err(anyhow::Error::from).into_tauri()?;
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

fn compare_memory_tree_nodes(left: &MemoryTreeNode, right: &MemoryTreeNode) -> std::cmp::Ordering {
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

fn compare_memory_tree_node_name_desc(left: &str, right: &str) -> std::cmp::Ordering {
  compare_date_file_names_desc(left, right)
    .then_with(|| compare_numeric_segment_desc(left, right))
    .then_with(|| right.to_ascii_lowercase().cmp(&left.to_ascii_lowercase()))
    .then_with(|| right.cmp(left))
}

fn compare_numeric_segment_desc(left: &str, right: &str) -> std::cmp::Ordering {
  match (parse_numeric_name(left), parse_numeric_name(right)) {
    (Some(left), Some(right)) => right.cmp(&left),
    _ => std::cmp::Ordering::Equal,
  }
}

fn parse_numeric_name(name: &str) -> Option<u32> {
  if name.bytes().all(|byte| byte.is_ascii_digit()) { name.parse().ok() } else { None }
}

fn compare_date_file_names_desc(left: &str, right: &str) -> std::cmp::Ordering {
  match (parse_date_file_name(left), parse_date_file_name(right)) {
    (Some(left), Some(right)) => right.cmp(&left),
    _ => std::cmp::Ordering::Equal,
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

fn resolve_memory_path(project_id: &str, relative_path: &str) -> TauriResult<PathBuf> {
  let candidate = PathBuf::from(relative_path);
  if candidate.as_os_str().is_empty() {
    return Err(anyhow::anyhow!("memory path must not be empty")).into_tauri();
  }
  if candidate.is_absolute()
    || candidate
      .components()
      .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
  {
    return Err(anyhow::anyhow!("memory path must be a relative path within the project memory root")).into_tauri();
  }

  Ok(project_memory_root(&project_id.to_string())?.join(candidate))
}
