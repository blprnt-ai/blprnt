import { useEffect, useMemo, useState } from 'react'
import type { ModelCatalogItem } from '@/lib/models/app.model'
import { enabledModelsModel } from '@/lib/models/enabled-models.model'
import { ModelsCatalogModel } from '@/lib/models/models-catalog.model'
import { useAppViewModel } from './use-app-viewmodel'

interface UseLlmModelsResult {
  allModels: ModelCatalogItem[]
  freeModels: ModelCatalogItem[]
  enabledModels: ModelCatalogItem[]
  openRouterModels: Record<string, ModelCatalogItem[]>
  toggleSlug: (slug: string) => void
  hasCodex: boolean
  hasClaude: boolean
}

export const useLlmModels = (): UseLlmModelsResult => {
  const [models, setModels] = useState<ModelCatalogItem[]>([])
  const enabledModelSlugs = enabledModelsModel.slugs
  const isLoaded = enabledModelsModel.isLoaded
  const allModels = useMemo(() => models.filter((m) => m.enabled).toSorted(sortModels), [models])
  const providers = useAppViewModel().providers

  const hasCodex = providers.some((p) => p.provider === 'openai_fnf')
  const hasClaude = providers.some((p) => p.provider === 'anthropic_fnf')
  const hasOpenai = providers.some((p) => p.provider === 'openai')
  const hasAnthropic = providers.some((p) => p.provider === 'anthropic')
  const hasOpenRouter = providers.some((p) => p.provider === 'open_router')

  useEffect(() => {
    let isMounted = true
    ModelsCatalogModel.list()
      .then((catalog) => {
        if (!isMounted) return
        setModels(catalog)
      })
      .catch((error) => {
        console.error('Error loading models catalog', error)
      })

    return () => {
      isMounted = false
    }
  }, [])

  // biome-ignore lint/correctness/useExhaustiveDependencies: don't depend on appViewmodel.isFreeUser
  const freeModels = useMemo(
    () =>
      allModels.filter((model) => {
        if (hasOpenRouter) return true

        // if (model.provider === 'openai') {
        //   console.log('openai', model.slug, hasCodex, hasOpenai)
        // } else if (model.provider === 'anthropic') {
        //   console.log('anthropic', model.slug, hasClaude, hasAnthropic)
        // }

        if (model.provider === 'openai' && (hasCodex || hasOpenai)) return true
        if (model.provider === 'anthropic' && (hasClaude || hasAnthropic)) return true

        return false
      }),
    [allModels, hasCodex, hasClaude],
  )

  const enabledModels = useMemo(
    () =>
      freeModels
        .filter((model) => model.enabled)
        .map((model) => ({
          ...model,
          toggledOn: enabledModelSlugs.includes(model.slug),
        })),
    [freeModels, enabledModelSlugs],
  )

  // biome-ignore lint/correctness/useExhaustiveDependencies: don't depend on appViewmodel.isFreeUser
  const openRouterModels = useMemo(
    () =>
      allModels
        .filter(
          (model) =>
            !(
              hasOpenRouter ||
              (model.provider === 'openai' && (hasCodex || hasOpenai)) ||
              (model.provider === 'anthropic' && (hasClaude || hasAnthropic))
            ),
        )
        .reduce(
          (acc, model) => {
            acc[model.provider] = [...(acc[model.provider] || []), model]
            return acc
          },
          {} as Record<string, ModelCatalogItem[]>,
        ),
    [allModels],
  )

  if (!isLoaded)
    return {
      allModels: [],
      enabledModels: [],
      freeModels: [],
      hasClaude: false,
      hasCodex: false,
      openRouterModels: {},
      toggleSlug: () => {},
    }

  return {
    allModels,
    enabledModels,
    freeModels,
    hasClaude,
    hasCodex,
    openRouterModels,
    toggleSlug: enabledModelsModel.toggleSlug,
  }
}

const sortModels = (a: ModelCatalogItem, b: ModelCatalogItem) => {
  return a.name.localeCompare(b.name)
}
