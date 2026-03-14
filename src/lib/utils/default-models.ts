import type { ProjectRecord, ProviderRecord, SessionRecord } from '@/bindings'

export const newProviderId = 'NEW_PROVIDER_ID'
export const newPersonalityId = 'NEW_PERSONALITY_ID'
export const newProjectId = 'NEW_PROJECT_ID'
export const newSessionId = 'NEW_SESSION_ID'
export const newModelOverride = 'NEW_MODEL_OVERRIDE'

export const defaultProviderModel: ProviderRecord = {
  base_url: null,
  created_at: Date.now(),
  id: newProviderId,
  provider: 'mock',
  updated_at: Date.now(),
}

export const defaultProjectModel: ProjectRecord = {
  agent_primer: null,
  created_at: Date.now(),
  id: newProjectId,
  name: '',
  updated_at: Date.now(),
  working_directories: [],
}

export const defaultPersonalityModel = {
  created_at: new Date().toISOString(),
  description: '',
  id: newPersonalityId,
  is_default: false,
  is_user_defined: true,
  name: '',
  system_prompt: '',
  updated_at: new Date().toISOString(),
}

export const defaultSessionModel: SessionRecord = {
  agent_kind: 'crew',
  created_at: Date.now(),
  description: '',
  id: newSessionId,
  model_override: newModelOverride,
  name: '',
  network_access: true,
  parent_id: null,
  personality_key: null,
  project: '',
  queue_mode: 'queue',
  read_only: false,
  reasoning_effort: 'medium',
  token_usage: 0,
  updated_at: Date.now(),
  yolo: false,
}
