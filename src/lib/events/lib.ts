import { listen as TAURI_LISTEN, once as TAURI_ONCE } from '@tauri-apps/api/event'
import type { BlprntEventKind, ErrorEvent, McpServerStatus, SessionEvent, TunnelMessage } from '@/bindings'

export enum BlprntEventEnum {
  BackendReady = 'backendReady',
  ReportBugMenuClicked = 'reportBugMenuClicked',
  SessionEvent = 'sessionEvent',
  Error = 'error',
  OAuthCallback = 'oauthCallback',
  TunnelMessage = 'tunnelMessage',
  McpServerStatus = 'mcpServerStatus',
}

interface EventPayload {
  backendReady: null
  reportBugMenuClicked: null
  sessionEvent: SessionEvent
  oauthCallback: string[]
  error: ErrorEvent
  tunnelMessage: TunnelMessage
  mcpServerStatus: McpServerStatus
}

export const listen = <K extends BlprntEventKind>(event: K, callback: (payload: EventPayload[K]) => void) => {
  return TAURI_LISTEN<EventPayload[K]>(event, ({ payload }) => callback(payload))
}

export const once = <K extends BlprntEventKind>(event: K, callback: (payload: EventPayload[K]) => void) => {
  return TAURI_ONCE<EventPayload[K]>(event, ({ payload }) => callback(payload))
}
