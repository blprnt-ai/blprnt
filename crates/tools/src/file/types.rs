#![allow(clippy::mut_range_bound, clippy::needless_range_loop)]

#[derive(Clone, Default, Copy, Debug, Eq, PartialEq)]
pub enum DiffMode {
  #[default]
  Default,
  Create,
}

pub struct ApplyPatch;

#[derive(Clone, Debug)]
struct Chunk {
  orig_index: usize,
  del_lines:  Vec<String>,
  ins_lines:  Vec<String>,
}

#[derive(Clone, Debug)]
struct ParserState {
  lines: Vec<String>,
  index: usize,
  fuzz:  i32,
}

impl ParserState {
  fn new(lines: Vec<String>) -> Self {
    Self { lines, index: 0, fuzz: 0 }
  }
}

#[derive(Clone, Debug)]
struct ParseUpdateResult {
  chunks: Vec<Chunk>,
}

#[derive(Clone, Debug)]
struct ReadSectionResult {
  next_context:   Vec<String>,
  section_chunks: Vec<Chunk>,
  end_index:      usize,
  eof:            bool,
}

#[derive(Clone, Debug)]
struct FindContextResult {
  new_index: isize,
  fuzz:      i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SectionMode {
  Keep,
  Add,
  Delete,
}

const END_PATCH: &str = "*** End Patch";
const END_FILE: &str = "*** End of File";
const END_SECTION_MARKERS: [&str; 5] = [END_PATCH, "*** Update File:", "*** Delete File:", "*** Add File:", END_FILE];
const SECTION_TERMINATORS: [&str; 4] = [END_PATCH, "*** Update File:", "*** Delete File:", "*** Add File:"];

impl ApplyPatch {
  pub fn apply_diff(input: &str, diff: &str, mode: Option<DiffMode>) -> Result<String, String> {
    let diff_lines = Self::normalize_diff_lines(diff);

    match mode.unwrap_or_default() {
      DiffMode::Create => Self::parse_create_diff(&diff_lines),
      DiffMode::Default => {
        let result = Self::parse_update_diff(&diff_lines, input)?;
        Self::apply_chunks(input, &result.chunks)
      }
    }
  }

  fn normalize_diff_lines(diff: &str) -> Vec<String> {
    let mut lines: Vec<String> =
      diff.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line).to_string()).collect();

    if matches!(lines.last(), Some(last) if last.is_empty()) {
      lines.pop();
    }

    lines
  }

  fn is_done(state: &ParserState, prefixes: &[&str]) -> bool {
    if state.index >= state.lines.len() {
      return true;
    }

    let current = &state.lines[state.index];
    prefixes.iter().any(|prefix| current.starts_with(prefix))
  }

  fn read_str(state: &mut ParserState, prefix: &str) -> String {
    if let Some(current) = state.lines.get(state.index)
      && current.starts_with(prefix)
    {
      state.index += 1;
      return current.strip_prefix(prefix).unwrap_or_default().to_string();
    }
    String::new()
  }

  fn parse_create_diff(lines: &[String]) -> Result<String, String> {
    let mut parser_lines = Vec::with_capacity(lines.len() + 1);
    parser_lines.extend(lines.iter().cloned());
    parser_lines.push(END_PATCH.to_string());

    let mut parser = ParserState::new(parser_lines);
    let mut output: Vec<String> = Vec::new();

    while !Self::is_done(&parser, &SECTION_TERMINATORS) {
      let line = parser.lines[parser.index].clone();
      parser.index += 1;
      if !line.starts_with('+') {
        return Err(format!("Invalid Add File Line: {}", line));
      }
      output.push(line[1..].to_string());
    }

    Ok(output.join("\n"))
  }

  fn parse_update_diff(lines: &[String], input: &str) -> Result<ParseUpdateResult, String> {
    let mut parser_lines = Vec::with_capacity(lines.len() + 1);
    parser_lines.extend(lines.iter().cloned());
    parser_lines.push(END_PATCH.to_string());

    let mut parser = ParserState::new(parser_lines);
    let input_lines: Vec<String> = input.split('\n').map(|line| line.to_string()).collect();
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut cursor: usize = 0;

    while !Self::is_done(&parser, &END_SECTION_MARKERS) {
      let anchor = Self::read_str(&mut parser, "@@ ");
      let has_bare_anchor = if anchor.is_empty() {
        if let Some(line) = parser.lines.get(parser.index) {
          if line == "@@" {
            parser.index += 1;
            true
          } else {
            false
          }
        } else {
          false
        }
      } else {
        false
      };

      if anchor.is_empty() && !has_bare_anchor && cursor != 0 {
        let line = Self::line_or_undefined(&parser.lines, parser.index);
        return Err(format!("Invalid Line:\n{}", line));
      }

      if !anchor.trim().is_empty() {
        cursor = Self::advance_cursor_to_anchor(&anchor, &input_lines, cursor, &mut parser);
      }

      let section_result = Self::read_section(&parser.lines, parser.index)?;
      let next_context_text = section_result.next_context.join("\n");
      let context_result = Self::find_context(&input_lines, &section_result.next_context, cursor, section_result.eof);

      if context_result.new_index == -1 {
        if section_result.eof {
          return Err(format!("Invalid EOF Context {}:\n{}", cursor, next_context_text));
        }
        return Err(format!("Invalid Context {}:\n{}", cursor, next_context_text));
      }

      let new_index = context_result.new_index as usize;
      parser.fuzz += context_result.fuzz;
      for mut chunk in section_result.section_chunks {
        chunk.orig_index += new_index;
        chunks.push(chunk);
      }

      cursor = new_index + section_result.next_context.len();
      parser.index = section_result.end_index;
    }

    Ok(ParseUpdateResult { chunks })
  }

  fn advance_cursor_to_anchor(
    anchor: &str,
    input_lines: &[String],
    mut cursor: usize,
    parser: &mut ParserState,
  ) -> usize {
    let mut found = false;

    if !input_lines.iter().take(cursor).any(|line| line == anchor) {
      for i in cursor..input_lines.len() {
        if input_lines[i] == anchor {
          cursor = i + 1;
          found = true;
          break;
        }
      }
    }

    let anchor_trim = anchor.trim();
    if !found && !input_lines.iter().take(cursor).any(|line| line.trim() == anchor_trim) {
      for i in cursor..input_lines.len() {
        if input_lines[i].trim() == anchor_trim {
          cursor = i + 1;
          parser.fuzz += 1;
          break;
        }
      }
    }

    cursor
  }

  fn read_section(lines: &[String], start_index: usize) -> Result<ReadSectionResult, String> {
    let mut context: Vec<String> = Vec::new();
    let mut del_lines: Vec<String> = Vec::new();
    let mut ins_lines: Vec<String> = Vec::new();
    let mut section_chunks: Vec<Chunk> = Vec::new();
    let mut mode = SectionMode::Keep;
    let mut index = start_index;
    let orig_index = index;

    while index < lines.len() {
      let raw = &lines[index];
      if raw.starts_with("@@")
        || raw.starts_with(END_PATCH)
        || raw.starts_with("*** Update File:")
        || raw.starts_with("*** Delete File:")
        || raw.starts_with("*** Add File:")
        || raw.starts_with(END_FILE)
      {
        break;
      }
      if raw == "***" {
        break;
      }
      if raw.starts_with("***") {
        return Err(format!("Invalid Line: {}", raw));
      }

      index += 1;
      let last_mode = mode;
      let mut line = raw.clone();
      if line.is_empty() {
        line = " ".to_string();
      }

      mode = match line.chars().next() {
        Some('+') => SectionMode::Add,
        Some('-') => SectionMode::Delete,
        Some(' ') => SectionMode::Keep,
        _ => return Err(format!("Invalid Line: {}", line)),
      };

      line = line[1..].to_string();

      let switching_to_context = mode == SectionMode::Keep && last_mode != mode;
      if switching_to_context && (!ins_lines.is_empty() || !del_lines.is_empty()) {
        let del_lines_len = del_lines.len();
        section_chunks.push(Chunk { del_lines, ins_lines, orig_index: context.len() - del_lines_len });
        del_lines = Vec::new();
        ins_lines = Vec::new();
      }

      match mode {
        SectionMode::Delete => {
          del_lines.push(line.clone());
          context.push(line);
        }
        SectionMode::Add => {
          ins_lines.push(line);
        }
        SectionMode::Keep => {
          context.push(line);
        }
      }
    }

    if !ins_lines.is_empty() || !del_lines.is_empty() {
      let del_lines_len = del_lines.len();
      section_chunks.push(Chunk { del_lines, ins_lines, orig_index: context.len() - del_lines_len });
    }

    if index < lines.len() && lines[index] == END_FILE {
      index += 1;
      return Ok(ReadSectionResult { end_index: index, eof: true, next_context: context, section_chunks });
    }

    if index == orig_index {
      let line = Self::line_or_undefined(lines, index);
      return Err(format!("Nothing in this section - index={} {}", index, line));
    }

    Ok(ReadSectionResult { end_index: index, eof: false, next_context: context, section_chunks })
  }

  fn find_context(lines: &[String], context: &[String], start: usize, eof: bool) -> FindContextResult {
    if eof {
      let end_start = lines.len().saturating_sub(context.len());
      let end_match = Self::find_context_core(lines, context, end_start);
      if end_match.new_index != -1 {
        return end_match;
      }
      let fallback = Self::find_context_core(lines, context, start);
      return FindContextResult { fuzz: fallback.fuzz + 10000, new_index: fallback.new_index };
    }

    Self::find_context_core(lines, context, start)
  }

  fn find_context_core(lines: &[String], context: &[String], start: usize) -> FindContextResult {
    if context.is_empty() {
      return FindContextResult { fuzz: 0, new_index: start as isize };
    }

    for i in start..lines.len() {
      if Self::equals_slice(lines, context, i, Self::identity) {
        return FindContextResult { fuzz: 0, new_index: i as isize };
      }
    }

    for i in start..lines.len() {
      if Self::equals_slice(lines, context, i, Self::trim_end) {
        return FindContextResult { fuzz: 1, new_index: i as isize };
      }
    }

    for i in start..lines.len() {
      if Self::equals_slice(lines, context, i, Self::trim_all) {
        return FindContextResult { fuzz: 100, new_index: i as isize };
      }
    }

    FindContextResult { fuzz: 0, new_index: -1 }
  }

  fn equals_slice(source: &[String], target: &[String], start: usize, map_fn: fn(&str) -> String) -> bool {
    if start + target.len() > source.len() {
      return false;
    }

    for i in 0..target.len() {
      if map_fn(&source[start + i]) != map_fn(&target[i]) {
        return false;
      }
    }

    true
  }

  fn apply_chunks(input: &str, chunks: &[Chunk]) -> Result<String, String> {
    let orig_lines: Vec<String> = input.split('\n').map(|line| line.to_string()).collect();
    let mut dest_lines: Vec<String> = Vec::new();
    let mut orig_index: usize = 0;

    for chunk in chunks {
      if chunk.orig_index > orig_lines.len() {
        return Err(format!("applyDiff: chunk.origIndex {} > input length {}", chunk.orig_index, orig_lines.len()));
      }
      if orig_index > chunk.orig_index {
        return Err(format!("applyDiff: overlapping chunk at {} (cursor {})", chunk.orig_index, orig_index));
      }

      dest_lines.extend_from_slice(&orig_lines[orig_index..chunk.orig_index]);
      orig_index = chunk.orig_index;

      if !chunk.ins_lines.is_empty() {
        dest_lines.extend(chunk.ins_lines.iter().cloned());
      }

      orig_index += chunk.del_lines.len();
    }

    dest_lines.extend_from_slice(&orig_lines[orig_index..]);
    Ok(dest_lines.join("\n"))
  }

  fn identity(value: &str) -> String {
    value.to_string()
  }

  fn trim_end(value: &str) -> String {
    value.trim_end().to_string()
  }

  fn trim_all(value: &str) -> String {
    value.trim().to_string()
  }

  fn line_or_undefined(lines: &[String], index: usize) -> String {
    lines.get(index).cloned().unwrap_or_else(|| "undefined".to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_apply_diff_full() {
    let input = "";
    let diff = r#"+[package]
+name = "blprnt"
+version = "0.9.0"
+edition = "2021"
+
+[dependencies]
+"#;

    let result = ApplyPatch::apply_diff(input, diff, Some(DiffMode::Create));
    assert_eq!(
      result,
      Ok("[package]\nname = \"blprnt\"\nversion = \"0.9.0\"\nedition = \"2021\"\n\n[dependencies]\n".to_string())
    );
  }
}
