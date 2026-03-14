use std::fs;
use std::path::Path;
use std::path::PathBuf;

use chrono::NaiveDate;
use convert_case::Case;
use convert_case::Casing;

use super::contracts::MemoryWriteStatus;
use super::errors::ManagedMemoryStoreError;
use super::managed::ManagedDailyMemoryDocument;
use super::parsing::parse_document;
use super::paths::MemoryPathInfo;
use super::paths::MemorySummaryPathInfo;
use super::utils::normalize_for_match;
use crate::memory::MemoryContract;
use crate::memory::MemorySummaryContract;
use crate::memory::local_today;
use crate::memory::summary::enforce_markdown_token_budget;
use crate::tools::MemoryWriteResult;

#[derive(Clone, Debug)]
pub struct ManagedMemoryStore {
  root: PathBuf,
}

impl ManagedMemoryStore {
  pub fn new(root: PathBuf) -> Self {
    Self { root }
  }

  pub fn root(&self) -> &PathBuf {
    &self.root
  }

  pub fn ensure_dir_for_date(&self, date: NaiveDate) -> Result<bool, ManagedMemoryStoreError> {
    let path_info = MemoryPathInfo::for_date(self.root.clone(), date);

    if !path_info.absolute_path.exists() {
      fs::create_dir_all(&path_info.absolute_path)?;
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn append_entry_for_date(
    &self,
    date: NaiveDate,
    content: &str,
  ) -> Result<MemoryWriteResult, ManagedMemoryStoreError> {
    let path_info = MemoryPathInfo::for_date(self.root.clone(), date);

    let mut current_slug: Option<String> = None;
    let mut file_buffer = String::new();

    for line in content.lines() {
      let is_topic_line = line.starts_with('#');

      if is_topic_line {
        if let Some(existing_slug) = current_slug.take() {
          let file_path = path_info.absolute_path.join(format!("{existing_slug}.md"));
          fs::write(file_path, &file_buffer)?;
          file_buffer.clear();
        }

        let heading_text = line.trim_start_matches('#').trim();
        let slug = heading_text
          .chars()
          .filter(|c| c.is_alphanumeric() || c.is_whitespace())
          .collect::<String>()
          .to_case(Case::Kebab);
        current_slug = Some(slug);
      }

      if current_slug.is_some() {
        file_buffer.push_str(line);
        file_buffer.push('\n');
      }
    }

    if let Some(existing_slug) = current_slug {
      let file_path = path_info.absolute_path.join(format!("{existing_slug}.md"));
      fs::write(file_path, &file_buffer)?;
    }

    Ok(MemoryWriteResult {
      status: MemoryWriteStatus::Written,
      path:   path_info.relative_display_path(),
      date:   path_info.date.clone(),
    })
  }

  pub fn parse_document(content: &str) -> Result<ManagedDailyMemoryDocument, ManagedMemoryStoreError> {
    parse_document(content)
  }

  pub fn read_document_from_path(path: &Path) -> Result<ManagedDailyMemoryDocument, ManagedMemoryStoreError> {
    let content = fs::read_to_string(path)?;
    let mut document = Self::parse_document(&content)?;
    if document.date.is_empty()
      && let Some(stem) = path.file_stem().and_then(|value| value.to_str())
      && chrono::NaiveDate::parse_from_str(stem, "%Y-%m-%d").is_ok()
    {
      document.date = stem.to_string();
    }
    Ok(document)
  }

  pub fn empty_document(date: NaiveDate, timezone: &str) -> String {
    let _ = date;
    let _ = timezone;
    String::new()
  }

  pub fn ensure_rolling_summary_file(&self) -> Result<MemorySummaryPathInfo, ManagedMemoryStoreError> {
    let path_info = MemorySummaryPathInfo::rolling(self.root.clone());
    self.ensure_rolling_summary_path(&path_info)?;
    Ok(path_info)
  }

  pub fn read_rolling_summary_markdown(&self) -> Result<String, ManagedMemoryStoreError> {
    let path_info = self.ensure_rolling_summary_file()?;
    fs::read_to_string(path_info.absolute_path).map_err(Into::into)
  }

  pub fn rewrite_rolling_summary_markdown(
    &self,
    content: &str,
  ) -> Result<MemorySummaryPathInfo, ManagedMemoryStoreError> {
    let path_info = MemorySummaryPathInfo::rolling(self.root.clone());
    self.write_rolling_summary_document(&path_info, content)?;
    Ok(path_info)
  }

  pub async fn read_all_daily_logs(prior_path: &Path) -> anyhow::Result<String> {
    let mut logs = Vec::new();
    for entry in fs::read_dir(prior_path)? {
      let entry = entry?;

      let path = entry.path();
      if path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("md") {
        logs.push(fs::read_to_string(path)?);
      }
    }

    Ok(logs.join("\n"))
  }

  pub fn newest_prior_daily_path_info(
    root: &std::path::Path,
    today: NaiveDate,
  ) -> anyhow::Result<Option<MemoryPathInfo>> {
    let daily_root = root.join(MemoryContract::DAILY_DIR);
    if !daily_root.exists() {
      return Ok(None);
    }

    let mut newest_prior = None;

    for entry in fs::read_dir(daily_root)? {
      let entry = entry?;
      let path = entry.path();

      if !path.is_dir() {
        continue;
      }

      let Some(date) = path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
      else {
        continue;
      };

      if date >= today {
        continue;
      }

      if newest_prior.as_ref().is_none_or(|current: &MemoryPathInfo| {
        date > NaiveDate::parse_from_str(&current.date, "%Y-%m-%d").expect("valid memory date")
      }) {
        newest_prior = Some(MemoryPathInfo::for_date(root.to_path_buf(), date));
      }
    }

    Ok(newest_prior)
  }

  pub async fn memory_for_agent(root: &Path) -> String {
    let summary_info = MemorySummaryPathInfo::rolling(root.to_path_buf());
    let summary_content = if summary_info.absolute_path.exists() {
      fs::read_to_string(summary_info.absolute_path).ok()
    } else {
      None
    }
    .map(|s| enforce_markdown_token_budget(&s, MemorySummaryContract::SOFT_TOKEN_BUDGET, true))
    .map(|s| format!("## Memories\n\nThis is a summary of your previous days memories, exluding today. Use them to help you answer the user's request. Ignore them if they're not relevant. \n<memory-summary>\n{s}\n</memory-summary>"));

    // // DO NOT REMOVE, MIGHT NEED THIS IN THE FUTURE
    let today = local_today();
    // let prior_daily = match Self::newest_prior_daily_path_info(root, today).ok().flatten() {
    //   Some(prior_path) => Self::read_all_daily_logs(&prior_path.absolute_path).await.ok().map(|s| (s, prior_path.date)),
    //   None => None,
    // }
    // .map(|(s, d)| {
    //   let s = enforce_markdown_token_budget(&s, MemorySummaryContract::SOFT_TOKEN_BUDGET, true);
    //   (s, d)
    // })
    // .map(|(s, d)| format!("<previous-memories date=\"{d}\">\n{s}\n</previous-memories>"));

    let todays_daily = Self::read_all_daily_logs(&MemoryPathInfo::for_date(root.to_path_buf(), today).absolute_path)
      .await
      .ok()
      .map(|s| enforce_markdown_token_budget(&s, MemorySummaryContract::SOFT_TOKEN_BUDGET, true))
      .map(|s| format!("These are your memories for today.\n<todays-memories>\n{s}\n</todays-memories>"));

    // summary_content.unwrap_or_default()

    [summary_content, todays_daily].into_iter().filter_map(|s| s).collect::<Vec<String>>().join("\n")
  }

  fn ensure_rolling_summary_path(&self, path_info: &MemorySummaryPathInfo) -> Result<(), ManagedMemoryStoreError> {
    if let Some(parent) = path_info.absolute_path.parent() {
      fs::create_dir_all(parent)?;
    }
    if !path_info.absolute_path.exists() {
      fs::write(&path_info.absolute_path, "")?;
    }
    Ok(())
  }

  fn write_rolling_summary_document(
    &self,
    path_info: &MemorySummaryPathInfo,
    content: &str,
  ) -> Result<(), ManagedMemoryStoreError> {
    if let Some(parent) = path_info.absolute_path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(&path_info.absolute_path, content)?;
    Ok(())
  }
}

// Keep these around for future heuristic work.
#[allow(dead_code)]
fn fuzzy_equal(a: &str, b: &str) -> bool {
  normalize_for_match(a) == normalize_for_match(b)
}
