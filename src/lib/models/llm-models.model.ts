import { load } from '@tauri-apps/plugin-store'
import { debounce } from 'lodash'
import { flow, makeAutoObservable, observable } from 'mobx'
import type { LlmModel } from '@/bindings'

const BASE_URL = 'https://openrouter.ai/api/v1'

export interface OpenRouterModel {
  id: string
  name: string
  canonical_slug: string
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
  architecture: {
    input_modalities: string[]
    output_modalities: string[]
  }
}

export interface ModelOption {
  model: string
  display_name: string
}

export interface OpenRouterModelsResponse {
  data: OpenRouterModel[]
}

class LlmModelsModel {
  public models: LlmModel[] = observable.array()
  public openrouterModels: OpenRouterModel[] = observable.array()

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  get modelOptions(): ModelOption[] {
    return this.models
      .filter((model) => model.enabled)
      .map((model) => ({
        display_name: model.name,
        model: model.slug,
      }))
  }

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
    const filteredModels = models.filter(
      (model) =>
        model.architecture.input_modalities.includes('text') &&
        model.architecture.output_modalities.includes('text') &&
        model.supported_parameters.includes('tools'),
    )

    this.openrouterModels = filteredModels
  }

  loadModels = async () => {
    const store = await load('imported-models.json')
    const models = await store.get<LlmModel[]>('models')

    if (!models?.length) return

    this.setModels(models)
  }

  setModels = (models: LlmModel[]) => {
    this.models = models
  }

  importModel = flow(function* (this: LlmModelsModel, model: OpenRouterModel) {
    const existingModel = this.models.find((m) => m.slug === model.canonical_slug)
    if (existingModel) return

    const newModel: LlmModel = {
      context_length: model.context_length,
      enabled: true,
      name: model.name,
      provider_slug: null,
      slug: model.canonical_slug,
      supports_reasoning: model.supported_parameters.includes('reasoning'),
    }

    this.models.push(newModel)
    yield this.updateStore()
  })

  toggleModel = flow(function* (this: LlmModelsModel, model: LlmModel) {
    const existingModel = this.models.find((m) => m.slug === model.slug)
    if (!existingModel) return

    existingModel.enabled = !existingModel.enabled

    yield this.updateStore()
  })

  setModelProviderSlug = flow(function* (this: LlmModelsModel, model: LlmModel, providerSlug: string) {
    const existingModel = this.models.find((m) => m.slug === model.slug)
    if (!existingModel) return

    existingModel.provider_slug = providerSlug.trim() ? providerSlug : null
    yield this.updateStore()
  })

  deleteModel = flow(function* (this: LlmModelsModel, model: LlmModel) {
    const nextModels = this.models.filter((existingModel) => existingModel.slug !== model.slug)
    if (nextModels.length === this.models.length) return

    this.models = nextModels
    yield this.updateStore()
  })

  saveCustomModel = flow(function* (this: LlmModelsModel, model: LlmModel) {
    this.models.push(model)
    yield this.updateStore()
  })

  updateStore = debounce(async () => {
    const store = await load('imported-models.json')
    await store.set('models', this.models)
    await store.save()
  }, 1200)
}

export const llmModelsModel = new LlmModelsModel()
