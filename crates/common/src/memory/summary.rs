use super::utils::estimate_token_count;
use super::utils::truncate_summary_text;

pub(crate) fn enforce_markdown_token_budget(markdown: &str, token_budget: usize, with_warnings: bool) -> String {
  let trimmed = markdown.trim();
  if trimmed.is_empty() || token_budget == 0 {
    return String::new();
  }

  if estimate_token_count(trimmed) <= token_budget {
    return trimmed.to_string();
  }

  let mut remaining = token_budget;
  let mut kept_blocks = Vec::new();
  for block in trimmed.split("\n\n").map(str::trim).filter(|block| !block.is_empty()) {
    let block_tokens = estimate_token_count(block).max(1);
    if kept_blocks.is_empty() {
      if block_tokens > remaining {
        return truncate_summary_text(block, token_budget.saturating_mul(4));
      }
    } else if block_tokens > remaining {
      break;
    }

    remaining = remaining.saturating_sub(block_tokens);
    kept_blocks.push(block.to_string());
  }

  if kept_blocks.is_empty() {
    return truncate_summary_text(trimmed, token_budget.saturating_mul(4));
  }

  let fitted = kept_blocks.join("\n\n");
  if estimate_token_count(&fitted) <= token_budget {
    fitted
  } else {
    let truncated = truncate_summary_text(&fitted, token_budget.saturating_mul(4));
    if with_warnings { format!("**These results have been truncated**\n\n{}", truncated) } else { truncated }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn enforce_markdown_token_budget_keeps_short_summary_intact() {
    let summary = "## Preferences\nKeep output terse.";

    assert_eq!(enforce_markdown_token_budget(summary, 64, false), summary);
  }

  #[test]
  fn enforce_markdown_token_budget_drops_trailing_blocks_after_budget() {
    let summary = "## Preferences\n".to_string()
      + &"A".repeat(120)
      + "\n\n## Context\n"
      + &"B".repeat(120)
      + "\n\n## Extra\n"
      + &"C".repeat(120);

    let fitted = enforce_markdown_token_budget(&summary, 70, false);

    assert!(fitted.contains("## Preferences"));
    assert!(fitted.contains("## Context"));
    assert!(!fitted.contains("## Extra"));
    assert!(estimate_token_count(&fitted) <= 70);
  }
}
