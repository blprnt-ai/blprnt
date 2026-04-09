import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { DashboardPage } from './dashboard.page'
import { DashboardViewmodel, DashboardViewmodelContext } from './dashboard.viewmodel'

export const DashboardProvider = observer(() => {
  const [viewmodel, setViewmodel] = useState(() => new DashboardViewmodel())

  useEffect(() => {
    const nextViewmodel = new DashboardViewmodel()
    setViewmodel(nextViewmodel)
    void nextViewmodel.init()

    return () => {
      nextViewmodel.destroy()
    }
  }, [])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <DashboardViewmodelContext.Provider value={viewmodel}>
      <DashboardPage />
    </DashboardViewmodelContext.Provider>
  )
})
