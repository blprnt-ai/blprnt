use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::paths::BlprntPath;
use crate::tools::SkillItem;

pub struct SkillsUtils;

const SKILLS_CACHE_TTL: Duration = Duration::from_secs(3);
static SKILLS_CACHE: OnceLock<RwLock<Option<SkillsCacheEntry>>> = OnceLock::new();

#[derive(Clone)]
struct SkillsCacheEntry {
  cached_at: Instant,
  items:     Vec<SkillItem>,
}

impl SkillsUtils {
  pub fn list_skills() -> Result<Vec<SkillItem>> {
    if let Some(cached) = read_cached_skills() {
      return Ok(cached);
    }

    let refreshed = discover_skills();
    write_cached_skills(refreshed.clone());

    Ok(refreshed)
  }

  pub fn get_skill_content(skill_name: &str) -> Option<String> {
    let skill_dir = resolve_skill_dir(skill_name)?;
    Self::load_skill_from(skill_dir.join("SKILL.md"))
  }

  pub fn get_skill_references(skill_name: &str, reference_path: &str) -> Result<String> {
    let skill_dir = resolve_skill_dir(skill_name).context("Skill not found")?;
    let skill_dir_canonical = std::fs::canonicalize(&skill_dir).context("Failed to resolve skill directory")?;
    let reference_path = Path::new(reference_path);

    validate_relative_skill_path(reference_path, "Invalid reference path")?;

    let target_path = std::fs::canonicalize(skill_dir_canonical.join(reference_path))
      .context("Reference path not found or inaccessible")?;

    if !target_path.starts_with(&skill_dir_canonical) {
      anyhow::bail!("Reference path escapes skill directory")
    }

    std::fs::read_to_string(&target_path).context("Failed to read reference file")
  }

  pub fn get_skill_script_path(skill_name: &str, script_name: &str) -> Result<PathBuf> {
    let skill_dir = resolve_skill_dir(skill_name).context("Skill not found")?;
    let skill_dir_canonical = std::fs::canonicalize(&skill_dir).context("Failed to resolve skill directory")?;
    let scripts_dir =
      std::fs::canonicalize(skill_dir_canonical.join("scripts")).context("Skill scripts directory not found")?;
    let script_path = Path::new(script_name);

    validate_relative_skill_path(script_path, "Invalid script name")?;

    let target_path =
      std::fs::canonicalize(scripts_dir.join(script_path)).context("Script not found or inaccessible")?;

    if !target_path.starts_with(&scripts_dir) {
      anyhow::bail!("Script path escapes skill scripts directory")
    }

    Ok(target_path)
  }

  pub fn load_skill_from(skill_path: PathBuf) -> Option<String> {
    parse_skill_content(skill_path)
  }

  pub fn pretty_skill_name(skill_id: &str) -> String {
    let parts = skill_id
      .split('-')
      .filter(|p| !p.is_empty())
      .map(|part| {
        let mut chars = part.chars();
        let Some(first) = chars.next() else {
          return String::new();
        };

        let first = first.to_uppercase().collect::<String>();
        let rest = chars.as_str();
        format!("{first}{rest}")
      })
      .filter(|s| !s.is_empty())
      .collect::<Vec<_>>();

    if parts.is_empty() { skill_id.to_string() } else { parts.join(" ") }
  }
}

struct SkillsBasePaths {
  prebuilt: PathBuf,
  user:     PathBuf,
}

fn resolve_skills_base_dirs() -> SkillsBasePaths {
  let prebuilt = BlprntPath::app_resources().join("skills");
  let user = BlprntPath::blprnt_home().join("user-skills");
  SkillsBasePaths { prebuilt, user }
}

fn resolve_skill_dir(skill_name: &str) -> Option<PathBuf> {
  let base_dirs = resolve_skills_base_dirs();
  let user_path = base_dirs.user.join(skill_name);
  if user_path.is_dir() {
    return Some(user_path);
  }

  let prebuilt_path = base_dirs.prebuilt.join(skill_name);
  if prebuilt_path.is_dir() {
    return Some(prebuilt_path);
  }

  None
}

fn validate_relative_skill_path(path: &Path, error_message: &str) -> Result<()> {
  if path
    .components()
    .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
  {
    anyhow::bail!(error_message.to_string())
  }

  Ok(())
}

fn cache_store() -> &'static RwLock<Option<SkillsCacheEntry>> {
  SKILLS_CACHE.get_or_init(|| RwLock::new(None))
}

fn read_cached_skills() -> Option<Vec<SkillItem>> {
  let cache_guard = cache_store().read().ok()?;
  let entry = cache_guard.as_ref()?;

  (entry.cached_at.elapsed() < SKILLS_CACHE_TTL).then(|| entry.items.clone())
}

fn write_cached_skills(items: Vec<SkillItem>) {
  if let Ok(mut cache_guard) = cache_store().write() {
    *cache_guard = Some(SkillsCacheEntry { cached_at: Instant::now(), items });
  }
}

fn discover_skills() -> Vec<SkillItem> {
  let mut skills_by_id = BTreeMap::new();
  let skills_base_paths = resolve_skills_base_dirs();

  for dir in read_skill_dirs(skills_base_paths.prebuilt) {
    if let Some(skill) = parse_skill_metadata_from_dir(dir) {
      skills_by_id.insert(skill.id.clone(), skill);
    }
  }

  for dir in read_skill_dirs(skills_base_paths.user) {
    if let Some(skill) = parse_skill_metadata_from_dir(dir) {
      skills_by_id.insert(skill.id.clone(), skill);
    }
  }

  skills_by_id.into_values().collect()
}

fn read_skill_dirs(skills_path: PathBuf) -> Vec<PathBuf> {
  let entries = match std::fs::read_dir(skills_path) {
    Ok(entries) => entries,
    Err(_) => return Vec::new(),
  };

  let entries = entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect::<Vec<_>>();

  entries.into_iter().filter(|path| path.is_dir()).collect()
}

fn parse_skill_metadata_from_dir(dir: PathBuf) -> Option<SkillItem> {
  let skill_id = dir.file_name()?.to_str()?.to_string();
  let skill_path = dir.join("SKILL.md");
  if skill_path.exists() { parse_skill_metadata(ParseSkillMetadataParams { skill_id, skill_path }) } else { None }
}

struct ParseSkillMetadataParams {
  skill_id:   String,
  skill_path: PathBuf,
}

fn parse_skill_metadata(params: ParseSkillMetadataParams) -> Option<SkillItem> {
  let skill_content = std::fs::read_to_string(params.skill_path).ok()?;
  let frontmatter = split_skill_document(&skill_content).map(|(yaml_frontmatter, _)| yaml_frontmatter)?;
  let frontmatter = serde_saphyr::from_str::<Value>(frontmatter).ok()?;
  let frontmatter = frontmatter.as_object()?;

  let name = frontmatter_string(frontmatter, "name")?;
  let description = frontmatter_string(frontmatter, "description")?;
  let tags = frontmatter_string_list(frontmatter, "tags");
  let version = frontmatter_string(frontmatter, "version");

  Some(SkillItem { id: params.skill_id, name, description, tags, version })
}

fn parse_skill_content(skill_path: PathBuf) -> Option<String> {
  let skill_content = std::fs::read_to_string(skill_path).ok()?;

  split_skill_document(&skill_content).map(|(_, body)| body.to_string())
}

fn split_skill_document(content: &str) -> Option<(&str, &str)> {
  if !content.starts_with("---") {
    return None;
  }

  let mut sections = content.splitn(3, "---");
  sections.next()?;
  let frontmatter = sections.next()?;
  let body = sections.next()?;

  Some((frontmatter, body))
}

fn frontmatter_string(frontmatter: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
  frontmatter.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn frontmatter_string_list(frontmatter: &serde_json::Map<String, Value>, key: &str) -> Option<Vec<String>> {
  let tags = frontmatter
    .get(key)
    .and_then(Value::as_array)
    .map(|items| {
      items
        .iter()
        .filter_map(|item| item.as_str().map(str::trim).filter(|value| !value.is_empty()).map(str::to_string))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  (!tags.is_empty()).then_some(tags)
}
