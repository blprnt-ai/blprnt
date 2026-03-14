import { flow, makeAutoObservable } from 'mobx'
import type {
  AgentKind,
  ControlEvent,
  DeleteQueuedPromptOutcome,
  LlmEvent,
  PersonalityModelDto,
  QueueMode,
  ReasoningEffort,
  ReasoningEffortChanged,
  SessionCreateParams,
  SessionPatchV2,
  SessionPlan,
  SessionRecord,
  SessionRecordDto,
  Status,
  TokenUsage,
} from '@/bindings'
import { tauriPersonalitiesApi } from '@/lib/api/tauri/personalities.api'
import { tauriSessionApi } from '@/lib/api/tauri/session.api'
import { EventType, globalEventBus } from '@/lib/events'
import type { ModelOverrideChangedEvent } from '@/lib/events/event-bus'

export enum RunningState {
  Idle = 'idle',
  Running = 'running',
}

export class SessionModel {
  private static registry = new Map<string, SessionModel>()
  private static subscribed = false

  public id: string
  public parentId: string | null | undefined
  public name: string
  public agentKind: AgentKind
  public yolo: boolean
  public readOnly: boolean
  public networkAccess: boolean
  public reasoningEffort: ReasoningEffort | null = null
  public tokenUsage: number
  public modelOverride: string
  public webSearchEnabled: boolean | null
  public persistedWebSearchEnabled: boolean | null
  public queueMode: QueueMode | null
  public createdAt: number
  public updatedAt: number
  public projectId: string | null
  public personalityId: string | null
  public plan: SessionPlan | null = null

  private runningState: RunningState
  public status: string

  private unsubscribers: Array<() => void> = []

  constructor(model: SessionRecordDto) {
    const personalityKey = (model as SessionRecordDto & { personality_key?: string | null }).personality_key ?? null
    this.id = model.id
    this.parentId = model.parent_id ?? null
    this.name = model.name
    this.agentKind = model.agent_kind
    this.yolo = model.yolo
    this.readOnly = model.read_only
    this.networkAccess = model.network_access
    this.tokenUsage = model.token_usage
    this.reasoningEffort = model.reasoning_effort ?? null
    this.modelOverride = model.model_override
    this.persistedWebSearchEnabled = model.web_search_enabled ?? null
    this.webSearchEnabled = model.web_search_enabled ?? null
    this.queueMode = model.queue_mode ?? null
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
    this.projectId = model.project ?? null
    this.personalityId = personalityKey
    this.runningState = RunningState.Idle
    this.plan = model.plan ?? null
    this.status = ''
    this.runningState = model.status === 'Running' ? RunningState.Running : RunningState.Idle

    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = () => {
    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.Internal, (event) => {
        if (event.payload.event.type === 'model_override_changed' && event.payload.event.sessionId === this.id) {
          this.handleModelOverrideChanged(event.payload.event)
        }
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionControl, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handleControlEvent(event.payload.event)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionLlm, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handleLlmEvent(event.payload.event)
      }),
    )
  }

  destroy = () => {
    this.unsubscribers.forEach((unsubscribe) => unsubscribe())
    this.unsubscribers = []
  }

  updateFrom = (model: SessionRecord) => {
    const personalityKey = (model as SessionRecord & { personality_key?: string | null }).personality_key ?? null
    this.name = model.name
    this.agentKind = model.agent_kind
    this.yolo = model.yolo
    this.readOnly = model.read_only
    this.networkAccess = model.network_access
    this.tokenUsage = model.token_usage
    this.reasoningEffort = model.reasoning_effort ?? this.reasoningEffort
    this.modelOverride = model.model_override
    this.persistedWebSearchEnabled = model.web_search_enabled ?? null
    this.webSearchEnabled = model.web_search_enabled ?? null
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at
    this.projectId = model.project ?? null
    this.personalityId = personalityKey
  }

  private setRunningState = (state: RunningState) => {
    this.runningState = state
  }

  private static ensureSubscribed = () => {
    if (SessionModel.subscribed) return
    SessionModel.subscribed = true

    globalEventBus.subscribe(EventType.Internal, (event) => {
      const internal = event.payload.event
      if (internal.type === 'session_removed') {
        SessionModel.registry.delete(internal.sessionId)
      }
    })
  }

  static getOrCreate = (model: SessionRecordDto) => {
    SessionModel.ensureSubscribed()
    const existing = SessionModel.registry.get(model.id)
    if (existing) {
      existing.updateFrom(model)
      return existing
    }

    const instance = new SessionModel(model)
    SessionModel.registry.set(model.id, instance)
    instance.init()

    return instance
  }

  static get = async (sessionId: string) => {
    const result = await tauriSessionApi.start(sessionId)

    return SessionModel.getOrCreate(result)
  }

  static list = async (projectId: string) => {
    const result = await tauriSessionApi.list(projectId)

    return result.map((session) =>
      SessionModel.getOrCreate({ ...session, queue_mode: session.queue_mode ?? 'queue', status: 'Idle' }),
    )
  }

  static stopById = async (sessionId: string) => {
    await tauriSessionApi.stop(sessionId)
  }

  static deleteById = async (sessionId: string) => {
    await tauriSessionApi.delete(sessionId)
  }

  static listPersonalities = async (): Promise<PersonalityModelDto[]> => {
    return tauriPersonalitiesApi.list()
  }

  static create = async (params: SessionCreateParams) => {
    const result = await tauriSessionApi.create(params)

    return SessionModel.getOrCreate({ ...result, queue_mode: params.queue_mode ?? 'queue', status: 'Idle' })
  }

  get isSubagent() {
    return this.parentId !== null
  }

  get isRunning() {
    return this.runningState === RunningState.Running
  }

  setRunning = (state: boolean) => {
    this.runningState = state ? RunningState.Running : RunningState.Idle
  }

  update = async (patch: SessionPatchV2) => {
    const result = await tauriSessionApi.update(this.id, patch)
    this.updateFrom(result)

    return this
  }

  start = async () => {
    const result = await tauriSessionApi.start(this.id)
    this.updateFrom(result)
    return this
  }

  delete = async () => {
    await tauriSessionApi.delete(this.id)
  }

  stop = async () => {
    await tauriSessionApi.stop(this.id)
  }

  sendPrompt = async (prompt: string, imageUrls: string[] | null) => {
    this.setRunningState(RunningState.Running)
    await tauriSessionApi.sendPrompt(this.id, prompt, imageUrls)
  }

  deleteQueuedPrompt = async (queueItemId: string): Promise<DeleteQueuedPromptOutcome> => {
    return tauriSessionApi.deleteQueuedPrompt({
      queue_item_id: queueItemId,
      session_id: this.id,
    })
  }

  rewindTo = async (historyId: string) => {
    await tauriSessionApi.rewindTo(this.id, historyId)
  }

  sendInterrupt = flow(function* (this: SessionModel) {
    this.setRunningState(RunningState.Idle)
    yield tauriSessionApi.sendInterrupt(this.id)
  })

  submitAnswer = async (questionId: string, answer: string) => {
    return tauriSessionApi.submitAnswer(this.id, questionId, answer)
  }

  getTerminalSnapshot = async (terminalId: string) => {
    return tauriSessionApi.getTerminalSnapshot(this.id, terminalId)
  }

  closeTerminal = async (terminalId: string) => {
    await tauriSessionApi.closeTerminal(this.id, terminalId)
  }

  private handleControlEvent = (event: ControlEvent) => {
    switch (event.type) {
      case 'turnStart':
        this.setRunningState(RunningState.Running)
        break
      case 'turnStop': {
        this.setRunningState(RunningState.Idle)
        if (this.status) this.status = ''

        break
      }
      case 'compactingStart':
        break
      case 'compactingDone':
        break
    }
  }

  private handleLlmEvent = (event: LlmEvent) => {
    switch (event.type) {
      case 'status':
        this.handleStatus(event as Status)
        break
      case 'tokenUsage':
        this.handleTokenUsage(event as TokenUsage)
        break
      case 'reasoningEffortChanged':
        this.handleReasoningEffortChanged(event as ReasoningEffortChanged)
        break
    }
  }

  private handleStatus = (payload: Status) => {
    this.status = payload.status
  }

  private handleTokenUsage = (payload: TokenUsage) => {
    this.tokenUsage = payload.inputTokens + payload.outputTokens
  }

  private handleReasoningEffortChanged = (payload: ReasoningEffortChanged) => {
    this.reasoningEffort = payload.effort
  }

  private handleModelOverrideChanged = (payload: ModelOverrideChangedEvent) => {
    this.modelOverride = payload.modelOverride
  }
}
