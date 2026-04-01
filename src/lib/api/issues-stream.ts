import type { IssueStreamMessageDto } from '@/bindings/IssueStreamMessageDto'
import { buildWebSocketUrl } from './url'

export interface IssueStreamHandlers {
  onMessage: (message: IssueStreamMessageDto) => void
  onOpen?: () => void
  onError?: () => void
  onClose?: () => void
}

export function connectIssueStream(employeeId: string, handlers: IssueStreamHandlers) {
  const socket = new WebSocket(buildWebSocketUrl('/issues/stream', { employee_id: employeeId }))

  socket.onopen = () => {
    handlers.onOpen?.()
  }

  socket.onmessage = (event) => {
    handlers.onMessage(JSON.parse(event.data) as IssueStreamMessageDto)
  }

  socket.onerror = () => {
    handlers.onError?.()
  }

  socket.onclose = () => {
    handlers.onClose?.()
  }

  return socket
}
