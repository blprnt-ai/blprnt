import { useSessionViewContext } from './use-session-view-context'

export const useSessionViewSessionId = () => {
  const context = useSessionViewContext()
  if (!context) throw new Error('SessionContext is not available')

  return context.sessionId
}
