import { SessionViewContext } from '@/hooks/use-session-view-context'

interface SessionViewProviderProps {
  sessionId: string
  children: React.ReactNode
}

export const SessionViewProvider = ({ sessionId, children }: SessionViewProviderProps) => {
  return <SessionViewContext.Provider value={{ sessionId }}>{children}</SessionViewContext.Provider>
}
