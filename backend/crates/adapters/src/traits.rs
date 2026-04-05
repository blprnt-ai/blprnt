use std::sync::Arc;

use anyhow::Result;
use events::AdapterEvent;
use persistence::prelude::RunRecord;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub trait LlmAdapter {
  fn run(run: RunRecord, tx: Arc<oneshot::Sender<Result<()>>>);

  fn emit_event(tx: Arc<mpsc::Sender<AdapterEvent>>);
}
