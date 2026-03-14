import { useEffect } from 'react'
import { CustomProviders } from './custom-providers'
import { FnfProviders } from './fnf-providers'
import { useProvidersPageViewmodel } from './providers-page-viewmodel'

export const ProvidersPage = () => {
  const viewmodel = useProvidersPageViewmodel()

  // biome-ignore lint/correctness/useExhaustiveDependencies: run once
  useEffect(() => {
    viewmodel.refreshProviders()
  }, [])

  return (
    <>
      <FnfProviders />
      <CustomProviders />
    </>
  )
}
