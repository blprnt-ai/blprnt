import { tauriProvidersApi } from '@/lib/api/tauri/providers.api'
import type { ModelCatalogItem } from '@/lib/models/app.model'

let modelsCache: ModelCatalogItem[] | null = null
let lastPullTimestamp: number | null = null

const CACHE_DURATION = 1000 * 60 * 15 // 15 minutes

export const ModelsCatalogModel = {
  list: async () => {
    if (modelsCache && lastPullTimestamp && Date.now() - lastPullTimestamp < CACHE_DURATION) {
      return modelsCache
    }

    const result = await tauriProvidersApi.getModels()
    const catalog: ModelCatalogItem[] = result.map((model) => {
      const stripFreeName = model.name.replace('(free)', '').trim()
      const nameParts = stripFreeName.split(':')
      const name = nameParts.length === 1 ? nameParts[0] : nameParts[1]
      const provider = model.slug.split('/')[0].trim()

      return { ...model, name, provider, toggledOn: false }
    })

    modelsCache = catalog
    lastPullTimestamp = Date.now()

    return catalog
  },
}
