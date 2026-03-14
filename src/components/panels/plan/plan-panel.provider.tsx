import { useEffect, useState } from 'react'
import { PlanPanel } from './plan-panel'
import { PlanPanelViewmodel, PlanPanelViewmodelContext } from './plan-panel.viewmodel'

interface PlanPanelProviderProps {
  projectId: string
  planId: string
}

export const PlanPanelProvider = ({ projectId, planId }: PlanPanelProviderProps) => {
  const [viewModel, setViewModel] = useState<PlanPanelViewmodel | null>(null)

  useEffect(() => {
    const viewModel = new PlanPanelViewmodel(projectId, planId)
    viewModel.init().then(() => {
      setViewModel(viewModel)
    })

    return () => viewModel.destroy()
  }, [planId, projectId])

  if (!viewModel) return null

  return (
    <PlanPanelViewmodelContext.Provider value={viewModel}>
      <PlanPanel />
    </PlanPanelViewmodelContext.Provider>
  )
}
