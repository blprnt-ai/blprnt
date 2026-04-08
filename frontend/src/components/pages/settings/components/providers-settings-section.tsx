import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { AppLoader } from '@/components/organisms/app-loader'
import { ProvidersContent } from '@/components/pages/providers/providers.content'
import { ProvidersViewmodel, ProvidersViewmodelContext } from '@/components/pages/providers/providers.viewmodel'

export const ProvidersSettingsSection = observer(() => {
  const [viewmodel] = useState(() => new ProvidersViewmodel())

  useEffect(() => {
    void viewmodel.init()
  }, [viewmodel])

  if (viewmodel.isLoading) return <AppLoader />

  return (
    <ProvidersViewmodelContext.Provider value={viewmodel}>
      <ProvidersContent />
    </ProvidersViewmodelContext.Provider>
  )
})
