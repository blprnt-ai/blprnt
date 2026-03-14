crate::ix!();

#[allow(unused_assignments)]
pub fn repair_json_close_unexpected_eof_in_array_tag(input: &str) -> Result<String, JsonRepairError> {
  use std::collections::VecDeque;

  #[derive(Clone, Copy)]
  enum Context {
    Object,
    Array,
  }

  let mut repaired = String::new();
  let chars = input.chars().peekable();
  let mut stack = VecDeque::new();
  let mut context_stack = VecDeque::new();
  let mut in_string = false;
  let mut escaped = false;
  let mut changed = false;
  let mut added_chars = String::new();

  for c in chars.clone() {
    repaired.push(c);

    if in_string {
      if escaped {
        escaped = false;
      } else if c == '\\' {
        escaped = true;
      } else if c == '"' {
        in_string = false;
      }
      continue;
    } else {
      if c == '"' {
        in_string = true;
      } else if c == '{' {
        stack.push_back('}');
        context_stack.push_back(Context::Object);
      } else if c == '[' {
        stack.push_back(']');
        context_stack.push_back(Context::Array);
      } else if (c == '}' || c == ']') && Some(c) == stack.back().copied() {
        stack.pop_back();
        context_stack.pop_back();
      }
    }
  }

  // If we are still inside a string, close it
  if in_string {
    repaired.push('"');
    changed = true;
    added_chars.push('"');
  }

  // Remove trailing whitespace
  let mut temp = repaired.clone();
  while temp.ends_with(|c: char| c.is_whitespace()) {
    temp.pop();
  }

  // Get current context
  let current_context = context_stack.back().copied();

  // Attempt to fix incomplete structures
  if temp.ends_with(':') {
    // Missing value after colon
    repaired.push_str(" null");
    changed = true;
    added_chars.push_str(" null");
  } else if temp.ends_with('"') {
    // Possibly incomplete key or value
    let mut chars = temp.chars().rev().peekable();
    // Skip the closing quote
    chars.next();

    let mut in_string = true;
    let mut escaped = false;

    for c in chars.clone() {
      if in_string {
        if escaped {
          escaped = false;
        } else if c == '\\' {
          escaped = true;
        } else if c == '"' {
          in_string = false;
          break;
        }
      } else {
        break;
      }
    }

    // Skip whitespace
    while let Some(&c) = chars.peek() {
      if c.is_whitespace() {
        chars.next();
      } else {
        break;
      }
    }

    if let Some(context) = current_context {
      match context {
        Context::Object => {
          if let Some(&c) = chars.peek()
            && (c == ',' || c == '{')
          {
            // Missing colon and value after key
            repaired.push_str(": null");
            changed = true;
            added_chars.push_str(": null");
          }
        }
        Context::Array => {
          // Do nothing in array context
        }
      }
    }
  }

  // Close any open structures
  while let Some(c) = stack.pop_back() {
    repaired.push(c);
    changed = true;
    added_chars.push(c);
  }

  if changed {
    info!("Repaired JSON by adding the following characters at the end: {}", added_chars);
  }

  Ok(repaired)
}
