import { useLlmModels } from '@/hooks/use-llm-models'
import { EnabledModelsTable } from './enabled-models-table'
import { PlanGroupSections } from './plan-group-sections'

export const ModelsPage = () => {
  const { enabledModels, openRouterModels, toggleSlug } = useLlmModels()

  return (
    <>
      <EnabledModelsTable llmModels={enabledModels} toggleSlug={toggleSlug} />

      <PlanGroupSections openRouterModels={openRouterModels} />
    </>
  )
}
