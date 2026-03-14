import type {
  ControlEvent,
  ErrorEvent,
  LlmEvent,
  PromptEvent,
  SessionEvent,
  SignalEvent,
  TunnelMessage,
} from '@/bindings'

export enum EventType {
  Any = 'any',
  BackendReady = 'backend_ready',
  Error = 'error',
  OAuthCallback = 'oauth_callback',
  ReportBugMenuClicked = 'report_bug_menu_clicked',
  TunnelMessage = 'tunnel_message',
  SessionEvent = 'session_event',
  SessionControl = 'session_control',
  SessionLlm = 'session_llm',
  SessionPrompt = 'session_prompt',
  SessionSignal = 'session_signal',
  Internal = 'session_internal',
}

export interface ProjectEvent {
  projectId: string
}

export interface SessionItemEvent {
  sessionId: string
}

export interface ModelOverrideChangedEvent {
  sessionId: string
  modelOverride: string
}

export interface PlanCompletedEvent {
  planId: string
  sessionId: string
}

export interface PlanEvent {
  planId: string
  projectId: string | null
}

export interface McpServerEvent {
  serverId: string
}

export type InternalEvent =
  | ({ type: 'project_added' | 'project_removed' | 'project_updated' } & ProjectEvent)
  | ({ type: 'session_added' | 'session_removed' | 'session_updated' } & SessionItemEvent)
  | ({ type: 'model_override_changed' } & ModelOverrideChangedEvent)
  | ({ type: 'plan_updated' } & PlanEvent)
  | ({ type: 'plan_completed' } & PlanCompletedEvent)
  | ({
      type: 'mcp_server_added' | 'mcp_server_updated' | 'mcp_server_removed' | 'mcp_server_status_changed'
    } & McpServerEvent)

export type EventPayloadMap = {
  [EventType.Any]: unknown
  [EventType.BackendReady]: null
  [EventType.Error]: ErrorEvent
  [EventType.OAuthCallback]: string[]
  [EventType.ReportBugMenuClicked]: null
  [EventType.SessionEvent]: SessionEvent
  [EventType.SessionControl]: { sessionId: string; event: ControlEvent }
  [EventType.SessionLlm]: { sessionId: string; event: LlmEvent }
  [EventType.SessionPrompt]: { sessionId: string; event: PromptEvent }
  [EventType.SessionSignal]: { sessionId: string; event: SignalEvent }
  [EventType.TunnelMessage]: TunnelMessage
  [EventType.Internal]: { event: InternalEvent }
}

export interface EventEnvelope<TType extends EventType = EventType> {
  type: TType
  payload: EventPayloadMap[TType]
  timestamp: number
}

export type EventHandler<TType extends EventType = EventType> = (event: EventEnvelope<TType>) => void
export type EventPredicate<TType extends EventType = EventType> = (event: EventEnvelope<TType>) => boolean

interface SubscriptionInternal {
  handler: EventHandler
  predicate?: EventPredicate
}

export class EventBus {
  private readonly subscriptions = new Map<EventType, Set<SubscriptionInternal>>()

  subscribe<TType extends EventType>(type: TType, handler: EventHandler<TType>, predicate?: EventPredicate<TType>) {
    const subscription: SubscriptionInternal = {
      handler: handler as EventHandler,
      predicate: predicate as EventPredicate | undefined,
    }
    const existing = this.subscriptions.get(type)
    if (existing) {
      existing.add(subscription)
    } else {
      this.subscriptions.set(type, new Set([subscription]))
    }

    return () => {
      const current = this.subscriptions.get(type)
      if (!current) return
      current.delete(subscription)
      if (current.size === 0) this.subscriptions.delete(type)
    }
  }

  emit<TType extends EventType>(type: TType, payload: EventPayloadMap[TType]) {
    const event: EventEnvelope<TType> = { payload, timestamp: Date.now(), type }
    const targeted = this.subscriptions.get(type)
    const all = this.subscriptions.get(EventType.Any)

    this.dispatch(targeted, event)
    this.dispatch(all, event)
  }

  clear() {
    this.subscriptions.clear()
  }

  private dispatch(subs: Set<SubscriptionInternal> | undefined, event: EventEnvelope) {
    if (!subs?.size) return
    subs.forEach((subscription) => {
      if (subscription.predicate && !subscription.predicate(event)) return
      subscription.handler(event)
    })
  }
}

export const globalEventBus = new EventBus()
