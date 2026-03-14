crate::ix!();

pub fn repair_json_truncated_boolean_behavior(input: &str) -> Result<String, JsonRepairError> {
  let mut output = String::new();
  let chars = input.chars().peekable();
  let mut inside_string = false;
  let mut last_token = String::new();
  let mut open_braces = 0;
  let mut open_brackets = 0;
  let mut changed = false;

  for c in chars.clone() {
    output.push(c);

    if c == '"' {
      // Check if the quote is escaped
      let mut backslash_count = 0;

      if output.len() >= 2 {
        let mut idx = output.len() - 2;
        loop {
          if output.as_bytes()[idx] == b'\\' {
            backslash_count += 1;
            if idx == 0 {
              break;
            } else {
              idx -= 1;
            }
          } else {
            break;
          }
        }
      }

      if backslash_count % 2 == 0 {
        inside_string = !inside_string;
      }
      last_token.clear();
    } else if !inside_string {
      match c {
        '{' => open_braces += 1,
        '}' => {
          if open_braces > 0 {
            open_braces -= 1;
          }
        }
        '[' => open_brackets += 1,
        ']' => {
          if open_brackets > 0 {
            open_brackets -= 1;
          }
        }
        _ => {}
      }

      if c.is_alphabetic() {
        last_token.push(c);
      } else {
        last_token.clear();
      }
    }
  }

  // If we ended with an incomplete boolean
  if !inside_string {
    let mut appended = String::new();
    if last_token == "t" || last_token == "tr" || last_token == "tru" {
      let remaining = &"true"[last_token.len()..];
      appended.push_str(remaining);
    } else if last_token == "f" || last_token == "fa" || last_token == "fal" || last_token == "fals" {
      let remaining = &"false"[last_token.len()..];
      appended.push_str(remaining);
    }

    if !appended.is_empty() {
      output.push_str(&appended);
      changed = true;
    }
  }

  // Close any unclosed strings
  if inside_string {
    output.push('"');
    changed = true;
  }

  // Close any unclosed brackets and braces
  for _ in 0..open_brackets {
    output.push(']');
    changed = true;
  }
  for _ in 0..open_braces {
    output.push('}');
    changed = true;
  }

  if changed {
    info!("Repaired truncated booleans or unclosed delimiters in JSON.");
  }

  Ok(output)
}
