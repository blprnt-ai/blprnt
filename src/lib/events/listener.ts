import type { ControlEvent, LlmEvent, McpServerStatus, PromptEvent, SessionEvent, SignalEvent } from '@/bindings'
import { BlprntEventEnum, listen } from '@/lib/events/lib'
import { EventType, globalEventBus } from './event-bus'

let isListening = false

export const startEventBusListeners = async () => {
  if (isListening) return
  isListening = true

  const unlistenSession = await listen(BlprntEventEnum.SessionEvent, (payload: SessionEvent) => {
    globalEventBus.emit(EventType.SessionEvent, payload)

    switch (payload.eventData.eventType) {
      case 'control':
        console.log('control', payload.eventData)
        globalEventBus.emit(EventType.SessionControl, {
          event: payload.eventData as ControlEvent,
          sessionId: payload.sessionId,
        })
        break
      case 'llm':
        globalEventBus.emit(EventType.SessionLlm, {
          event: payload.eventData as LlmEvent,
          sessionId: payload.sessionId,
        })
        break
      case 'prompt':
        globalEventBus.emit(EventType.SessionPrompt, {
          event: payload.eventData as PromptEvent,
          sessionId: payload.sessionId,
        })
        break
      case 'signal':
        globalEventBus.emit(EventType.SessionSignal, {
          event: payload.eventData as SignalEvent,
          sessionId: payload.sessionId,
        })
        break
    }
  })

  const unlistenError = await listen(BlprntEventEnum.Error, (payload) => {
    globalEventBus.emit(EventType.Error, payload)
  })

  const unlistenBackendReady = await listen(BlprntEventEnum.BackendReady, () => {
    globalEventBus.emit(EventType.BackendReady, null)
  })

  const unlistenOAuth = await listen(BlprntEventEnum.OAuthCallback, (payload) => {
    globalEventBus.emit(EventType.OAuthCallback, payload)
  })

  const unlistenReportBugMenuClicked = await listen(BlprntEventEnum.ReportBugMenuClicked, () => {
    globalEventBus.emit(EventType.ReportBugMenuClicked, null)
  })

  const unlistenTunnelMessage = await listen(BlprntEventEnum.TunnelMessage, (payload) => {
    globalEventBus.emit(EventType.TunnelMessage, payload)
  })

  const unlistenMcpServerStatus = await listen(BlprntEventEnum.McpServerStatus, (payload: McpServerStatus) => {
    globalEventBus.emit(EventType.Internal, {
      event: {
        serverId: payload.server_id,
        type: 'mcp_server_status_changed',
      },
    })
  })

  return () => {
    unlistenSession()
    unlistenError()
    unlistenBackendReady()
    unlistenOAuth()
    unlistenReportBugMenuClicked()
    unlistenTunnelMessage()
    unlistenMcpServerStatus()
  }
}
