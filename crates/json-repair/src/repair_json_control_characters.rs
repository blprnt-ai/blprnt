crate::ix!();

pub fn repair_json_control_characters(input: &str) -> Result<String, JsonRepairError> {
  let mut output = String::with_capacity(input.len());
  let mut changed = false;
  let mut removed_chars = Vec::new();

  for c in input.chars() {
    if (c >= '\u{20}' && c <= '\u{10FFFF}') || c == '\n' || c == '\t' {
      output.push(c);
    } else {
      changed = true;
      removed_chars.push(c);
    }
  }

  if changed {
    info!(
      "Removed control characters: {:?}",
      removed_chars.iter().map(|&c| format!("\\u{{{:04X}}}", c as u32)).collect::<Vec<String>>().join(", ")
    );
  }

  Ok(output)
}
