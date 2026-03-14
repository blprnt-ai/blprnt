import { createContext, useContext } from 'react'
import type { AppViewModel } from '@/app.viewmodel'

export const AppViewModelContext = createContext<AppViewModel | undefined>(undefined)

export const useAppViewModel = () => {
  const context = useContext(AppViewModelContext)
  if (!context) throw new Error('useAppViewModel must be used within a AppViewModelProvider')
  return context
}

export const useAppState = () => useAppViewModel().state
export const useIsLoading = () => useAppViewModel().isLoading
export const useIsFirstLoad = () => useAppViewModel().isFirstLoad
