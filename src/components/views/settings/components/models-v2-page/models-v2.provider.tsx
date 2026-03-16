import { useEffect, useState } from 'react'
import { ModelsV2Page } from './models-v2.page'
import { ModelsV2ViewModel, ModelsV2ViewModelContext } from './models-v2.viewmodel'

export const ModelsV2Provider = () => {
  const [viewmodel] = useState(() => new ModelsV2ViewModel())

  // biome-ignore lint/correctness/useExhaustiveDependencies: Only run on first render
  useEffect(() => {
    void viewmodel.init()
  }, [])

  return (
    <ModelsV2ViewModelContext.Provider value={viewmodel}>
      <ModelsV2Page />
    </ModelsV2ViewModelContext.Provider>
  )
}
