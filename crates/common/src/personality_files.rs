use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::paths::BlprntPath;

pub const PERSONALITY_FRONTMATTER_DELIMITER: &str = "---";
pub const PERSONALITY_FILE_NAME: &str = "PERSONALITY.md";
pub const SKILLS_DIR: &str = "skills";
pub const SYSTEM_PERSONALITIES_DIR: &str = "personalities";
pub const USER_PERSONALITIES_DIR: &str = "user-personalities";

/// Precedence contract: when IDs collide, user personality wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonalityPrecedencePolicy {
  UserOverridesSystemSameId,
}

pub const PERSONALITY_PRECEDENCE_POLICY: PersonalityPrecedencePolicy =
  PersonalityPrecedencePolicy::UserOverridesSystemSameId;

/// System personalities are immutable and cannot be created/updated/deleted.
pub const SYSTEM_PERSONALITIES_READ_ONLY: bool = true;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalitySource {
  System,
  User,
}

impl PersonalitySource {
  pub fn is_read_only(self) -> bool {
    match self {
      Self::System => SYSTEM_PERSONALITIES_READ_ONLY,
      Self::User => false,
    }
  }

  pub fn can_write(self) -> bool {
    !self.is_read_only()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalityFrontmatter {
  pub id:          String,
  pub name:        String,
  pub description: String,
  pub is_default:  bool,
  pub is_system:   bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalityDocument {
  pub frontmatter: PersonalityFrontmatter,
  pub body:        String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalityBasePaths {
  pub system: PathBuf,
  pub user:   PathBuf,
}

pub fn resolve_personality_base_dirs() -> PersonalityBasePaths {
  let base_dir = resolve_tauri_src_sibling_root();

  PersonalityBasePaths {
    system: base_dir.join(SYSTEM_PERSONALITIES_DIR),
    user:   base_dir.join(USER_PERSONALITIES_DIR),
  }
}

fn resolve_tauri_src_sibling_root() -> PathBuf {
  let mut probe_roots = Vec::new();

  if let Ok(current_dir) = std::env::current_dir() {
    probe_roots.push(current_dir);
  }

  if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
    probe_roots.push(PathBuf::from(manifest_dir));
  }

  probe_roots.push(BlprntPath::app_resources());

  for root in probe_roots {
    if root.join(SKILLS_DIR).is_dir() {
      return root;
    }

    for ancestor in root.ancestors() {
      let tauri_src = ancestor.join("tauri-src");
      if tauri_src.join(SKILLS_DIR).is_dir() {
        return tauri_src;
      }
    }
  }

  BlprntPath::app_resources()
}

pub fn normalize_personality_slug(input: &str) -> String {
  let mut slug = String::new();
  let mut last_dash = false;

  for ch in input.chars() {
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
  if slug.is_empty() { "personality".to_string() } else { slug }
}

pub fn with_collision_suffix(base_slug: &str, collision_index: u32) -> String {
  let normalized_base = normalize_personality_slug(base_slug);
  if collision_index == 0 { normalized_base } else { format!("{normalized_base}-{collision_index}") }
}

pub fn parse_personality_markdown(content: &str) -> Result<PersonalityDocument> {
  let mut sections = content.splitn(3, PERSONALITY_FRONTMATTER_DELIMITER);
  let first = sections.next().unwrap_or_default();
  if !first.trim().is_empty() {
    anyhow::bail!("personality frontmatter missing")
  }

  let yaml = sections.next().unwrap_or_default();
  let body = sections.next().unwrap_or_default();

  let frontmatter = serde_saphyr::from_str::<PersonalityFrontmatter>(yaml)
    .map_err(|error| anyhow::anyhow!("invalid personality frontmatter: {error}"))?;

  Ok(PersonalityDocument { frontmatter, body: body.trim_start().to_string() })
}

pub fn render_personality_markdown(document: &PersonalityDocument) -> Result<String> {
  let yaml = serde_saphyr::to_string(&document.frontmatter)
    .map_err(|error| anyhow::anyhow!("failed to encode personality frontmatter: {error}"))?;

  Ok(format!("{PERSONALITY_FRONTMATTER_DELIMITER}\n{yaml}{PERSONALITY_FRONTMATTER_DELIMITER}\n\n{}", document.body))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn slug_normalization_is_stable() {
    assert_eq!(normalize_personality_slug("Formal Assistant"), "formal-assistant");
    assert_eq!(normalize_personality_slug("***"), "personality");
    assert_eq!(normalize_personality_slug("  A__B  C  "), "a-b-c");
  }

  #[test]
  fn collision_suffix_rule_is_deterministic() {
    assert_eq!(with_collision_suffix("Formal Assistant", 0), "formal-assistant");
    assert_eq!(with_collision_suffix("Formal Assistant", 1), "formal-assistant-1");
    assert_eq!(with_collision_suffix("", 2), "personality-2");
  }

  #[test]
  fn frontmatter_round_trip() {
    let doc = PersonalityDocument {
      frontmatter: PersonalityFrontmatter {
        id:          "formal".to_string(),
        name:        "Formal".to_string(),
        description: "Professional and concise".to_string(),
        is_default:  false,
        is_system:   true,
      },
      body:        "System prompt body".to_string(),
    };

    let markdown = render_personality_markdown(&doc).unwrap();
    let parsed = parse_personality_markdown(&markdown).unwrap();
    assert_eq!(parsed, doc);
  }

  #[test]
  fn parse_requires_all_frontmatter_fields() {
    let content = r#"---
id: formal
name: Formal
description: Professional and concise
is_default: false
---

body
"#;

    let parsed = parse_personality_markdown(content);
    assert!(parsed.is_err());
  }
}
