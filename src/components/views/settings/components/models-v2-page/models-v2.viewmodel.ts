import { load } from '@tauri-apps/plugin-store'
import { debounce, random } from 'lodash'
import { flow, makeAutoObservable, observable } from 'mobx'
import { createContext, useContext } from 'react'

const BASE_URL = 'https://openrouter.ai/api/v1'

export type ImportedSortColumn = 'enabled' | 'name' | 'provider_slug' | 'context_length'
export type OpenRouterSortColumn = 'name' | 'usage' | 'context_length' | 'imported'
export type SortDirection = 'asc' | 'desc'

export interface SortState<TColumn extends string> {
  column: TColumn
  direction: SortDirection
}

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
  architecture: {
    input_modalities: string[]
    output_modalities: string[]
  }
}

export interface OpenRouterModelsResponse {
  data: OpenRouterModel[]
}

export interface BlprntModel {
  id: string
  name: string
  slug: string
  context_length: number
  provider_slug: string | null
  enabled: boolean
}

export interface CustomModelDraft {
  id: string
  name: string
  contextLength: string
  providerSlug: string
  openRouterSlug: string
  promptPrice: string
  completionPrice: string
  requestPrice: string
  imagePrice: string
}

const createCustomModelDraft = (): CustomModelDraft => ({
  completionPrice: '0',
  contextLength: '',
  id: '',
  imagePrice: '0',
  name: '',
  openRouterSlug: '',
  promptPrice: '0',
  providerSlug: '',
  requestPrice: '0',
})

const fuzzyMatch = (text: string, query: string): boolean => {
  const textLower = text.toLowerCase()
  const queryLower = query.toLowerCase()

  let textIndex = 0
  for (const char of queryLower) {
    const foundIndex = textLower.indexOf(char, textIndex)
    if (foundIndex === -1) return false
    textIndex = foundIndex + 1
  }

  return true
}

const getModelProvider = (model: OpenRouterModel) => model.id.split('/')[0] ?? ''

const compareText = (left?: string | null, right?: string | null) => (left ?? '').localeCompare(right ?? '')

const compareNumber = (left: number, right: number) => left - right

const normalizePromptPrice = (value: string) => {
  const price = Number.parseFloat(value)
  return Number.isFinite(price) ? price : 0
}

export class ModelsV2ViewModel {
  openRouterModels: OpenRouterModel[] = observable.array([])
  blprntModels: BlprntModel[] = observable.array([])
  isLoading = false
  importedSort: SortState<ImportedSortColumn> = {
    column: 'enabled',
    direction: 'desc',
  }
  importedSearchQuery = ''
  openRouterSort: SortState<OpenRouterSortColumn> = {
    column: 'imported',
    direction: 'desc',
  }
  openRouterSearchQuery = ''
  selectedOpenRouterProviders: string[] = observable.array([])
  isCustomModelFormOpen = false
  customModelDraft: CustomModelDraft = createCustomModelDraft()
  customModelFormError: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  get importedIds() {
    return new Set(this.blprntModels.map((model) => model.id))
  }

  get openRouterProviders() {
    const providers = new Set(this.openRouterModels.map((model) => getModelProvider(model)))

    return Array.from(providers).filter(Boolean).sort()
  }

  get sortedImportedModels() {
    const query = this.importedSearchQuery.trim()
    const multiplier = this.importedSort.direction === 'asc' ? 1 : -1

    return this.blprntModels
      .filter((model) => !query || fuzzyMatch(model.name, query))
      .slice()
      .sort((left, right) => {
        let result = 0
        switch (this.importedSort.column) {
          case 'enabled':
            result = compareNumber(Number(left.enabled), Number(right.enabled))
            break
          case 'name':
            result = compareText(left.name, right.name)
            break
          case 'provider_slug':
            result = compareText(left.provider_slug, right.provider_slug)
            break
          case 'context_length':
            result = compareNumber(left.context_length, right.context_length)
            break
        }

        if (result === 0) return left.name.localeCompare(right.name)
        return result * multiplier
      })
  }

  get sortedOpenRouterModels() {
    const query = this.openRouterSearchQuery.trim()
    const multiplier = this.openRouterSort.direction === 'asc' ? 1 : -1

    return this.openRouterModels
      .filter((model) => !query || fuzzyMatch(model.name, query))
      .filter(
        (model) =>
          this.selectedOpenRouterProviders.length === 0 ||
          this.selectedOpenRouterProviders.includes(getModelProvider(model)),
      )
      .slice()
      .sort((left, right) => {
        let result = 0
        switch (this.openRouterSort.column) {
          case 'name':
            result = compareText(left.name, right.name)
            break
          case 'usage':
            result = compareNumber(
              normalizePromptPrice(left.pricing.prompt),
              normalizePromptPrice(right.pricing.prompt),
            )
            break
          case 'context_length':
            result = compareNumber(left.context_length, right.context_length)
            break
          case 'imported':
            result = compareNumber(Number(this.importedIds.has(left.id)), Number(this.importedIds.has(right.id)))
            break
        }

        if (result === 0) return left.name.localeCompare(right.name)
        return result * multiplier
      })
  }

  get openRouterUsageScalePromptPrices() {
    return this.sortedOpenRouterModels
      .map((model) => normalizePromptPrice(model.pricing.prompt))
      .filter((price) => price > 0)
  }

  get minOpenRouterPromptPrice() {
    if (this.openRouterUsageScalePromptPrices.length === 0) return 0
    return Math.min(...this.openRouterUsageScalePromptPrices)
  }

  get maxOpenRouterPromptPrice() {
    if (this.openRouterUsageScalePromptPrices.length === 0) return 0
    return Math.max(...this.openRouterUsageScalePromptPrices)
  }

  setImportedSearchQuery(value: string) {
    this.importedSearchQuery = value
  }

  setOpenRouterSearchQuery(value: string) {
    this.openRouterSearchQuery = value
  }

  setImportedSort(column: ImportedSortColumn) {
    this.importedSort = {
      column,
      direction: this.importedSort.column === column && this.importedSort.direction === 'desc' ? 'asc' : 'desc',
    }
  }

  setOpenRouterSort(column: OpenRouterSortColumn) {
    this.openRouterSort = {
      column,
      direction: this.openRouterSort.column === column && this.openRouterSort.direction === 'desc' ? 'asc' : 'desc',
    }
  }

  toggleOpenRouterProvider(provider: string) {
    this.selectedOpenRouterProviders = this.selectedOpenRouterProviders.includes(provider)
      ? this.selectedOpenRouterProviders.filter((value) => value !== provider)
      : [...this.selectedOpenRouterProviders, provider]
  }

  openCustomModelForm() {
    this.isCustomModelFormOpen = true
    this.customModelFormError = null
  }

  closeCustomModelForm() {
    this.isCustomModelFormOpen = false
    this.customModelDraft = createCustomModelDraft()
    this.customModelFormError = null
  }

  setCustomModelDraftField<K extends keyof CustomModelDraft>(field: K, value: CustomModelDraft[K]) {
    this.customModelDraft = {
      ...this.customModelDraft,
      [field]: value,
    }
    this.customModelFormError = null
  }

  init = flow(function* (this: ModelsV2ViewModel) {
    this.isLoading = true
    try {
      yield this.loadModelsFromOpenRouter()
      yield this.loadModels()
    } finally {
      this.isLoading = false
    }
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
    const filteredModels = models.filter(
      (model) =>
        model.architecture.input_modalities.includes('text') &&
        model.architecture.output_modalities.includes('text') &&
        model.supported_parameters.includes('tools'),
    )
    this.openRouterModels = filteredModels
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
      context_length: model.context_length,
      enabled: true,
      id: model.id,
      name: model.name,
      provider_slug: null,
      slug: model.cannonical_slug,
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

    existingModel.provider_slug = providerSlug.trim() ? providerSlug : null
    yield this.updateStore()
  })

  deleteModel = flow(function* (this: ModelsV2ViewModel, model: BlprntModel) {
    const nextModels = this.blprntModels.filter((existingModel) => existingModel.id !== model.id)
    if (nextModels.length === this.blprntModels.length) return

    this.blprntModels = nextModels
    yield this.updateStore()
  })

  saveCustomModel = flow(function* (this: ModelsV2ViewModel) {
    const name = this.customModelDraft.name.trim()
    const contextLength = Number.parseInt(this.customModelDraft.contextLength.trim(), 10)

    if (!name) {
      this.customModelFormError = 'Display name is required.'
      return
    }

    if (!Number.isFinite(contextLength) || contextLength <= 0) {
      this.customModelFormError = 'Context length must be a positive integer.'
      return
    }

    if (this.blprntModels.some((model) => model.slug === this.customModelDraft.openRouterSlug.trim())) {
      this.customModelFormError = 'OpenRouter model already exists.'
      return
    }

    const newModel: BlprntModel = {
      context_length: contextLength,
      enabled: true,
      id: `${random(1000000, 9999999).toString()}-${random(1000000, 9999999).toString()}`,
      name,
      provider_slug: this.customModelDraft.providerSlug.trim() || null,
      slug: this.customModelDraft.openRouterSlug.trim(),
    }

    this.blprntModels.push(newModel)
    this.closeCustomModelForm()
    yield this.updateStore()
  })

  updateStore = debounce(async () => {
    const store = await load('imported-models.json')
    await store.set('models', this.blprntModels)
    await store.save()
  }, 1200)
}

export const ModelsV2ViewModelContext = createContext<ModelsV2ViewModel | null>(null)

export const useModelsV2ViewModel = () => {
  const viewmodel = useContext(ModelsV2ViewModelContext)
  if (!viewmodel) throw new Error('useModelsV2ViewModel must be used within ModelsV2ViewModelContext')
  return viewmodel
}
