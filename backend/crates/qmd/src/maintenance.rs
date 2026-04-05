use std::sync::Arc;

use crate::Result;
use crate::Storage;

#[derive(Clone)]
pub struct Maintenance {
  storage: Arc<dyn Storage>,
}

impl Maintenance {
  pub fn new(storage: Arc<dyn Storage>) -> Self {
    Self { storage }
  }

  pub async fn vacuum(&self) -> Result<()> {
    self.storage.vacuum().await
  }

  pub async fn cleanup_orphaned_content(&self) -> Result<usize> {
    self.storage.cleanup_orphaned_content().await
  }

  pub async fn cleanup_orphaned_vectors(&self) -> Result<usize> {
    self.storage.cleanup_orphaned_vectors().await
  }

  pub async fn clear_llm_cache(&self) -> Result<usize> {
    self.storage.clear_cached_results().await
  }

  pub async fn delete_inactive_docs(&self) -> Result<usize> {
    self.storage.delete_inactive_docs().await
  }

  pub async fn clear_embeddings(&self) -> Result<()> {
    self.storage.clear_all_embeddings().await
  }
}
