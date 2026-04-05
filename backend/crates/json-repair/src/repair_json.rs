crate::ix!();

/// applies all known fixes in order until one proves to work
pub fn repair_json_string(input: &str) -> Result<Value, JsonRepairError> {
  repair_json_string_series(input)
}

/// this one will try all of the fixes in parallel first before falling back to serial application.
///
/// may be useful in some contexts
pub fn repair_json_string_heavy(input: &str) -> Result<Value, JsonRepairError> {
  match repair_json_string_parallel(input) {
    Ok(repaired) => Ok(repaired),
    Err(e) => repair_json_string_series(input),
  }
}

/// this one we use for certain cases where the JSON is known to have list items which are all
/// `Sentence fragments of this format`
pub fn repair_json_with_known_capitalized_sentence_fragment_list_items(input: &str) -> Result<Value, JsonRepairError> {
  let repaired = repair_json_string_series(input)?;
  Ok(repair_standard_list_items_with_possible_splits(repaired))
}
