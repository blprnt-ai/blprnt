import { useEffect, useState } from 'react'
import { ModelsV2Page } from './models-v2.page'
import { ModelsV2ViewModel, ModelsV2ViewModelContext } from './models-v2.viewmodel'

export const ModelsV2Provider = () => {
  const [viewmodel, setViewmodel] = useState<ModelsV2ViewModel | null>(null)

  useEffect(() => {
    const viewmodel = new ModelsV2ViewModel()
    viewmodel.init().then(() => setViewmodel(viewmodel))
  }, [])

  if (!viewmodel) return null

  return (
    <ModelsV2ViewModelContext.Provider value={viewmodel}>
      <ModelsV2Page />
    </ModelsV2ViewModelContext.Provider>
  )
}
