import { createContext, useContext, useEffect, useState } from 'react'
import { AppModel } from '@/models/app.model'

export const AppModelContext = createContext<AppModel | null>(null)

export const AppModelProvider = ({ children }: { children: React.ReactNode }) => {
  const [appModel] = useState(() => new AppModel())

  // biome-ignore lint/correctness/useExhaustiveDependencies: AppModel is a mobx store
  useEffect(() => {
    appModel.init()
  }, [])

  return <AppModelContext.Provider value={appModel}>{children}</AppModelContext.Provider>
}

export const useAppModel = () => {
  const appModel = useContext(AppModelContext)
  if (!appModel) throw new Error('AppModel not found')

  return appModel
}
