import { createContext, type PropsWithChildren, useContext, useEffect, useState } from 'react'
import { useSidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import { SessionsTreeViewmodel } from './sessions-tree.viewmodel'

const SessionsTreeViewmodelContext = createContext<SessionsTreeViewmodel | null>(null)

export const useSessionsTreeViewmodel = () => {
  const viewmodel = useContext(SessionsTreeViewmodelContext)
  if (!viewmodel) throw new Error('useSessionsTreeViewmodel must be used within SessionsTreeProvider')

  return viewmodel
}

interface SessionsTreeProviderProps extends PropsWithChildren {
  projectId: string
}

export const SessionsTreeProvider = ({ projectId, children }: SessionsTreeProviderProps) => {
  const sidebar = useSidebarViewmodel()
  const [viewmodel, setViewModel] = useState<SessionsTreeViewmodel | null>(null)

  useEffect(() => {
    const viewmodel = new SessionsTreeViewmodel(sidebar, projectId)
    viewmodel.init()
    setViewModel(viewmodel)

    return () => viewmodel.destroy()
  }, [sidebar, projectId])

  if (!viewmodel) return null

  return <SessionsTreeViewmodelContext.Provider value={viewmodel}>{children}</SessionsTreeViewmodelContext.Provider>
}
