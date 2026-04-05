crate::ix!();

pub fn remove_control_characters_in_value(value: &mut Value) -> bool {
  let mut changed = false;
  match value {
    Value::String(s) => {
      let original = s.clone();
      let cleaned: String = s.chars().filter(|&c| (c >= '\u{20}' && c <= '\u{10FFFF}') || c == '\n').collect();

      if cleaned != *s {
        *s = cleaned;
        changed = true;
      }
    }
    Value::Array(arr) => {
      for v in arr {
        if remove_control_characters_in_value(v) {
          changed = true;
        }
      }
    }
    Value::Object(map) => {
      for v in map.values_mut() {
        if remove_control_characters_in_value(v) {
          changed = true;
        }
      }
    }
    _ => {}
  }

  if changed {
    info!("Removed control characters from JSON value.");
  }
  changed
}
