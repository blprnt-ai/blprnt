import { makeAutoObservable } from 'mobx'
import type { QueueMode, SessionCreateParams } from '@/bindings'
import type { SessionModel } from '@/lib/models/session.model'
import { newModelOverride } from '@/lib/utils/default-models'

export class SessionFormViewModel {
  constructor(public readonly model: SessionModel) {
    makeAutoObservable(this, { model: false }, { autoBind: true })
  }

  get name() {
    return this.model.name
  }

  setName = (name: string) => {
    this.model.name = name
  }

  get modelOverride() {
    return this.model.modelOverride
  }

  setModelOverride = (modelOverride: string) => {
    this.model.modelOverride = modelOverride
  }

  get webSearchEnabled() {
    return this.model.webSearchEnabled ?? false
  }

  setWebSearchEnabled = (webSearchEnabled: boolean) => {
    this.model.webSearchEnabled = webSearchEnabled
  }

  get personalityId() {
    return this.model.personalityId ?? ''
  }

  setPersonalityId = (personalityId: string) => {
    this.model.personalityId = personalityId || null
  }

  get queueMode() {
    return this.model.queueMode ?? 'queue'
  }

  setQueueMode = (queueMode: string) => {
    this.model.queueMode = (queueMode as QueueMode) || null
  }

  get yolo() {
    return this.model.yolo
  }

  setYolo = (yolo: boolean) => {
    this.model.yolo = yolo
  }

  get networkAccess() {
    return this.model.networkAccess
  }

  setNetworkAccess = (networkAccess: boolean) => {
    this.model.networkAccess = networkAccess
  }

  get readOnly() {
    return this.model.readOnly
  }

  setReadOnly = (readOnly: boolean) => {
    this.model.readOnly = readOnly
  }

  get isValid() {
    return this.name.trim() !== '' && this.model.modelOverride !== newModelOverride
  }

  toCreateParams = (projectId: string): SessionCreateParams => {
    return {
      agent_kind: this.model.agentKind,
      description: '',
      model_override: this.model.modelOverride,
      name: this.model.name,
      network_access: this.model.networkAccess,
      personality_key: this.model.personalityId,
      project_id: projectId,
      queue_mode: this.model.queueMode ?? 'queue',
      read_only: this.model.readOnly,
      reasoning_effort: this.model.reasoningEffort ?? 'medium',
      web_search_enabled: this.webSearchEnabled,
      yolo: this.model.yolo,
    }
  }
}
