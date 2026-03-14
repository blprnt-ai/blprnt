#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedDailyMemoryDocument {
  pub date:         String,
  pub timezone:     String,
  pub raw_markdown: String,
}

/// Canonical rolling summary document backing `memories/summary.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedMemorySummaryDocument {
  pub content: String,
}

impl ManagedDailyMemoryDocument {
  pub fn to_markdown(&self) -> String {
    let content = self.raw_markdown.trim();
    if content.is_empty() { String::new() } else { format!("{content}\n") }
  }

  pub(crate) fn from_plain_markdown(raw_markdown: String) -> Self {
    Self { date: String::new(), timezone: String::new(), raw_markdown }
  }
}

impl ManagedMemorySummaryDocument {
  pub fn to_markdown(&self) -> String {
    let content = self.content.trim();
    if content.is_empty() { String::new() } else { format!("{content}\n") }
  }
}
