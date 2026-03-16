import { flow, makeAutoObservable, observable } from 'mobx'
import { createContext, useContext } from 'react'
import type { LlmModel } from '@/bindings'
import { llmModelsModel, type OpenRouterModel } from '@/lib/models/llm-models.model'

export type ImportedSortColumn = 'enabled' | 'name' | 'provider_slug' | 'context_length'
export type OpenRouterSortColumn = 'name' | 'usage' | 'context_length' | 'imported'
export type SortDirection = 'asc' | 'desc'

export interface SortState<TColumn extends string> {
  column: TColumn
  direction: SortDirection
}

export interface CustomModelDraft {
  name: string
  contextLength: string
  providerSlug: string
  slug: string
  promptPrice: string
  supportsReasoning: boolean
}

const createCustomModelDraft = (): CustomModelDraft => ({
  contextLength: '',
  name: '',
  promptPrice: '0',
  providerSlug: '',
  slug: '',
  supportsReasoning: true,
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

const compareNumber = (left: number, right: number) => Number(left) - Number(right)

const normalizePromptPrice = (value: string) => {
  const price = Number.parseFloat(value)
  return Number.isFinite(price) ? price : 0
}

export class ModelsV2ViewModel {
  private readonly llmModelsModel = llmModelsModel

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

  get models() {
    return this.llmModelsModel.models
  }

  get openrouterModels() {
    return this.llmModelsModel.openrouterModels
  }

  get importedIds() {
    return new Set(this.models.map((model) => model.slug))
  }

  get openRouterProviders() {
    const providers = new Set(this.openrouterModels.map((model) => getModelProvider(model)))

    return Array.from(providers).filter(Boolean).sort()
  }

  get sortedImportedModels() {
    const query = this.importedSearchQuery.trim()
    const multiplier = this.importedSort.direction === 'asc' ? 1 : -1

    return this.models
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

    return this.openrouterModels
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

  toggleCustomModelSupportsReasoning() {
    this.customModelDraft.supportsReasoning = !this.customModelDraft.supportsReasoning
    this.customModelFormError = null
  }

  init = flow(function* (this: ModelsV2ViewModel) {
    yield this.llmModelsModel.loadModels()
    yield this.llmModelsModel.loadModelsFromOpenRouter()
  })

  importModel = (model: OpenRouterModel) => {
    this.llmModelsModel.importModel(model)
  }

  toggleModel = (model: LlmModel) => {
    this.llmModelsModel.toggleModel(model)
  }

  setModelProviderSlug = (model: LlmModel, providerSlug: string) => {
    this.llmModelsModel.setModelProviderSlug(model, providerSlug)
  }

  deleteModel = (model: LlmModel) => {
    this.llmModelsModel.deleteModel(model)
  }

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

    if (this.models.some((model) => model.slug === this.customModelDraft.slug.trim())) {
      this.customModelFormError = 'OpenRouter model already exists.'
      return
    }

    const newModel: LlmModel = {
      context_length: contextLength,
      enabled: true,
      name,
      provider_slug: this.customModelDraft.providerSlug.trim() || null,
      slug: this.customModelDraft.slug.trim(),
      supports_reasoning: this.customModelDraft.supportsReasoning,
    }

    yield this.llmModelsModel.saveCustomModel(newModel)
    this.closeCustomModelForm()
  })
}

export const ModelsV2ViewModelContext = createContext<ModelsV2ViewModel | null>(null)

export const useModelsV2ViewModel = () => {
  const viewmodel = useContext(ModelsV2ViewModelContext)
  if (!viewmodel) throw new Error('useModelsV2ViewModel must be used within ModelsV2ViewModelContext')
  return viewmodel
}
