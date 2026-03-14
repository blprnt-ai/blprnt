import { useEffect, useState } from 'react'
import { SessionPanel } from './session-panel'
import { SessionPanelViewmodel, SessionPanelViewmodelContext } from './session-panel.viewmodel'

interface SessionPanelProviderProps {
  sessionId: string
}

export const SessionPanelProvider = ({ sessionId }: SessionPanelProviderProps) => {
  const [viewModel, setViewModel] = useState<SessionPanelViewmodel | null>(null)

  useEffect(() => {
    const viewModel = new SessionPanelViewmodel(sessionId)
    viewModel.init().then(() => {
      setViewModel(viewModel)
    })

    return () => viewModel.destroy()
  }, [sessionId])

  if (!viewModel) return null

  return (
    <SessionPanelViewmodelContext.Provider value={viewModel}>
      <SessionPanel />
    </SessionPanelViewmodelContext.Provider>
  )
}
