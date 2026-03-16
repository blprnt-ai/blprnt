import { load } from '@tauri-apps/plugin-store'
import { debounce } from 'lodash'
import { flow, makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'

const BASE_URL = 'https://openrouter.ai/api/v1'

export interface OpenRouterModel {
  id: string
  name: string
  cannonical_slug: string
  created: number
  pricing: {
    prompt: string
    completion: string
    request: string
    image: string
  }
  context_length: number
  description: string
  supported_parameters: string[]
}

export interface OpenRouterModelsResponse {
  data: OpenRouterModel[]
}

export interface BlprntModel extends OpenRouterModel {
  provider_slug: string | null
  enabled: boolean
}

export class ModelsV2ViewModel {
  openRouterModels: OpenRouterModel[] = []
  blprntModels: BlprntModel[] = []

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = flow(function* (this: ModelsV2ViewModel) {
    yield this.loadModelsFromOpenRouter()
    yield this.loadModels()
  })

  loadModelsFromOpenRouter = async () => {
    try {
      const response = await fetch(`${BASE_URL}/models`)
      const data = (await response.json()) as OpenRouterModelsResponse

      this.setOpenRouterModels(data.data)
    } catch (error) {
      console.error(error)
      return []
    }
  }

  setOpenRouterModels = (models: OpenRouterModel[]) => {
    console.log('setOpenRouterModels', models)
    this.openRouterModels = models
  }

  loadModels = async () => {
    const store = await load('imported-models.json')
    const models = await store.get<BlprntModel[]>('models')

    if (!models?.length) return

    this.setBlprntModels(models)
  }

  setBlprntModels = (models: BlprntModel[]) => {
    this.blprntModels = models
  }

  importModel = flow(function* (this: ModelsV2ViewModel, model: OpenRouterModel) {
    const existingModel = this.blprntModels.find((m) => m.id === model.id)
    if (existingModel) return

    const newModel: BlprntModel = {
      ...model,
      enabled: true,
      provider_slug: null,
    }

    this.blprntModels.push(newModel)
    yield this.updateStore()
  })

  toggleModel = flow(function* (this: ModelsV2ViewModel, model: BlprntModel) {
    const existingModel = this.blprntModels.find((m) => m.id === model.id)
    if (!existingModel) return

    existingModel.enabled = !existingModel.enabled

    yield this.updateStore()
  })

  setModelProviderSlug = flow(function* (this: ModelsV2ViewModel, model: BlprntModel, providerSlug: string) {
    const existingModel = this.blprntModels.find((m) => m.id === model.id)
    if (!existingModel) return

    existingModel.provider_slug = providerSlug
    yield this.updateStore()
  })

  updateStore = debounce(async () => {
    const store = await load('imported-models.json')
    await store.set('models', this.blprntModels)
  }, 1200)
}

export const ModelsV2ViewModelContext = createContext<ModelsV2ViewModel | null>(null)

export const useModelsV2ViewModel = () => {
  const viewmodel = useContext(ModelsV2ViewModelContext)
  if (!viewmodel) throw new Error('useModelsV2ViewModel must be used within ModelsV2ViewModelContext')
  return viewmodel
}
