import type { ReactNode } from 'react'
import { useMemo } from 'react'
import { AppViewModel } from '@/app.viewmodel'
import { AppViewModelContext } from '@/hooks/use-app-viewmodel'

interface AppViewModelProviderProps {
  children: ReactNode
}

export const AppViewModelProvider = ({ children }: AppViewModelProviderProps) => {
  const appViewModel = useMemo(() => new AppViewModel(), [])
  return <AppViewModelContext.Provider value={appViewModel}>{children}</AppViewModelContext.Provider>
}
