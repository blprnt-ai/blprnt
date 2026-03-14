use serde_json::Value;

pub const ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY: &str = "advanced_reasoning_effort_classifier_enabled";
pub const ADVANCED_SKILL_MATCHER_ENABLED_KEY: &str = "advanced_skill_matcher_enabled";
pub const QMD_RUNTIME_SETUP_PROMPT_SUPPRESSED_KEY: &str = "qmd_runtime_setup_prompt_suppressed";

pub fn default_advanced_pre_turn_helper_enabled() -> bool {
  true
}

pub fn store_bool_with_default_true(value: Option<&Value>) -> bool {
  value.and_then(Value::as_bool).unwrap_or_else(default_advanced_pre_turn_helper_enabled)
}

pub fn store_bool_with_default_false(value: Option<&Value>) -> bool {
  value.and_then(Value::as_bool).unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::default_advanced_pre_turn_helper_enabled;
  use super::store_bool_with_default_false;
  use super::store_bool_with_default_true;

  #[test]
  fn advanced_pre_turn_helpers_default_to_enabled_when_missing() {
    assert!(default_advanced_pre_turn_helper_enabled());
    assert!(store_bool_with_default_true(None));
  }

  #[test]
  fn advanced_pre_turn_helpers_use_explicit_store_boolean_values() {
    assert!(store_bool_with_default_true(Some(&serde_json::json!(true))));
    assert!(!store_bool_with_default_true(Some(&serde_json::json!(false))));
  }

  #[test]
  fn store_bool_with_default_false_defaults_to_false_when_missing() {
    assert!(!store_bool_with_default_false(None));
  }

  #[test]
  fn store_bool_with_default_false_uses_explicit_store_boolean_values() {
    assert!(store_bool_with_default_false(Some(&serde_json::json!(true))));
    assert!(!store_bool_with_default_false(Some(&serde_json::json!(false))));
  }
}