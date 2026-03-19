use anyhow::Result;
use persistence::prelude::RunRecord;
use tokio::sync::mpsc;

pub trait LlmAdapter {
  fn run_step(run: RunRecord) -> Result<mpsc::Receiver<String>>;
}
