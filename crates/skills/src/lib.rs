use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use shared::paths;

const BUILTIN_SKILL_FILES: &[(&str, &str)] = &[
  ("blprnt/SKILL.md", include_str!("../../../skills/blprnt/SKILL.md")),
  ("blprnt/references/api-reference.md", include_str!("../../../skills/blprnt/references/api-reference.md")),
  ("blprnt/references/runtime-workflows.md", include_str!("../../../skills/blprnt/references/runtime-workflows.md")),
  ("hire-employee/SKILL.md", include_str!("../../../skills/hire-employee/SKILL.md")),
  (
    "hire-employee/references/api-references.md",
    include_str!("../../../skills/hire-employee/references/api-references.md"),
  ),
  ("para-memory-files/SKILL.md", include_str!("../../../skills/para-memory-files/SKILL.md")),
  ("para-memory-files/references/schemas.md", include_str!("../../../skills/para-memory-files/references/schemas.md")),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillSource {
  User,
  Builtin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillMetadata {
  pub name:        String,
  pub description: String,
  pub path:        PathBuf,
  pub source:      SkillSource,
}

pub fn ensure_builtin_skills_installed() -> Result<()> {
  let cache_root = paths::blprnt_builtin_skills_dir();
  let mirror_root = paths::blprnt_builtin_skills_mirror_dir();
  fs::create_dir_all(&cache_root).with_context(|| format!("failed to create {}", cache_root.display()))?;
  fs::create_dir_all(&mirror_root).with_context(|| format!("failed to create {}", mirror_root.display()))?;

  for (relative_path, contents) in BUILTIN_SKILL_FILES {
    let target = cache_root.join(relative_path);
    if let Some(parent) = target.parent() {
      fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&target, contents).with_context(|| format!("failed to write {}", target.display()))?;
  }

  for skill_name in builtin_skill_names() {
    ensure_mirror_link(&cache_root.join(skill_name), &mirror_root.join(skill_name))?;
  }

  Ok(())
}

pub fn list_skills() -> Result<Vec<SkillMetadata>> {
  ensure_builtin_skills_installed()?;

  let mut skills = Vec::new();
  scan_skill_root(&paths::agents_skills_dir(), SkillSource::User, &mut skills)?;
  scan_skill_root(&paths::blprnt_builtin_skills_dir(), SkillSource::Builtin, &mut skills)?;

  let mut seen = HashSet::new();
  skills.retain(|skill| seen.insert(skill.path.clone()));
  skills.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));
  Ok(skills)
}

pub fn validate_skill_path(path: &Path, expected_name: Option<&str>) -> Result<SkillMetadata> {
  ensure_builtin_skills_installed()?;
  let requested_path = path.to_path_buf();
  let canonical = dunce::canonicalize(path).with_context(|| format!("failed to resolve {}", path.display()))?;
  anyhow::ensure!(canonical.is_absolute(), "skill path must be absolute: {}", canonical.display());
  anyhow::ensure!(
    canonical.file_name().and_then(|value| value.to_str()) == Some("SKILL.md"),
    "skill path must point to SKILL.md: {}",
    canonical.display()
  );

  let source = if requested_path.starts_with(paths::blprnt_builtin_skills_dir())
    || requested_path.starts_with(paths::blprnt_builtin_skills_mirror_dir())
    || canonical.starts_with(paths::blprnt_builtin_skills_dir())
    || canonical.starts_with(paths::blprnt_builtin_skills_mirror_dir())
  {
    SkillSource::Builtin
  } else {
    SkillSource::User
  };

  let contents = fs::read_to_string(&canonical).with_context(|| format!("failed to read {}", canonical.display()))?;
  let (name, description) = parse_skill_header(&contents, &canonical)?;
  if let Some(expected_name) = expected_name {
    anyhow::ensure!(
      expected_name == name,
      "skill name mismatch for {}: expected {}, found {}",
      canonical.display(),
      expected_name,
      name
    );
  }

  Ok(SkillMetadata { name, description, path: canonical, source })
}

fn builtin_skill_names() -> Vec<&'static str> {
  let mut names = Vec::new();
  for (relative_path, _) in BUILTIN_SKILL_FILES {
    if let Some((head, tail)) = relative_path.split_once('/')
      && tail == "SKILL.md"
    {
      names.push(head);
    }
  }
  names
}

fn ensure_mirror_link(target_dir: &Path, link_dir: &Path) -> Result<()> {
  if let Some(parent) = link_dir.parent() {
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
  }

  if link_dir.exists() || fs::symlink_metadata(link_dir).is_ok() {
    remove_path(link_dir)?;
  }

  #[cfg(target_os = "windows")]
  std::os::windows::fs::symlink_dir(target_dir, link_dir)
    .with_context(|| format!("failed to link {} -> {}", link_dir.display(), target_dir.display()))?;

  #[cfg(not(target_os = "windows"))]
  std::os::unix::fs::symlink(target_dir, link_dir)
    .with_context(|| format!("failed to link {} -> {}", link_dir.display(), target_dir.display()))?;

  Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
  let meta = fs::symlink_metadata(path).with_context(|| format!("failed to inspect {}", path.display()))?;
  if meta.file_type().is_symlink() || meta.is_file() {
    fs::remove_file(path).with_context(|| format!("failed to remove {}", path.display()))?;
  } else {
    fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
  }
  Ok(())
}

fn scan_skill_root(root: &Path, source: SkillSource, output: &mut Vec<SkillMetadata>) -> Result<()> {
  if !root.is_dir() {
    return Ok(());
  }

  for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
    let entry = entry?;
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    let skill_path = path.join("SKILL.md");
    if !skill_path.is_file() {
      continue;
    }

    let canonical =
      dunce::canonicalize(&skill_path).with_context(|| format!("failed to resolve {}", skill_path.display()))?;
    let contents = fs::read_to_string(&canonical).with_context(|| format!("failed to read {}", canonical.display()))?;
    let (name, description) = parse_skill_header(&contents, &canonical)?;
    output.push(SkillMetadata { name, description, path: canonical, source: source.clone() });
  }

  Ok(())
}

fn parse_skill_header(contents: &str, path: &Path) -> Result<(String, String)> {
  let mut lines = contents.lines();
  anyhow::ensure!(lines.next() == Some("---"), "missing frontmatter in {}", path.display());

  let mut name = None;
  let mut description = None;
  for line in lines {
    if line == "---" {
      break;
    }
    if let Some(value) = line.strip_prefix("name:") {
      name = Some(value.trim().trim_matches('"').to_string());
      continue;
    }
    if let Some(value) = line.strip_prefix("description:") {
      description = Some(value.trim().trim_matches('"').trim_start_matches('>').trim().to_string());
    }
  }

  let name = name.with_context(|| format!("missing skill name in {}", path.display()))?;
  Ok((name, description.unwrap_or_default()))
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;

  static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

  struct HomeGuard {
    previous_home: Option<String>,
  }

  impl HomeGuard {
    fn set(temp_home: &TempDir) -> Self {
      let previous_home = std::env::var("HOME").ok();
      unsafe { std::env::set_var("HOME", temp_home.path()) };
      Self { previous_home }
    }
  }

  impl Drop for HomeGuard {
    fn drop(&mut self) {
      match &self.previous_home {
        Some(home) => unsafe { std::env::set_var("HOME", home) },
        None => unsafe { std::env::remove_var("HOME") },
      }
    }
  }

  #[test]
  fn installs_builtins_into_cache_and_mirror() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let home = TempDir::new().unwrap();
    let _guard = HomeGuard::set(&home);

    ensure_builtin_skills_installed().unwrap();

    let cache_skill = paths::blprnt_builtin_skills_dir().join("blprnt").join("SKILL.md");
    let mirror_skill = paths::blprnt_builtin_skills_mirror_dir().join("blprnt").join("SKILL.md");
    assert!(cache_skill.is_file(), "expected builtin cache skill at {}", cache_skill.display());
    assert!(mirror_skill.exists(), "expected builtin mirror skill at {}", mirror_skill.display());
  }

  #[test]
  fn list_skills_includes_builtin_and_user_skills() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let home = TempDir::new().unwrap();
    let _guard = HomeGuard::set(&home);

    let user_skill_dir = paths::agents_skills_dir().join("user-skill");
    fs::create_dir_all(&user_skill_dir).unwrap();
    fs::write(user_skill_dir.join("SKILL.md"), "---\nname: user-skill\ndescription: user space\n---\n\n# User Skill\n")
      .unwrap();

    let skills = list_skills().unwrap();
    assert!(skills.iter().any(|skill| skill.name == "blprnt"));
    assert!(skills.iter().any(|skill| skill.name == "user-skill"));
  }

  #[test]
  fn validate_skill_path_resolves_canonical_skill_metadata() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let home = TempDir::new().unwrap();
    let _guard = HomeGuard::set(&home);

    ensure_builtin_skills_installed().unwrap();
    let metadata =
      validate_skill_path(&paths::blprnt_builtin_skills_mirror_dir().join("blprnt").join("SKILL.md"), Some("blprnt"))
        .unwrap();

    assert_eq!(metadata.name, "blprnt");
    assert!(metadata.path.is_absolute());
    assert!(matches!(metadata.source, SkillSource::Builtin));
  }
}
