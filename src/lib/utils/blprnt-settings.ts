export const ADVANCED_REASONING_EFFORT_CLASSIFIER_ENABLED_KEY = 'advanced_reasoning_effort_classifier_enabled'
export const ADVANCED_SKILL_MATCHER_ENABLED_KEY = 'advanced_skill_matcher_enabled'
export const ADVANCED_USE_MEMORY_HOOKS_KEY = 'advanced_use_memory_hooks'

export const defaultAdvancedPreTurnHelperEnabled = () => true

export const storeBoolWithDefaultTrue = (value: unknown) =>
  typeof value === 'boolean' ? value : defaultAdvancedPreTurnHelperEnabled()
