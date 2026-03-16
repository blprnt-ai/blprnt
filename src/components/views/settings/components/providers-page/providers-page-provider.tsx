import { useState } from 'react'
import { ProvidersPage } from './providers-page'
import { ProvidersPageViewmodel, ProvidersPageViewmodelContext } from './providers-page-viewmodel'

export const ProvidersPageProvider = () => {
  const [viewmodel] = useState(() => new ProvidersPageViewmodel())

  return (
    <ProvidersPageViewmodelContext.Provider value={viewmodel}>
      <ProvidersPage />
    </ProvidersPageViewmodelContext.Provider>
  )
}
