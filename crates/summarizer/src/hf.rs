use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use hf_hub::Repo;
use hf_hub::RepoType;
use hf_hub::api::tokio::Api;

#[derive(Debug, Clone)]
pub struct HuggingFaceRepoRef {
  pub repo_id:   String,
  pub revision:  String,
  pub repo_type: RepoType,
}

impl HuggingFaceRepoRef {
  pub fn model(repo_id: impl Into<String>) -> Self {
    Self { repo_id: repo_id.into(), revision: "main".to_string(), repo_type: RepoType::Model }
  }

  pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
    self.revision = revision.into();
    self
  }

  pub fn with_repo_type(mut self, repo_type: RepoType) -> Self {
    self.repo_type = repo_type;
    self
  }

  fn to_repo(&self) -> Repo {
    Repo::with_revision(self.repo_id.clone(), self.repo_type, self.revision.clone())
  }
}

#[derive(Debug, Clone)]
pub struct DownloadedModelFiles {
  pub config_path:           Option<PathBuf>,
  pub tokenizer_config_path: Option<PathBuf>,
  pub tokenizer_json_path:   Option<PathBuf>,
  pub tokenizer_model_path:  Option<PathBuf>,
  pub model_path:            PathBuf,
}

pub async fn download_file(repo_ref: &HuggingFaceRepoRef, file_name: &str) -> Result<PathBuf> {
  let api = Api::new().context("failed to initialize Hugging Face API client")?;
  let repo = api.repo(repo_ref.to_repo());

  let downloaded_path = repo.get(file_name).await.with_context(|| {
    format!("failed to download '{}' from repo '{}' at revision '{}'", file_name, repo_ref.repo_id, repo_ref.revision)
  })?;

  Ok(downloaded_path)
}

pub async fn try_download_file(repo_ref: &HuggingFaceRepoRef, file_name: &str) -> Result<Option<PathBuf>> {
  match download_file(repo_ref, file_name).await {
    Ok(path) => Ok(Some(path)),
    Err(_) => Ok(None),
  }
}

pub async fn download_gguf_model(repo_id: impl Into<String>, file_name: impl AsRef<str>) -> Result<PathBuf> {
  let repo_ref = HuggingFaceRepoRef::model(repo_id);
  download_file(&repo_ref, file_name.as_ref()).await
}

pub async fn download_gguf_model_with_revision(
  repo_id: impl Into<String>,
  revision: impl Into<String>,
  file_name: impl AsRef<str>,
) -> Result<PathBuf> {
  let repo_ref = HuggingFaceRepoRef::model(repo_id).with_revision(revision);
  download_file(&repo_ref, file_name.as_ref()).await
}

pub async fn download_common_model_files(
  repo_ref: &HuggingFaceRepoRef,
  preferred_model_file_names: &[&str],
) -> Result<DownloadedModelFiles> {
  let model_path = download_first_available(repo_ref, preferred_model_file_names).await?.with_context(|| {
    format!(
      "none of the requested model files were found in repo '{}': {}",
      repo_ref.repo_id,
      preferred_model_file_names.join(", ")
    )
  })?;

  let config_path = try_download_file(repo_ref, "config.json").await?;
  let tokenizer_config_path = try_download_file(repo_ref, "tokenizer_config.json").await?;
  let tokenizer_json_path = try_download_file(repo_ref, "tokenizer.json").await?;
  let tokenizer_model_path = try_download_file(repo_ref, "tokenizer.model").await?;

  Ok(DownloadedModelFiles { config_path, tokenizer_config_path, tokenizer_json_path, tokenizer_model_path, model_path })
}

pub async fn download_first_available(
  repo_ref: &HuggingFaceRepoRef,
  candidate_file_names: &[&str],
) -> Result<Option<PathBuf>> {
  for candidate_file_name in candidate_file_names {
    if let Some(path) = try_download_file(repo_ref, candidate_file_name).await? {
      return Ok(Some(path));
    }
  }

  Ok(None)
}

pub fn copy_model_to(source_model_path: impl AsRef<Path>, destination_model_path: impl AsRef<Path>) -> Result<PathBuf> {
  let source_model_path = source_model_path.as_ref();
  let destination_model_path = destination_model_path.as_ref();

  if let Some(parent_directory) = destination_model_path.parent() {
    fs::create_dir_all(parent_directory)
      .with_context(|| format!("failed to create destination directory '{}'", parent_directory.display()))?;
  }

  fs::copy(source_model_path, destination_model_path).with_context(|| {
    format!("failed to copy '{}' to '{}'", source_model_path.display(), destination_model_path.display())
  })?;

  Ok(destination_model_path.to_path_buf())
}
