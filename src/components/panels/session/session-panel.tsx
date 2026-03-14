import { SessionPanelLayout } from '@/components/panels/session/organisms/session-panel-layout'
import { useSessionPanelViewmodel } from './session-panel.viewmodel'

export const SessionPanel = () => {
  const viewmodel = useSessionPanelViewmodel()

  if (!viewmodel.session) return null

  return <SessionPanelLayout />
}
