import { useEffect } from 'react'
import { useAdvancedPageViewModel } from './adcanced-page-viewmodel'
import { BunSettings } from './bun-settings'
import { PreTurnHelpers } from './pre-turn-helpers'

export const AdvancedPage = () => {
  const viewmodel = useAdvancedPageViewModel()

  // biome-ignore lint/correctness/useExhaustiveDependencies: run once
  useEffect(() => {
    viewmodel.loadSettings()
    viewmodel.loadBunStatus()
  }, [])

  return (
    <>
      <PreTurnHelpers />
      <BunSettings />
    </>
  )
}
