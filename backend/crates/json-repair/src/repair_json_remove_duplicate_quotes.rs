crate::ix!();

pub fn repair_json_remove_duplicate_quotes(input: &str) -> Result<String, JsonRepairError> {
  let mut repaired = String::with_capacity(input.len());
  let mut chars = input.chars().peekable();
  let mut in_string = false;
  let mut escape_next = false;

  let mut changed = false;

  while let Some(c) = chars.next() {
    if c == '\\' && !escape_next {
      // Start of an escape sequence
      escape_next = true;
      repaired.push(c);
      continue;
    }

    if c == '"' && !escape_next {
      if in_string {
        // Skip any duplicate quotes
        let mut skip_quotes = false;
        while let Some(&'"') = chars.peek() {
          chars.next();
          skip_quotes = true;
        }

        if skip_quotes {
          changed = true;
        }

        // Peek ahead to check the next non-space character
        let mut peek_chars = chars.clone();
        let mut next_non_space = None;
        while let Some(&next_c) = peek_chars.peek() {
          if next_c.is_whitespace() {
            peek_chars.next();
          } else {
            next_non_space = Some(next_c);
            break;
          }
        }

        if let Some(next_c) = next_non_space {
          if [',', '}', ']', ':'].contains(&next_c) {
            // Valid end of string
            in_string = false;
            repaired.push(c);
          } else {
            // Unescaped quote inside string, remove it
            changed = true;
            continue;
          }
        } else {
          // End of input, assume end of string
          in_string = false;
          repaired.push(c);
        }
      } else {
        // Start of string
        in_string = true;
        repaired.push(c);
      }
      escape_next = false;
    } else {
      if escape_next {
        escape_next = false;
      }
      repaired.push(c);
    }
  }

  if changed {
    info!("Removed duplicate or invalid quotes in JSON.");
  }

  Ok(repaired)
}
