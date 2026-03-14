use std::num::NonZeroU32;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use llama_cpp_4::context::LlamaContext;
use llama_cpp_4::context::params::LlamaContextParams;
use llama_cpp_4::llama_backend::LlamaBackend;
use llama_cpp_4::llama_batch::LlamaBatch;
use llama_cpp_4::model::AddBos;
use llama_cpp_4::model::LlamaModel;
use llama_cpp_4::model::Special;
use llama_cpp_4::model::params::LlamaModelParams;
use llama_cpp_4::sampling::LlamaSampler;
use llama_cpp_4::token::LlamaToken;

pub use crate::hf::DownloadedModelFiles;
pub use crate::hf::HuggingFaceRepoRef;
pub use crate::hf::copy_model_to;
pub use crate::hf::download_common_model_files;
pub use crate::hf::download_file;
pub use crate::hf::download_first_available;
pub use crate::hf::download_gguf_model;
pub use crate::hf::download_gguf_model_with_revision;
pub use crate::hf::try_download_file;

#[derive(Debug, Clone)]
pub struct SummarizerConfig {
  pub model_path:           PathBuf,
  pub context_size:         u32,
  pub max_generated_tokens: usize,
  pub temperature:          f32,
  pub top_k:                i32,
  pub top_p:                f32,
  pub seed:                 u32,
}

impl Default for SummarizerConfig {
  fn default() -> Self {
    Self {
      model_path:           PathBuf::new(),
      context_size:         4096,
      max_generated_tokens: 220,
      temperature:          0.2,
      top_k:                40,
      top_p:                0.95,
      seed:                 1234,
    }
  }
}

pub struct LocalSummarizer {
  backend: LlamaBackend,
  model:   LlamaModel,
  config:  SummarizerConfig,
}

impl LocalSummarizer {
  pub fn load(config: SummarizerConfig) -> Result<Self> {
    let backend = LlamaBackend::init().context("failed to initialize llama backend")?;

    let model_params = LlamaModelParams::default();
    let model = LlamaModel::load_from_file(&backend, Path::new(&config.model_path), &model_params)
      .with_context(|| format!("failed to load GGUF model from {}", config.model_path.display()))?;

    Ok(Self { backend, model, config })
  }

  pub async fn load_from_hugging_face_gguf(
    repo_id: impl Into<String>,
    file_name: impl AsRef<str>,
    mut config: SummarizerConfig,
  ) -> Result<Self> {
    let downloaded_model_path = download_gguf_model(repo_id, file_name).await?;
    config.model_path = downloaded_model_path;
    Self::load(config)
  }

  pub fn summarize(&self, system_prompt: &str, input_text: &str) -> Result<String> {
    let prompt = self.build_summary_prompt(system_prompt, input_text);
    let mut context = self.create_context()?;

    let prompt_tokens =
      self.model.str_to_token(&prompt, AddBos::Never).context("failed to tokenize summarization prompt")?;

    let mut batch = LlamaBatch::new(self.config.max_generated_tokens.max(prompt_tokens.len()) + 16, 1);

    for (token_index, token) in prompt_tokens.iter().copied().enumerate() {
      batch
        .add(token, token_index as i32, &[0], token_index == prompt_tokens.len() - 1)
        .context("failed to add prompt token to batch")?;
    }

    context.decode(&mut batch).context("initial decode failed")?;

    let sampler = LlamaSampler::chain_simple([
      LlamaSampler::temp(self.config.temperature),
      LlamaSampler::top_k(self.config.top_k),
      LlamaSampler::top_p(self.config.top_p, 1),
      LlamaSampler::dist(self.config.seed),
    ]);

    let mut generated_text = String::new();
    let mut generated_token_count = 0usize;

    loop {
      let next_token = sampler.sample(&context, batch.n_tokens() - 1);

      if self.model.is_eog_token(next_token) {
        break;
      }

      let piece = token_to_piece(&self.model, next_token)?;
      generated_text.push_str(&piece);

      generated_token_count += 1;
      if generated_token_count >= self.config.max_generated_tokens {
        break;
      }

      batch.clear();
      batch.add(next_token, 0, &[0], true).context("failed to add generated token to batch")?;

      context.decode(&mut batch).context("decode during generation failed")?;
    }

    Ok(clean_summary_output(&generated_text))
  }

  fn create_context(&self) -> Result<LlamaContext<'_>> {
    let context_size = NonZeroU32::new(self.config.context_size).context("context_size must be greater than zero")?;

    let context_params = LlamaContextParams::default().with_n_ctx(Some(context_size));

    self.model.new_context(&self.backend, context_params).context("failed to create llama context")
  }

  fn build_summary_prompt(&self, system_prompt: &str, source_text: &str) -> String {
    format!(
      "<|im_start|>system\n\
You are a precise summarization assistant.\n\
{}\n\
<|im_end|>\n\
<|im_start|>user\n\
{}\n\
<|im_end|>\n\
<|im_start|>assistant\n",
      system_prompt, source_text
    )
  }
}

fn token_to_piece(model: &LlamaModel, token: LlamaToken) -> Result<String> {
  let bytes = model.token_to_bytes(token, Special::Tokenize).context("failed to decode token bytes")?;

  Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn clean_summary_output(raw_output: &str) -> String {
  raw_output.trim().trim_matches('\u{0}').replace("<|im_end|>", "").replace("<|endoftext|>", "").trim().to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cleans_output_markers() {
    let cleaned = clean_summary_output("  hello world<|im_end|><|endoftext|>  ");
    assert_eq!(cleaned, "hello world");
  }

  #[test]
  fn default_config_is_reasonable() {
    let config = SummarizerConfig::default();
    assert_eq!(config.context_size, 4096);
    assert_eq!(config.max_generated_tokens, 220);
  }
}
