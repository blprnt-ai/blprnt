pub(crate) fn estimate_token_count(value: &str) -> usize {
  (value.chars().count() / 4).max(1)
}

pub(crate) fn truncate_summary_text(value: &str, max_chars: usize) -> String {
  let trimmed = value.trim();
  if trimmed.chars().count() <= max_chars {
    return trimmed.to_string();
  }
  let truncated = trimmed.chars().take(max_chars.saturating_sub(1)).collect::<String>();
  format!("{}…", truncated.trim_end())
}

pub(crate) fn normalize_for_match(value: &str) -> String {
  value
    .to_lowercase()
    .chars()
    .map(|character| if character.is_alphanumeric() { character } else { ' ' })
    .collect::<String>()
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}
