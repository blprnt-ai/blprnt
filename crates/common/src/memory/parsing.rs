use super::errors::ManagedMemoryStoreError;
use super::managed::ManagedDailyMemoryDocument;

pub(super) fn parse_document(content: &str) -> Result<ManagedDailyMemoryDocument, ManagedMemoryStoreError> {
  let normalized = content.replace("\r\n", "\n");
  let plain_markdown = normalized.trim().to_string();

  if plain_markdown.is_empty() {
    Ok(ManagedDailyMemoryDocument::from_plain_markdown(String::new()))
  } else {
    Ok(ManagedDailyMemoryDocument::from_plain_markdown(plain_markdown))
  }
}
