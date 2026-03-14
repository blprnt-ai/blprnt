import { createContext, useContext } from 'react'

interface SessionViewContext {
  sessionId: string
}

export const SessionViewContext = createContext<SessionViewContext | null>(null)

export const useSessionViewContext = () => {
  const context = useContext(SessionViewContext)
  if (!context) throw new Error('SessionContext is not available')

  return context
}
