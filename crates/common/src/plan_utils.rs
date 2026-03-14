use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use surrealdb::types::ToSql;
use surrealdb::types::Uuid;

use crate::errors::ToolError;
use crate::paths::BlprntPath;
use crate::shared::prelude::SurrealId;
use crate::tools::PlanContentPatch;
use crate::tools::PlanContentPatchHunk;
use crate::tools::PlanDirectory;
use crate::tools::PlanDocumentStatus;
use crate::tools::PlanGetPayload;
use crate::tools::PlanMeta;
use crate::tools::PlanTodoItem;
use crate::tools::PlanWriteContext;

const FRONTMATTER_DELIMITER: &str = "---";
const PLAN_EXTENSION: &str = "plan.md";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PlanFrontmatter {
  pub name:              String,
  pub description:       String,
  #[serde(default)]
  pub todos:             Vec<PlanTodoItem>,
  pub created_at:        String,
  pub updated_at:        String,
  #[serde(default)]
  pub status:            PlanDocumentStatus,
  #[serde(default)]
  pub parent_session_id: Option<String>,
}

impl PlanFrontmatter {
  pub fn into_meta(self) -> PlanMeta {
    PlanMeta {
      name:              self.name,
      description:       self.description,
      todos:             self.todos,
      created_at:        self.created_at,
      updated_at:        self.updated_at,
      status:            self.status,
      parent_session_id: self.parent_session_id,
    }
  }
}

#[derive(Clone, Debug, serde::Deserialize)]
struct PlanFrontmatterCompat {
  name:              String,
  description:       String,
  #[serde(default)]
  todos:             Vec<PlanTodoItem>,
  created_at:        String,
  updated_at:        String,
  #[serde(default)]
  status:            Option<PlanDocumentStatus>,
  #[serde(default)]
  parent_session_id: Option<String>,
  #[serde(default, rename = "cancelled")]
  cancelled:         bool,
}

pub fn get_plan_content(project_id: SurrealId, plan_id: String) -> Result<PlanGetPayload> {
  let plan_directory = resolve_plan_directory(project_id)?;
  let base_path = PathBuf::from(&plan_directory.path);
  let path = base_path.join(&plan_id);

  let content = std::fs::read_to_string(&path)
    .map_err(|e| ToolError::FileReadFailed { path: path.display().to_string(), error: e.to_string() })?;
  let (frontmatter, body) = parse_frontmatter(&content)?;
  let meta = frontmatter.into_meta();

  let payload = PlanGetPayload {
    id:                plan_id.clone(),
    name:              meta.name,
    description:       meta.description,
    content:           body,
    created_at:        meta.created_at,
    updated_at:        meta.updated_at,
    status:            meta.status,
    parent_session_id: meta.parent_session_id,
    todos:             meta.todos,
  };

  Ok(payload)
}

pub fn get_plan_content_by_parent_session_id(
  project_id: SurrealId,
  parent_session_id: &str,
) -> Result<Option<PlanGetPayload>> {
  let plan_directory = resolve_plan_directory(project_id)?;
  let base_path = PathBuf::from(&plan_directory.path);
  ensure_plan_dir(&base_path)?;

  let entries = std::fs::read_dir(&base_path)
    .map_err(|e| ToolError::FileReadFailed { path: base_path.display().to_string(), error: e.to_string() })?;

  for entry in entries {
    let entry = match entry {
      Ok(entry) => entry,
      Err(_) => continue,
    };
    let path = entry.path();
    if !path.is_file() || !is_plan_file(&path) {
      continue;
    }

    let content = match std::fs::read_to_string(&path) {
      Ok(content) => content,
      Err(_) => continue,
    };

    let (frontmatter, body) = match parse_frontmatter(&content) {
      Ok(parsed) => parsed,
      Err(_) => continue,
    };

    if frontmatter.parent_session_id.as_deref() != Some(parent_session_id) {
      continue;
    }

    let meta = frontmatter.into_meta();
    if meta.status == PlanDocumentStatus::Archived {
      continue;
    }

    let plan_id = path.file_name().unwrap_or_default().to_string_lossy().to_string();

    return Ok(Some(PlanGetPayload {
      id:                plan_id,
      name:              meta.name,
      description:       meta.description,
      content:           body,
      created_at:        meta.created_at,
      updated_at:        meta.updated_at,
      status:            meta.status,
      parent_session_id: meta.parent_session_id,
      todos:             meta.todos,
    }));
  }

  Ok(None)
}

pub fn resolve_plan_directory(project_id: SurrealId) -> Result<PlanDirectory> {
  let project_id_value: String = project_id.key().clone().to_sql();
  let plans_path = PathBuf::from(format!("plans/{project_id_value}"));
  let path = BlprntPath::blprnt_home().join(plans_path);
  Ok(PlanDirectory { project_id, path: path.display().to_string() })
}

pub fn ensure_plan_dir(path: &PathBuf) -> Result<()> {
  std::fs::create_dir_all(path)
    .map_err(|e| ToolError::FileWriteFailed { path: path.display().to_string(), error: e.to_string() })?;
  Ok(())
}

pub fn build_plan_id(name: &str) -> String {
  let short_id = Uuid::new_v7().simple().to_string();
  let short_id = short_id.get(0..8).unwrap_or("00000000");
  let slug = slugify_name(name);
  format!("{slug}_{short_id}.{PLAN_EXTENSION}")
}

pub fn is_plan_file(path: &Path) -> bool {
  path.to_string_lossy().ends_with(&format!(".{PLAN_EXTENSION}"))
}

pub fn slugify_name(name: &str) -> String {
  let mut slug = String::new();
  let mut last_dash = false;
  for ch in name.chars() {
    let lowered = ch.to_ascii_lowercase();
    if lowered.is_ascii_alphanumeric() {
      slug.push(lowered);
      last_dash = false;
    } else if !last_dash {
      slug.push('-');
      last_dash = true;
    }
  }
  let slug = slug.trim_matches('-').to_string();
  if slug.is_empty() { "plan".to_string() } else { slug }
}

pub fn parse_frontmatter(content: &str) -> Result<(PlanFrontmatter, String)> {
  let mut sections = content.splitn(3, FRONTMATTER_DELIMITER);
  let first = sections.next().unwrap_or_default();
  if !first.trim().is_empty() {
    return Err(ToolError::General("plan frontmatter missing".to_string()).into());
  }
  let yaml = sections.next().unwrap_or_default();
  let body = sections.next().unwrap_or_default();
  let parsed = serde_saphyr::from_str::<PlanFrontmatterCompat>(yaml)
    .map_err(|e| ToolError::General(format!("invalid plan frontmatter: {e}")))?;
  let frontmatter = PlanFrontmatter {
    name:              parsed.name,
    description:       parsed.description,
    todos:             parsed.todos,
    created_at:        parsed.created_at,
    updated_at:        parsed.updated_at,
    status:            if parsed.cancelled { PlanDocumentStatus::Archived } else { parsed.status.unwrap_or_default() },
    parent_session_id: parsed.parent_session_id,
  };
  validate_frontmatter(&frontmatter)?;
  Ok((frontmatter, body.trim_start().to_string()))
}

fn validate_frontmatter(frontmatter: &PlanFrontmatter) -> Result<()> {
  if let Some(parent_session_id) = &frontmatter.parent_session_id
    && parent_session_id.trim().is_empty()
  {
    return Err(ToolError::General("invalid plan frontmatter: parent_session_id cannot be empty".to_string()).into());
  }

  Ok(())
}

pub fn render_plan_content(frontmatter: &PlanFrontmatter, body: &str) -> Result<String> {
  let yaml = serde_saphyr::to_string(frontmatter)
    .map_err(|e| ToolError::General(format!("failed to encode plan frontmatter: {e}")))?;
  Ok(format!("{FRONTMATTER_DELIMITER}\n{yaml}{FRONTMATTER_DELIMITER}\n\n{body}"))
}

pub fn apply_plan_content_patch(body: &str, patch: &PlanContentPatch) -> Result<String> {
  let mut lines = body_to_lines(body);

  for (index, hunk) in patch.hunks.iter().enumerate() {
    let Some(match_start) = find_unique_hunk_match(&lines, hunk, index)? else {
      return Err(ToolError::General(format!("plan content patch hunk {} matched zero locations", index)).into());
    };

    let replace_end = match_start + hunk.delete.len();
    lines.splice(match_start..replace_end, hunk.insert.iter().cloned());
  }

  Ok(lines.join("\n"))
}

fn body_to_lines(body: &str) -> Vec<String> {
  if body.is_empty() { Vec::new() } else { body.split('\n').map(ToString::to_string).collect() }
}

fn find_unique_hunk_match(lines: &[String], hunk: &PlanContentPatchHunk, hunk_index: usize) -> Result<Option<usize>> {
  if hunk.before.is_empty() && hunk.delete.is_empty() && hunk.insert.is_empty() && hunk.after.is_empty() {
    return Err(ToolError::General(format!("plan content patch hunk {} is empty", hunk_index)).into());
  }

  if hunk.delete.len() > lines.len() {
    return Ok(None);
  }

  let mut matched_start = None;
  let max_start = lines.len() - hunk.delete.len();

  for start in 0..=max_start {
    if !hunk_matches_at(lines, hunk, start) {
      continue;
    }

    if matched_start.is_some() {
      return Err(
        ToolError::General(format!("plan content patch hunk {} matched multiple locations", hunk_index)).into(),
      );
    }

    matched_start = Some(start);
  }

  Ok(matched_start)
}

fn hunk_matches_at(lines: &[String], hunk: &PlanContentPatchHunk, start: usize) -> bool {
  let before_len = hunk.before.len();
  let delete_len = hunk.delete.len();
  let after_len = hunk.after.len();

  if start < before_len {
    return false;
  }

  let delete_end = start + delete_len;
  if delete_end > lines.len() {
    return false;
  }

  let after_end = delete_end + after_len;
  if after_end > lines.len() {
    return false;
  }

  lines[start - before_len..start] == hunk.before
    && lines[start..delete_end] == hunk.delete
    && lines[delete_end..after_end] == hunk.after
}

pub fn build_write_context(
  plan_id: String,
  base_path: &Path,
  created_at: String,
  updated_at: String,
) -> PlanWriteContext {
  let plan_path = base_path.join(&plan_id);
  PlanWriteContext { plan_id, plan_path: plan_path.display().to_string(), created_at, updated_at }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  fn patch(hunks: Vec<PlanContentPatchHunk>) -> PlanContentPatch {
    PlanContentPatch { hunks }
  }

  #[test]
  fn test_get_plan_content_by_parent_session_id() {
    let project_id = Uuid::from_str("019c905a-87c6-7960-a819-26a3d5b0df18").unwrap();
    let project_id = SurrealId::from((String::from("projects"), project_id));
    let session_id = "sessions:u'019c9f69-f647-7062-adfb-0409df56937b'";

    let plan = get_plan_content_by_parent_session_id(project_id, session_id).unwrap();

    println!("plan: {:#?}", plan);
  }

  #[test]
  fn apply_plan_content_patch_replaces_exact_body_lines() {
    let body = "alpha\nbeta\ngamma";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec!["alpha".to_string()],
      delete: vec!["beta".to_string()],
      insert: vec!["delta".to_string()],
      after:  vec!["gamma".to_string()],
    }]);

    let patched = apply_plan_content_patch(body, &patch).unwrap();

    assert_eq!(patched, "alpha\ndelta\ngamma");
  }

  #[test]
  fn apply_plan_content_patch_replaces_multi_line_blocks() {
    let body = "intro\nold-a\nold-b\noutro";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec!["intro".to_string()],
      delete: vec!["old-a".to_string(), "old-b".to_string()],
      insert: vec!["new-a".to_string(), "new-b".to_string()],
      after:  vec!["outro".to_string()],
    }]);

    let patched = apply_plan_content_patch(body, &patch).unwrap();

    assert_eq!(patched, "intro\nnew-a\nnew-b\noutro");
  }

  #[test]
  fn apply_plan_content_patch_supports_insert_only_hunks() {
    let body = "alpha\ngamma";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec!["alpha".to_string()],
      delete: vec![],
      insert: vec!["beta".to_string()],
      after:  vec!["gamma".to_string()],
    }]);

    let patched = apply_plan_content_patch(body, &patch).unwrap();

    assert_eq!(patched, "alpha\nbeta\ngamma");
  }

  #[test]
  fn apply_plan_content_patch_supports_delete_only_hunks() {
    let body = "alpha\nbeta\ngamma";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec!["alpha".to_string()],
      delete: vec!["beta".to_string()],
      insert: vec![],
      after:  vec!["gamma".to_string()],
    }]);

    let patched = apply_plan_content_patch(body, &patch).unwrap();

    assert_eq!(patched, "alpha\ngamma");
  }

  #[test]
  fn apply_plan_content_patch_fails_when_hunk_matches_zero_locations() {
    let body = "alpha\nbeta\ngamma";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec!["alpha".to_string()],
      delete: vec!["delta".to_string()],
      insert: vec!["epsilon".to_string()],
      after:  vec!["gamma".to_string()],
    }]);

    let error = apply_plan_content_patch(body, &patch).unwrap_err().to_string();

    assert!(error.contains("matched zero locations"));
  }

  #[test]
  fn apply_plan_content_patch_fails_when_hunk_matches_multiple_locations() {
    let body = "alpha\nbeta\nalpha\nbeta";
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec![],
      delete: vec!["alpha".to_string(), "beta".to_string()],
      insert: vec!["delta".to_string()],
      after:  vec![],
    }]);

    let error = apply_plan_content_patch(body, &patch).unwrap_err().to_string();

    assert!(error.contains("matched multiple locations"));
  }

  #[test]
  fn apply_plan_content_patch_is_all_or_nothing() {
    let body = "alpha\nbeta\ngamma";
    let patch = patch(vec![
      PlanContentPatchHunk {
        before: vec!["alpha".to_string()],
        delete: vec!["beta".to_string()],
        insert: vec!["delta".to_string()],
        after:  vec!["gamma".to_string()],
      },
      PlanContentPatchHunk {
        before: vec!["missing".to_string()],
        delete: vec!["gamma".to_string()],
        insert: vec!["epsilon".to_string()],
        after:  vec![],
      },
    ]);

    let error = apply_plan_content_patch(body, &patch).unwrap_err().to_string();
    let original_lines = body_to_lines(body);

    assert!(error.contains("matched zero locations"));
    assert_eq!(original_lines.join("\n"), body);
  }

  #[test]
  fn apply_plan_content_patch_rejects_empty_hunks() {
    let body = "alpha";
    let patch = patch(vec![PlanContentPatchHunk::default()]);

    let error = apply_plan_content_patch(body, &patch).unwrap_err().to_string();

    assert!(error.contains("is empty"));
  }

  #[test]
  fn parse_frontmatter_and_patching_body_prevents_frontmatter_edits() {
    let document = render_plan_content(
      &PlanFrontmatter {
        name:              "Original Name".to_string(),
        description:       "Original Description".to_string(),
        todos:             vec![],
        created_at:        "2025-01-01T00:00:00Z".to_string(),
        updated_at:        "2025-01-01T00:00:00Z".to_string(),
        status:            PlanDocumentStatus::Pending,
        parent_session_id: None,
      },
      "body line",
    )
    .unwrap();

    let (frontmatter, body) = parse_frontmatter(&document).unwrap();
    let patch = patch(vec![PlanContentPatchHunk {
      before: vec![],
      delete: vec!["name: Original Name".to_string()],
      insert: vec!["name: Patched Name".to_string()],
      after:  vec![],
    }]);

    let error = apply_plan_content_patch(&body, &patch).unwrap_err().to_string();
    let reparsed = render_plan_content(&frontmatter, &body).unwrap();

    assert!(error.contains("matched zero locations"));
    assert!(reparsed.contains("name: Original Name"));
    assert!(!reparsed.contains("name: Patched Name"));
  }
}
