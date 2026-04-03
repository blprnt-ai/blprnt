import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { ProvidersPage } from './providers.page'
import { ProvidersViewmodel, ProvidersViewmodelContext } from './providers.viewmodel'

export const ProvidersProvider = observer(() => {
  const [viewmodel] = useState(() => new ProvidersViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <ProvidersViewmodelContext.Provider value={viewmodel}>
      <ProvidersPage />
    </ProvidersViewmodelContext.Provider>
  )
})
