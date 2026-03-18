use anyhow::Result;
use tokio::sync::mpsc;

pub trait LlmAdapter {
  fn run_step(step_content: String) -> Result<mpsc::Receiver<String>>;
}
