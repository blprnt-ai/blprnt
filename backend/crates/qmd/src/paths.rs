use regex::Regex;

use crate::NamedCollection;
use crate::QmdError;
use crate::Result;
use crate::Storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualPath {
  pub collection_name: String,
  pub path:            String, // relative path within collection (may be empty)
}

// =============================================================================
// Hash/docid helpers
// =============================================================================

/// Extract the short docid from a full hash (first 6 characters).
pub fn get_docid(hash: &str) -> String {
  hash.chars().take(6).collect()
}

// =============================================================================
// Basic path helpers (ported from TS string-based utilities)
// =============================================================================

pub fn normalize_path_separators(path: &str) -> String {
  path.replace('\\', "/")
}

fn is_wsl() -> bool {
  std::env::var_os("WSL_DISTRO_NAME").is_some() || std::env::var_os("WSL_INTEROP").is_some()
}

/// Check if a path is absolute.
///
/// Supports:
/// - Unix paths: `/path/to/file`
/// - Windows native: `C:\path` or `C:/path`
/// - Git Bash: `/c/path` or `/C/path` (C-Z drives, excluding A/B)
pub fn is_absolute_path(path: &str) -> bool {
  if path.is_empty() {
    return false;
  }

  if path.starts_with('/') {
    if !is_wsl() && path.len() >= 3 && path.as_bytes()[2] == b'/' {
      let drive = path.as_bytes()[1] as char;
      if matches!(drive, 'c'..='z' | 'C'..='Z') {
        return true;
      }
    }
    return true;
  }

  let bytes = path.as_bytes();
  if bytes.len() >= 2 && bytes[1] == b':' {
    let c0 = bytes[0] as char;
    if c0.is_ascii_alphabetic() {
      return true;
    }
  }

  false
}

/// Get the relative path from a prefix.
/// Returns `None` if `path` is not under `prefix`.
/// Returns empty string if `path` equals `prefix`.
pub fn get_relative_path_from_prefix(path: &str, prefix: &str) -> Option<String> {
  if prefix.is_empty() {
    return None;
  }

  let normalized_path = normalize_path_separators(path);
  let normalized_prefix = normalize_path_separators(prefix);

  let prefix_with_slash =
    if !normalized_prefix.ends_with('/') { format!("{normalized_prefix}/") } else { normalized_prefix.clone() };

  if normalized_path == normalized_prefix {
    return Some(String::new());
  }

  if normalized_path.starts_with(&prefix_with_slash) {
    return Some(normalized_path[prefix_with_slash.len()..].to_string());
  }

  None
}

/// String-based `resolve()` that normalizes `.` and `..` and forces `/` separators.
///
/// This follows the TypeScript implementation closely to keep behavior aligned
/// across runtimes.
pub fn resolve(paths: &[&str]) -> Result<String> {
  if paths.is_empty() {
    return Err(QmdError::InvalidArgument { message: "resolve: at least one path segment is required".to_string() });
  }

  let normalized_paths: Vec<String> = paths.iter().map(|p| normalize_path_separators(p)).collect();

  let mut windows_drive = String::new();

  let first = normalized_paths[0].as_str();
  let mut result = if is_absolute_path(first) {
    let mut result = first.to_string();

    if first.len() >= 2 && first.as_bytes()[1] == b':' && (first.as_bytes()[0] as char).is_ascii_alphabetic() {
      windows_drive = first[0..2].to_string();
      result = first[2..].to_string();
    } else if !is_wsl() && first.starts_with('/') && first.len() >= 3 && first.as_bytes()[2] == b'/' {
      let drive = first.as_bytes()[1] as char;
      if matches!(drive, 'c'..='z' | 'C'..='Z') {
        windows_drive = format!("{}:", drive.to_ascii_uppercase());
        result = first[2..].to_string();
      }
    }
    result
  } else {
    let pwd = std::env::var("PWD").ok().unwrap_or_else(|| {
      std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")).to_string_lossy().to_string()
    });
    let pwd = normalize_path_separators(&pwd);

    if pwd.len() >= 2 && pwd.as_bytes()[1] == b':' && (pwd.as_bytes()[0] as char).is_ascii_alphabetic() {
      windows_drive = pwd[0..2].to_string();
      format!("{}{}", &pwd[2..], format!("/{first}"))
    } else {
      format!("{pwd}/{first}")
    }
  };

  for p in normalized_paths.iter().skip(1) {
    if is_absolute_path(p) {
      result = p.clone();

      if p.len() >= 2 && p.as_bytes()[1] == b':' && (p.as_bytes()[0] as char).is_ascii_alphabetic() {
        windows_drive = p[0..2].to_string();
        result = p[2..].to_string();
      } else if !is_wsl() && p.starts_with('/') && p.len() >= 3 && p.as_bytes()[2] == b'/' {
        let drive = p.as_bytes()[1] as char;
        if matches!(drive, 'c'..='z' | 'C'..='Z') {
          windows_drive = format!("{}:", drive.to_ascii_uppercase());
          result = p[2..].to_string();
        } else {
          windows_drive.clear();
        }
      } else {
        windows_drive.clear();
      }
    } else {
      result.push('/');
      result.push_str(p);
    }
  }

  let parts: Vec<&str> = result.split('/').filter(|s| !s.is_empty()).collect();
  let mut normalized: Vec<&str> = Vec::new();
  for part in parts {
    if part == ".." {
      normalized.pop();
    } else if part != "." {
      normalized.push(part);
    }
  }

  let final_path = format!("/{}", normalized.join("/"));
  if !windows_drive.is_empty() {
    return Ok(format!("{windows_drive}{final_path}"));
  }
  Ok(final_path)
}

// =============================================================================
// Virtual path utilities (qmd://)
// =============================================================================

/// Normalize explicit virtual path formats to standard `qmd://` format.
pub fn normalize_virtual_path(input: &str) -> String {
  let mut path = input.trim().to_string();

  if path.starts_with("qmd:") {
    path = path[4..].to_string();
    path = path.trim_start_matches('/').to_string();
    return format!("qmd://{path}");
  }

  if path.starts_with("//") {
    path = path.trim_start_matches('/').to_string();
    return format!("qmd://{path}");
  }

  path
}

/// Parse a virtual path like `qmd://collection-name/path/to/file.md`.
pub fn parse_virtual_path(virtual_path: &str) -> Option<VirtualPath> {
  let normalized = normalize_virtual_path(virtual_path);
  let rest = normalized.strip_prefix("qmd://")?;

  let (collection_name, path) = match rest.split_once('/') {
    Some((coll, p)) => (coll, p),
    None => (rest, ""),
  };

  if collection_name.is_empty() {
    return None;
  }

  Some(VirtualPath { collection_name: collection_name.to_string(), path: path.to_string() })
}

/// Build a virtual path from collection name and relative path.
pub fn build_virtual_path(collection_name: &str, path: &str) -> String {
  format!("qmd://{collection_name}/{path}")
}

/// Check if a path is explicitly a virtual path (`qmd:` or `//` form).
pub fn is_virtual_path(path: &str) -> bool {
  let trimmed = path.trim();
  trimmed.starts_with("qmd:") || trimmed.starts_with("//")
}

/// Resolve a virtual path to an absolute filesystem path.
pub async fn resolve_virtual_path(storage: &dyn Storage, virtual_path: &str) -> Result<Option<String>> {
  let parsed = match parse_virtual_path(virtual_path) {
    Some(v) => v,
    None => return Ok(None),
  };

  let collections = storage.list_collections().await?;
  let Some(NamedCollection { collection, .. }) = collections.into_iter().find(|c| c.name == parsed.collection_name)
  else {
    return Ok(None);
  };

  Ok(Some(resolve(&[collection.path.as_str(), parsed.path.as_str()])?))
}

/// Convert an absolute filesystem path to a virtual path.
///
/// This Rust port does not validate document existence (the TS version checked
/// its local index for an active document row). Consumers can enforce that policy
/// in their `Storage` implementation when needed.
pub async fn to_virtual_path(storage: &dyn Storage, absolute_path: &str) -> Result<Option<String>> {
  let collections = storage.list_collections().await?;
  for coll in collections {
    let coll_path = coll.collection.path.clone();
    if absolute_path == coll_path || absolute_path.starts_with(&(coll_path.clone() + "/")) {
      let rel = if absolute_path == coll_path { "" } else { &absolute_path[(coll_path.len() + 1)..] };
      return Ok(Some(build_virtual_path(&coll.name, rel)));
    }
  }
  Ok(None)
}

// =============================================================================
// handelize()
// =============================================================================

fn is_variation_selector(c: char) -> bool {
  let cp = c as u32;
  (0xFE00..=0xFE0F).contains(&cp) || (0xE0100..=0xE01EF).contains(&cp)
}

fn emoji_to_hex(input: &str) -> String {
  let re = Regex::new(r"(\p{So}\p{Mn}?|\p{Sk})+").expect("emoji regex must compile");
  re.replace_all(input, |caps: &regex::Captures<'_>| {
    let run = caps.get(0).unwrap().as_str();
    let mut parts: Vec<String> = Vec::new();
    for ch in run.chars() {
      if is_variation_selector(ch) {
        continue;
      }
      let cp = ch as u32;
      parts.push(format!("{cp:x}"));
    }
    parts.join("-")
  })
  .to_string()
}

pub fn handelize(path: &str) -> Result<String> {
  if path.trim().is_empty() {
    return Err(QmdError::InvalidArgument { message: "handelize: path cannot be empty".to_string() });
  }

  let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
  let last_segment = segments.last().copied().unwrap_or("");
  let filename_without_ext = Regex::new(r"\.[^.]+$").unwrap().replace(last_segment, "").to_string();

  let has_valid_content = Regex::new(r"[\p{L}\p{N}\p{So}\p{Sk}$]").unwrap().is_match(&filename_without_ext);
  if !has_valid_content {
    return Err(QmdError::InvalidArgument {
      message: format!("handelize: path \"{path}\" has no valid filename content"),
    });
  }

  let cleaned = path
    .replace("___", "/")
    .to_lowercase()
    .split('/')
    .enumerate()
    .map(|(idx, segment)| {
      let segment = emoji_to_hex(segment);
      let is_last = idx == path.replace("___", "/").split('/').filter(|s| !s.is_empty()).count() - 1;

      if is_last {
        let ext_re = Regex::new(r"(\.[a-z0-9]+)$").unwrap();
        let ext = ext_re.captures(&segment).and_then(|c| c.get(1).map(|m| m.as_str().to_string())).unwrap_or_default();
        let name_without_ext =
          if ext.is_empty() { segment.clone() } else { segment[..(segment.len() - ext.len())].to_string() };

        let clean_re = Regex::new(r"[^\p{L}\p{N}$]+").unwrap();
        let cleaned_name = clean_re.replace_all(&name_without_ext, "-").to_string();
        let cleaned_name = cleaned_name.trim_matches('-').to_string();
        format!("{cleaned_name}{ext}")
      } else {
        let clean_re = Regex::new(r"[^\p{L}\p{N}$]+").unwrap();
        let dir = clean_re.replace_all(&segment, "-").to_string();
        dir.trim_matches('-').to_string()
      }
    })
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join("/");

  if cleaned.is_empty() {
    return Err(QmdError::InvalidArgument {
      message: format!("handelize: path \"{path}\" resulted in empty string after processing"),
    });
  }

  Ok(cleaned)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_relative_path_from_prefix_matches_ts_semantics() {
    assert_eq!(get_relative_path_from_prefix("/foo", "/foo"), Some("".to_string()));
    assert_eq!(get_relative_path_from_prefix("/foo/bar", "/foo"), Some("bar".to_string()));
    assert_eq!(get_relative_path_from_prefix("/foobar", "/foo"), None);
    assert_eq!(get_relative_path_from_prefix("/foo/bar", ""), None);
  }

  #[test]
  fn resolve_normalizes_dotdot_components() {
    let got = resolve(&["/a/b", "../c"]).unwrap();
    assert_eq!(got, "/a/c");
  }

  #[test]
  fn normalize_and_parse_virtual_paths() {
    assert_eq!(normalize_virtual_path("qmd:////docs/readme.md"), "qmd://docs/readme.md");
    assert_eq!(normalize_virtual_path("//docs/readme.md"), "qmd://docs/readme.md");

    let parsed = parse_virtual_path("qmd://docs/readme.md").unwrap();
    assert_eq!(parsed.collection_name, "docs");
    assert_eq!(parsed.path, "readme.md");

    let parsed_root = parse_virtual_path("qmd://docs/").unwrap();
    assert_eq!(parsed_root.path, "");
  }

  #[test]
  fn handelize_basic_cases() {
    assert_eq!(handelize("Read Me.md").unwrap(), "read-me.md");
    assert_eq!(handelize("foo___bar.md").unwrap(), "foo/bar.md");
    assert_eq!(handelize("api/$id.md").unwrap(), "api/$id.md");
  }

  #[test]
  fn handelize_converts_emoji_to_hex() {
    assert_eq!(handelize("🐘.md").unwrap(), "1f418.md");
    assert_eq!(handelize("✈️.md").unwrap(), "2708.md");
  }

  #[test]
  fn handelize_rejects_symbol_only_filenames() {
    let err = handelize("!!!.md").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("no valid filename content"));
  }
}
