import { createContext, type PropsWithChildren, useContext, useMemo } from 'react'
import { useSidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import type { SessionModel } from '@/lib/models/session.model'
import { SessionTreeViewmodel } from './session-tree.viewmodel'

const SessionTreeViewmodelContext = createContext<SessionTreeViewmodel | null>(null)

export const useSessionTreeViewmodel = () => {
  const viewmodel = useContext(SessionTreeViewmodelContext)
  if (!viewmodel) throw new Error('useSessionTreeViewmodel must be used within SessionTreeProvider')

  return viewmodel
}

interface SessionTreeProviderProps extends PropsWithChildren {
  projectId: string
  session: SessionModel
}

export const SessionTreeProvider = ({ projectId, session, children }: SessionTreeProviderProps) => {
  const sidebar = useSidebarViewmodel()
  const viewmodel = useMemo(() => new SessionTreeViewmodel(sidebar, projectId, session), [sidebar, projectId, session])

  return <SessionTreeViewmodelContext.Provider value={viewmodel}>{children}</SessionTreeViewmodelContext.Provider>
}
