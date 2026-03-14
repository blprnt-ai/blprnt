use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::personality_files::PERSONALITY_FILE_NAME;
use crate::personality_files::PERSONALITY_PRECEDENCE_POLICY;
use crate::personality_files::PersonalityDocument;
use crate::personality_files::PersonalityFrontmatter;
use crate::personality_files::PersonalityPrecedencePolicy;
use crate::personality_files::PersonalitySource;
use crate::personality_files::SYSTEM_PERSONALITIES_READ_ONLY;
use crate::personality_files::normalize_personality_slug;
use crate::personality_files::parse_personality_markdown;
use crate::personality_files::render_personality_markdown;
use crate::personality_files::resolve_personality_base_dirs;
use crate::personality_files::with_collision_suffix;

#[derive(Debug, Error)]
pub enum PersonalityServiceError {
  #[error("personality not found: {id}")]
  NotFound { id: String },

  #[error("system personality is read-only: {id}")]
  SystemReadOnly { id: String },

  #[error("invalid personality metadata: {message}")]
  InvalidMetadata { message: String },

  #[error("failed to read personalities directory '{path}': {error}")]
  DirectoryReadFailed { path: String, error: String },

  #[error("failed to read personality file '{path}': {error}")]
  FileReadFailed { path: String, error: String },

  #[error("failed to write personality file '{path}': {error}")]
  FileWriteFailed { path: String, error: String },

  #[error("failed to remove personality file '{path}': {error}")]
  FileDeleteFailed { path: String, error: String },

  #[error("failed to parse personality file '{path}': {error}")]
  ParseFailed { path: String, error: String },
}

pub type PersonalityServiceResult<T> = Result<T, PersonalityServiceError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalityRecord {
  pub source:      PersonalitySource,
  pub frontmatter: PersonalityFrontmatter,
  pub body:        String,
}

#[derive(Clone, Debug)]
pub struct PersonalityCreateInput {
  pub id:          Option<String>,
  pub name:        String,
  pub description: String,
  pub body:        String,
  pub is_default:  bool,
}

#[derive(Clone, Debug, Default)]
pub struct PersonalityUpdateInput {
  pub id:          Option<String>,
  pub name:        Option<String>,
  pub description: Option<String>,
  pub body:        Option<String>,
  pub is_default:  Option<bool>,
}

#[derive(Clone, Debug)]
pub struct PersonalityService {
  system_base_dir: PathBuf,
  user_base_dir:   PathBuf,
}

impl Default for PersonalityService {
  fn default() -> Self {
    Self::new()
  }
}

impl PersonalityService {
  pub fn new() -> Self {
    let base_dirs = resolve_personality_base_dirs();
    Self { system_base_dir: base_dirs.system, user_base_dir: base_dirs.user }
  }

  pub fn with_base_dirs(system_base_dir: PathBuf, user_base_dir: PathBuf) -> Self {
    Self { system_base_dir, user_base_dir }
  }

  pub fn list(&self) -> PersonalityServiceResult<Vec<PersonalityRecord>> {
    let system_records = self.load_source(PersonalitySource::System)?;
    let user_records = self.load_source(PersonalitySource::User)?;

    let mut merged = HashMap::<String, PersonalityRecord>::new();
    for record in system_records {
      merged.insert(record.frontmatter.id.clone(), record);
    }

    match PERSONALITY_PRECEDENCE_POLICY {
      PersonalityPrecedencePolicy::UserOverridesSystemSameId => {
        for record in user_records {
          merged.insert(record.frontmatter.id.clone(), record);
        }
      }
    }

    let mut items = merged.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| left.frontmatter.id.cmp(&right.frontmatter.id));
    Ok(items)
  }

  pub fn get(&self, id: &str) -> PersonalityServiceResult<Option<PersonalityRecord>> {
    let id = normalize_personality_slug(id);
    if let Some(user_record) = self.load_by_id(PersonalitySource::User, &id)? {
      return Ok(Some(user_record));
    }

    self.load_by_id(PersonalitySource::System, &id)
  }

  pub fn create(&self, input: PersonalityCreateInput) -> PersonalityServiceResult<PersonalityRecord> {
    self.ensure_user_dir_exists()?;

    let base_slug = input.id.unwrap_or_else(|| input.name.clone());
    let normalized_base_slug = normalize_personality_slug(&base_slug);
    let resolved_id = self.resolve_user_slug_collision(&normalized_base_slug)?;

    let frontmatter = PersonalityFrontmatter {
      id:          resolved_id,
      name:        input.name,
      description: input.description,
      is_default:  input.is_default,
      is_system:   false,
    };

    self.validate_frontmatter(&frontmatter)?;
    let body = Self::validate_body(input.body)?;

    let record = PersonalityRecord { source: PersonalitySource::User, frontmatter, body };
    self.write_record(&record)?;

    Ok(record)
  }

  pub fn update(&self, current_id: &str, input: PersonalityUpdateInput) -> PersonalityServiceResult<PersonalityRecord> {
    self.ensure_user_dir_exists()?;

    let current_id = normalize_personality_slug(current_id);
    let system_record = self.load_by_id(PersonalitySource::System, &current_id)?;
    let mut existing = self.load_by_id(PersonalitySource::User, &current_id)?.ok_or_else(|| match system_record {
      Some(_) => PersonalityServiceError::SystemReadOnly { id: current_id.clone() },
      None => PersonalityServiceError::NotFound { id: current_id.clone() },
    })?;

    let requested_id = input.id.unwrap_or_else(|| existing.frontmatter.id.clone());
    let normalized_requested_id = normalize_personality_slug(&requested_id);
    let updated_id = if normalized_requested_id == existing.frontmatter.id {
      existing.frontmatter.id.clone()
    } else {
      self.resolve_user_slug_collision(&normalized_requested_id)?
    };

    existing.frontmatter.id = updated_id;
    existing.frontmatter.name = input.name.unwrap_or(existing.frontmatter.name);
    existing.frontmatter.description = input.description.unwrap_or(existing.frontmatter.description);
    existing.frontmatter.is_default = input.is_default.unwrap_or(existing.frontmatter.is_default);
    existing.frontmatter.is_system = false;

    self.validate_frontmatter(&existing.frontmatter)?;
    existing.body = match input.body {
      Some(body) => Self::validate_body(body)?,
      None => Self::validate_body(existing.body)?,
    };

    self.write_record(&existing)?;
    if existing.frontmatter.id != current_id {
      self.remove_user_record_file(&current_id)?;
    }

    Ok(existing)
  }

  pub fn delete(&self, id: &str) -> PersonalityServiceResult<()> {
    let id = normalize_personality_slug(id);
    if self.load_by_id(PersonalitySource::User, &id)?.is_some() {
      self.remove_user_record_file(&id)?;
      return Ok(());
    }

    if self.load_by_id(PersonalitySource::System, &id)?.is_some() {
      return Err(PersonalityServiceError::SystemReadOnly { id });
    }

    Err(PersonalityServiceError::NotFound { id })
  }

  fn load_source(&self, source: PersonalitySource) -> PersonalityServiceResult<Vec<PersonalityRecord>> {
    let base_dir = self.base_dir(source);
    let entries = match fs::read_dir(&base_dir) {
      Ok(entries) => entries,
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(error) => {
        return Err(PersonalityServiceError::DirectoryReadFailed {
          path:  base_dir.display().to_string(),
          error: error.to_string(),
        });
      }
    };

    let mut records = Vec::new();
    for entry in entries {
      let entry = entry.map_err(|error| PersonalityServiceError::DirectoryReadFailed {
        path:  base_dir.display().to_string(),
        error: error.to_string(),
      })?;

      let entry_path = entry.path();
      if !entry_path.is_dir() {
        continue;
      }

      let file_path = entry_path.join(PERSONALITY_FILE_NAME);
      if !file_path.is_file() {
        continue;
      }

      let record = self.load_record_from_file(source, &file_path)?;
      records.push(record);
    }

    Ok(records)
  }

  fn load_by_id(&self, source: PersonalitySource, id: &str) -> PersonalityServiceResult<Option<PersonalityRecord>> {
    let file_path = self.user_or_system_file_path(source, id);
    if !file_path.is_file() {
      return Ok(None);
    }

    self.load_record_from_file(source, &file_path).map(Some)
  }

  fn load_record_from_file(
    &self,
    source: PersonalitySource,
    file_path: &PathBuf,
  ) -> PersonalityServiceResult<PersonalityRecord> {
    let content = fs::read_to_string(file_path).map_err(|error| PersonalityServiceError::FileReadFailed {
      path:  file_path.display().to_string(),
      error: error.to_string(),
    })?;

    let document = parse_personality_markdown(&content).map_err(|error| PersonalityServiceError::ParseFailed {
      path:  file_path.display().to_string(),
      error: error.to_string(),
    })?;

    self.validate_frontmatter(&document.frontmatter)?;
    let body = Self::validate_body(document.body)?;

    Ok(PersonalityRecord { source, frontmatter: document.frontmatter, body })
  }

  fn write_record(&self, record: &PersonalityRecord) -> PersonalityServiceResult<()> {
    if record.source == PersonalitySource::System && SYSTEM_PERSONALITIES_READ_ONLY {
      return Err(PersonalityServiceError::SystemReadOnly { id: record.frontmatter.id.clone() });
    }

    let file_path = self.user_or_system_file_path(record.source, &record.frontmatter.id);
    if let Some(parent) = file_path.parent() {
      fs::create_dir_all(parent).map_err(|error| PersonalityServiceError::FileWriteFailed {
        path:  parent.display().to_string(),
        error: error.to_string(),
      })?;
    }

    let document = PersonalityDocument { frontmatter: record.frontmatter.clone(), body: record.body.clone() };
    let content = render_personality_markdown(&document).map_err(|error| PersonalityServiceError::FileWriteFailed {
      path:  file_path.display().to_string(),
      error: error.to_string(),
    })?;

    fs::write(&file_path, content).map_err(|error| PersonalityServiceError::FileWriteFailed {
      path:  file_path.display().to_string(),
      error: error.to_string(),
    })
  }

  fn ensure_user_dir_exists(&self) -> PersonalityServiceResult<()> {
    fs::create_dir_all(&self.user_base_dir).map_err(|error| PersonalityServiceError::FileWriteFailed {
      path:  self.user_base_dir.display().to_string(),
      error: error.to_string(),
    })
  }

  fn remove_user_record_file(&self, id: &str) -> PersonalityServiceResult<()> {
    let file_path = self.user_or_system_file_path(PersonalitySource::User, id);
    if !file_path.is_file() {
      return Ok(());
    }

    fs::remove_file(&file_path).map_err(|error| PersonalityServiceError::FileDeleteFailed {
      path:  file_path.display().to_string(),
      error: error.to_string(),
    })?;

    if let Some(parent) = file_path.parent() {
      let _ = fs::remove_dir(parent);
    }

    Ok(())
  }

  fn resolve_user_slug_collision(&self, base_slug: &str) -> PersonalityServiceResult<String> {
    let base_slug = normalize_personality_slug(base_slug);
    for collision_index in 0..1000_u32 {
      let candidate = with_collision_suffix(&base_slug, collision_index);
      let candidate_path = self.user_or_system_file_path(PersonalitySource::User, &candidate);
      if !candidate_path.exists() {
        return Ok(candidate);
      }
    }

    Err(PersonalityServiceError::InvalidMetadata {
      message: format!("unable to resolve slug collision for '{base_slug}'"),
    })
  }

  fn validate_frontmatter(&self, frontmatter: &PersonalityFrontmatter) -> PersonalityServiceResult<()> {
    if frontmatter.name.trim().is_empty() {
      return Err(PersonalityServiceError::InvalidMetadata { message: "name cannot be empty".to_string() });
    }

    if frontmatter.description.trim().is_empty() {
      return Err(PersonalityServiceError::InvalidMetadata { message: "description cannot be empty".to_string() });
    }

    let normalized_id = normalize_personality_slug(&frontmatter.id);
    if normalized_id != frontmatter.id {
      return Err(PersonalityServiceError::InvalidMetadata {
        message: format!("id must be normalized slug '{normalized_id}'"),
      });
    }

    if frontmatter.is_system && SYSTEM_PERSONALITIES_READ_ONLY {
      return Ok(());
    }

    Ok(())
  }

  fn validate_body(body: String) -> PersonalityServiceResult<String> {
    if body.trim().is_empty() {
      return Err(PersonalityServiceError::InvalidMetadata { message: "body cannot be empty".to_string() });
    }

    Ok(body)
  }

  fn base_dir(&self, source: PersonalitySource) -> PathBuf {
    match source {
      PersonalitySource::System => self.system_base_dir.clone(),
      PersonalitySource::User => self.user_base_dir.clone(),
    }
  }

  fn user_or_system_file_path(&self, source: PersonalitySource, id: &str) -> PathBuf {
    let id = normalize_personality_slug(id);
    self.base_dir(source).join(id).join(PERSONALITY_FILE_NAME)
  }
}

#[cfg(test)]
mod tests {
  use std::time::SystemTime;
  use std::time::UNIX_EPOCH;

  use super::*;

  fn test_service() -> PersonalityService {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let system = std::env::temp_dir().join(format!("blprnt-personality-system-{unique}"));
    let user = std::env::temp_dir().join(format!("blprnt-personality-user-{unique}"));
    PersonalityService::with_base_dirs(system, user)
  }

  fn mk_record(id: &str, source: PersonalitySource) -> PersonalityRecord {
    PersonalityRecord {
      source,
      frontmatter: PersonalityFrontmatter {
        id:          id.to_string(),
        name:        id.to_string(),
        description: format!("{id} description"),
        is_default:  false,
        is_system:   source == PersonalitySource::System,
      },
      body: "prompt body".to_string(),
    }
  }

  fn seed_record(service: &PersonalityService, record: PersonalityRecord) {
    let file_path = service.user_or_system_file_path(record.source, &record.frontmatter.id);
    if let Some(parent) = file_path.parent() {
      fs::create_dir_all(parent).unwrap();
    }

    let content =
      render_personality_markdown(&PersonalityDocument { frontmatter: record.frontmatter, body: record.body }).unwrap();
    fs::write(file_path, content).unwrap();
  }

  #[test]
  fn list_is_deterministic_and_user_overrides_system() {
    let service = test_service();

    seed_record(&service, mk_record("zeta", PersonalitySource::System));
    seed_record(&service, mk_record("alpha", PersonalitySource::System));
    seed_record(
      &service,
      PersonalityRecord {
        source:      PersonalitySource::User,
        frontmatter: PersonalityFrontmatter {
          id:          "alpha".to_string(),
          name:        "alpha-user".to_string(),
          description: "override".to_string(),
          is_default:  false,
          is_system:   false,
        },
        body:        "prompt".to_string(),
      },
    );

    let items = service.list().unwrap();
    assert_eq!(items.iter().map(|item| item.frontmatter.id.clone()).collect::<Vec<_>>(), vec!["alpha", "zeta"]);
    assert_eq!(items[0].source, PersonalitySource::User);
    assert_eq!(items[0].frontmatter.name, "alpha-user");
  }

  #[test]
  fn create_normalizes_slug_and_resolves_user_collision() {
    let service = test_service();

    let first = service
      .create(PersonalityCreateInput {
        id:          None,
        name:        "Formal Assistant".to_string(),
        description: "desc".to_string(),
        body:        "body".to_string(),
        is_default:  false,
      })
      .unwrap();

    let second = service
      .create(PersonalityCreateInput {
        id:          None,
        name:        "Formal Assistant".to_string(),
        description: "desc".to_string(),
        body:        "body".to_string(),
        is_default:  false,
      })
      .unwrap();

    assert_eq!(first.frontmatter.id, "formal-assistant");
    assert_eq!(second.frontmatter.id, "formal-assistant-1");
  }

  #[test]
  fn delete_rejects_system_personality() {
    let service = test_service();
    seed_record(&service, mk_record("system-one", PersonalitySource::System));

    let err = service.delete("system-one").unwrap_err();
    assert!(matches!(err, PersonalityServiceError::SystemReadOnly { .. }));
  }
}
