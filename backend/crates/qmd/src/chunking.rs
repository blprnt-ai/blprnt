use regex::Regex;

#[derive(Debug, Clone)]
pub struct BreakPoint {
  pub pos:        usize,
  pub score:      f32,
  pub break_type: &'static str,
}

#[derive(Debug, Clone)]
pub struct CodeFenceRegion {
  pub start: usize,
  pub end:   usize,
}

pub const CHUNK_SIZE_TOKENS: usize = 900;
pub const CHUNK_OVERLAP_TOKENS: usize = (CHUNK_SIZE_TOKENS as f32 * 0.15) as usize;
pub const CHUNK_SIZE_CHARS: usize = CHUNK_SIZE_TOKENS * 4;
pub const CHUNK_OVERLAP_CHARS: usize = CHUNK_OVERLAP_TOKENS * 4;
pub const CHUNK_WINDOW_TOKENS: usize = 200;
pub const CHUNK_WINDOW_CHARS: usize = CHUNK_WINDOW_TOKENS * 4;

fn break_patterns() -> Vec<(Regex, f32, &'static str)> {
  vec![
    (Regex::new(r"\n#([^#]|$)").unwrap(), 100.0, "h1"),
    (Regex::new(r"\n##([^#]|$)").unwrap(), 90.0, "h2"),
    (Regex::new(r"\n###([^#]|$)").unwrap(), 80.0, "h3"),
    (Regex::new(r"\n####([^#]|$)").unwrap(), 70.0, "h4"),
    (Regex::new(r"\n#####([^#]|$)").unwrap(), 60.0, "h5"),
    (Regex::new(r"\n######([^#]|$)").unwrap(), 50.0, "h6"),
    (Regex::new(r"\n```").unwrap(), 80.0, "codeblock"),
    (Regex::new(r"\n(---|\*\*\*|___)\s*\n").unwrap(), 60.0, "hr"),
    (Regex::new(r"\n\n+").unwrap(), 20.0, "blank"),
    (Regex::new(r"\n[-*]\\s").unwrap(), 5.0, "list"),
    (Regex::new(r"\n\\d+\\.\\s").unwrap(), 5.0, "numlist"),
    (Regex::new(r"\n").unwrap(), 1.0, "newline"),
  ]
}

pub fn scan_break_points(text: &str) -> Vec<BreakPoint> {
  let mut seen: std::collections::BTreeMap<usize, BreakPoint> = std::collections::BTreeMap::new();

  for (re, score, break_type) in break_patterns() {
    for m in re.find_iter(text) {
      let pos = m.start();
      match seen.get(&pos) {
        Some(existing) if existing.score >= score => {}
        _ => {
          seen.insert(pos, BreakPoint { pos, score, break_type });
        }
      }
    }
  }

  seen.into_values().collect()
}

pub fn find_code_fences(text: &str) -> Vec<CodeFenceRegion> {
  let fence_re = Regex::new(r"\n```").unwrap();
  let mut regions = Vec::new();
  let mut in_fence = false;
  let mut fence_start = 0usize;

  for m in fence_re.find_iter(text) {
    if !in_fence {
      fence_start = m.start();
      in_fence = true;
    } else {
      regions.push(CodeFenceRegion { start: fence_start, end: m.start() + m.as_str().len() });
      in_fence = false;
    }
  }

  if in_fence {
    regions.push(CodeFenceRegion { start: fence_start, end: text.len() });
  }

  regions
}

pub fn is_inside_code_fence(pos: usize, fences: &[CodeFenceRegion]) -> bool {
  fences.iter().any(|f| pos > f.start && pos < f.end)
}

pub fn find_best_cutoff(
  break_points: &[BreakPoint],
  target_char_pos: usize,
  window_chars: usize,
  decay_factor: f32,
  code_fences: &[CodeFenceRegion],
) -> usize {
  let window_start = target_char_pos.saturating_sub(window_chars);
  let mut best_score = -1.0f32;
  let mut best_pos = target_char_pos;

  for bp in break_points {
    if bp.pos < window_start {
      continue;
    }
    if bp.pos > target_char_pos {
      break;
    }
    if is_inside_code_fence(bp.pos, code_fences) {
      continue;
    }

    let distance = (target_char_pos - bp.pos) as f32;
    let normalized_dist = if window_chars == 0 { 0.0 } else { distance / window_chars as f32 };
    let multiplier = 1.0 - (normalized_dist * normalized_dist) * decay_factor;
    let final_score = bp.score * multiplier;

    if final_score > best_score {
      best_score = final_score;
      best_pos = bp.pos;
    }
  }

  best_pos
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scan_break_points_prefers_heading_over_newline_at_same_position() {
    let text = "Intro\n# Heading\nBody\n";
    let bps = scan_break_points(text);
    let bp = bps.into_iter().find(|bp| bp.pos == "Intro".len()).unwrap();
    assert_eq!(bp.break_type, "h1");
    assert!(bp.score > 1.0);
  }

  #[test]
  fn scan_break_points_detects_h3_not_h2() {
    let text = "Intro\n### Three\nBody\n";
    let bps = scan_break_points(text);
    let bp = bps.into_iter().find(|bp| bp.pos == "Intro".len()).unwrap();
    assert_eq!(bp.break_type, "h3");
  }

  #[test]
  fn code_fence_regions_are_detected_and_respected() {
    let text = "a\n```\ncode\n```\nb\n";
    let fences = find_code_fences(text);
    assert_eq!(fences.len(), 1);

    let region = &fences[0];
    assert!(region.start < region.end);
    assert!(is_inside_code_fence(region.start + 2, &fences));
    assert!(!is_inside_code_fence(region.start, &fences)); // strict > start
    assert!(!is_inside_code_fence(region.end, &fences)); // strict < end
  }
}
