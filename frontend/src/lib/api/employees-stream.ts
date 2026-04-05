import type { EmployeeStreamMessageDto } from '@/bindings/EmployeeStreamMessageDto'
import { buildWebSocketUrl } from './url'

export interface EmployeesStreamHandlers {
  onMessage: (message: EmployeeStreamMessageDto) => void
  onOpen?: () => void
  onError?: () => void
  onClose?: () => void
}

export function connectEmployeesStream(employeeId: string, handlers: EmployeesStreamHandlers) {
  const socket = new WebSocket(buildWebSocketUrl('/employees/stream', { employee_id: employeeId }))

  socket.onopen = () => {
    handlers.onOpen?.()
  }

  socket.onmessage = (event) => {
    handlers.onMessage(JSON.parse(event.data) as EmployeeStreamMessageDto)
  }

  socket.onerror = () => {
    handlers.onError?.()
  }

  socket.onclose = () => {
    handlers.onClose?.()
  }

  return socket
}
