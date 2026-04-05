crate::ix!();

pub fn repair_json_handle_eof_between_lists(input: &str) -> Result<String, JsonRepairError> {
  let mut repaired = input.to_owned();
  let mut changed = false;

  // Initialize variables
  let mut in_string = false;
  let mut escape = false;

  // Stack to keep track of open braces/brackets
  let mut stack: Vec<char> = Vec::new();
  let chars_iter = input.chars().enumerate().peekable();

  for (_, c) in chars_iter.clone() {
    if escape {
      escape = false;
      continue;
    }

    match c {
      '\\' => {
        escape = true;
      }
      '"' => {
        if !escape {
          in_string = !in_string;
        }
      }
      '{' if !in_string => {
        stack.push('}');
      }
      '}' if !in_string => {
        if let Some(expected) = stack.pop() {
          if expected != '}' {
            // Mismatched closing brace
            // Handle or log as needed
          }
        } else {
          // Unmatched closing brace
          // Handle or log as needed
        }
      }
      '[' if !in_string => {
        stack.push(']');
      }
      ']' if !in_string => {
        if let Some(expected) = stack.pop() {
          if expected != ']' {
            // Mismatched closing bracket
            // Handle or log as needed
          }
        } else {
          // Unmatched closing bracket
          // Handle or log as needed
        }
      }
      _ => {}
    }
  }

  // Close unclosed string
  if in_string {
    repaired.push('"');
    changed = true;
  }

  // Close any open braces/brackets in the correct order
  while let Some(c) = stack.pop() {
    repaired.push(c);
    changed = true;
  }

  if changed {
    info!("Repaired EOF between lists.");
  }

  Ok(repaired)
}
