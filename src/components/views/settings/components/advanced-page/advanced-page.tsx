import { useEffect } from 'react'
import { useAdvancedPageViewModel } from './adcanced-page-viewmodel'
import { PreTurnHelpers } from './pre-turn-helpers'
import { RuntimeSettings } from './runtime-settings'

export const AdvancedPage = () => {
  const viewmodel = useAdvancedPageViewModel()

  // biome-ignore lint/correctness/useExhaustiveDependencies: run once
  useEffect(() => {
    viewmodel.loadSettings()
    viewmodel.loadJsRuntimeHealth()
  }, [])

  return (
    <>
      <PreTurnHelpers />
      <RuntimeSettings />
    </>
  )
}
