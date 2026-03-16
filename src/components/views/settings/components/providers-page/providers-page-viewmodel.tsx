import { flow, makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import type { Provider, TauriError } from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
import { ProviderModel } from '@/lib/models/provider.model'
import { defaultProviderModel } from '@/lib/utils/default-models'

export class ProvidersPageViewmodel {
  public isCodexLinking = false
  public isClaudeLinking = false

  public openaiProvider: ProviderModel = new ProviderModel({ ...defaultProviderModel, api_key: '' })
  public anthropicProvider: ProviderModel = new ProviderModel({ ...defaultProviderModel, api_key: '' })
  public openRouterProvider: ProviderModel = new ProviderModel({ ...defaultProviderModel, api_key: '', provider: 'open_router' })

  public showOpenAiApiKey = false
  public showAnthropicApiKey = false
  public showOpenRouterApiKey = false

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public linkCodexAccount = flow(function* (this: ProvidersPageViewmodel) {
    this.isCodexLinking = true
    yield ProviderModel.linkCodexAccount()
    this.isCodexLinking = false
  })

  public linkClaudeAccount = flow(function* (this: ProvidersPageViewmodel) {
    this.isClaudeLinking = true
    try {
      yield ProviderModel.linkClaudeAccount()
    } catch (error) {
      console.error('Failed to link Claude account', error)
      console.log('error', error, typeof error)
      if (error instanceof Error && 'message' in error) {
        basicToast.error({ description: error.message, title: 'Failed to link Claude account' })
      } else if ('message' in (error as TauriError)) {
        const err = error as TauriError
        basicToast.error({ description: err.message, title: 'Failed to link Claude account' })
      } else {
        basicToast.error({ description: 'Unknown error', title: 'Failed to link Claude account' })
      }
    }

    this.isClaudeLinking = false
  })

  public unlinkCodexAccount = flow(function* (this: ProvidersPageViewmodel) {
    yield ProviderModel.unlinkCodexAccount()
  })

  public unlinkClaudeAccount = flow(function* (this: ProvidersPageViewmodel) {
    yield ProviderModel.unlinkClaudeAccount()
  })

  public providerId(provider: Provider) {
    if (provider === 'openai') return this.openaiProvider.id
    if (provider === 'anthropic') return this.anthropicProvider.id
    if (provider === 'open_router') return this.openRouterProvider.id
  }

  public apiKey(provider: Provider) {
    if (provider === 'openai') return this.openaiProvider.apiKey
    if (provider === 'anthropic') return this.anthropicProvider.apiKey
    if (provider === 'open_router') return this.openRouterProvider.apiKey
    return ''
  }

  public baseUrl(provider: Provider) {
    if (provider === 'openai') return this.openaiProvider.baseUrl
    if (provider === 'anthropic') return this.anthropicProvider.baseUrl
    return ''
  }

  public apiKeyVisibility(provider: Provider) {
    if (provider === 'openai') return this.showOpenAiApiKey
    if (provider === 'anthropic') return this.showAnthropicApiKey
    if (provider === 'open_router') return this.showOpenRouterApiKey
    return false
  }

  public refreshProviders = flow(function* (this: ProvidersPageViewmodel) {
    const providers: ProviderModel[] = yield ProviderModel.list()
    const openaiProvider = providers.find((p) => p.provider === 'openai')
    const anthropicProvider = providers.find((p) => p.provider === 'anthropic')
    const openRouterProvider = providers.find((p) => p.provider === 'open_router')

    if (openaiProvider) this.openaiProvider = openaiProvider
    if (anthropicProvider) this.anthropicProvider = anthropicProvider
    if (openRouterProvider) this.openRouterProvider = openRouterProvider
  })

  public saveProvider = flow(function* (this: ProvidersPageViewmodel, provider: Provider) {
    const toastId = `save-provider-${provider}`
    basicToast.loading({ id: toastId, title: 'Saving provider...' })
    try {
      if (provider === 'openai') {
        yield this.saveOpenaiProvider()
      } else if (provider === 'anthropic') {
        yield this.saveAnthropicProvider()
      } else if (provider === 'open_router') {
        yield this.saveOpenRouterProvider()
      }
      yield this.refreshProviders()
      basicToast.success({ id: toastId, title: 'Provider saved' })
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      basicToast.error({ description: errorMessage, id: toastId, title: 'Failed to save provider' })
    }
  })

  public async saveOpenaiProvider() {
    if (!this.openaiProvider || !this.openaiProvider.apiKey) return
    await ProviderModel.upsert({
      api_key: this.openaiProvider.apiKey,
      base_url: this.openaiProvider.baseUrl,
      provider: 'openai',
    })
  }

  public async saveAnthropicProvider() {
    if (!this.anthropicProvider || !this.anthropicProvider.apiKey) return
    await ProviderModel.upsert({
      api_key: this.anthropicProvider.apiKey,
      base_url: this.anthropicProvider.baseUrl,
      provider: 'anthropic',
    })
  }

  public async saveOpenRouterProvider() {
    if (!this.openRouterProvider.apiKey) return
    if (this.openRouterProvider.id !== defaultProviderModel.id) {
      await this.openRouterProvider.delete()
    }

    this.openRouterProvider = await ProviderModel.create({
      apiKey: this.openRouterProvider.apiKey,
      provider: 'open_router',
    })
  }

  public setApiKey(provider: Provider, apiKey: string) {
    if (provider === 'openai') {
      this.openaiProvider.apiKey = apiKey
    } else if (provider === 'anthropic') {
      this.anthropicProvider.apiKey = apiKey
    } else if (provider === 'open_router') {
      this.openRouterProvider.apiKey = apiKey
    }
  }

  public setBaseUrl(provider: Provider, baseUrl: string) {
    if (provider === 'openai') {
      this.openaiProvider.baseUrl = baseUrl
    } else if (provider === 'anthropic') {
      this.anthropicProvider.baseUrl = baseUrl
    }
  }

  public toggleApiKeyVisibility = (provider: Provider) => {
    if (provider === 'openai') {
      this.showOpenAiApiKey = !this.showOpenAiApiKey
    } else if (provider === 'anthropic') {
      this.showAnthropicApiKey = !this.showAnthropicApiKey
    } else if (provider === 'open_router') {
      this.showOpenRouterApiKey = !this.showOpenRouterApiKey
    }
  }

  public deleteProvider = flow(function* (this: ProvidersPageViewmodel, provider: Provider) {
    const toastId = `delete-provider-${provider}`
    basicToast.loading({ id: toastId, title: 'Deleting provider...' })
    try {
      if (provider === 'openai') {
        yield this.openaiProvider.delete()
        this.openaiProvider = new ProviderModel({ ...defaultProviderModel, api_key: '' })
      } else if (provider === 'anthropic') {
        yield this.anthropicProvider.delete()
        this.anthropicProvider = new ProviderModel({ ...defaultProviderModel, api_key: '' })
      } else if (provider === 'open_router') {
        yield this.openRouterProvider.delete()
        this.openRouterProvider = new ProviderModel({ ...defaultProviderModel, api_key: '', provider: 'open_router' })
      }
      basicToast.success({ id: toastId, title: 'Provider deleted' })
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      basicToast.error({ description: errorMessage, id: toastId, title: 'Failed to delete provider' })
    }
  })
}

export const ProvidersPageViewmodelContext = createContext<ProvidersPageViewmodel>(new ProvidersPageViewmodel())
export const useProvidersPageViewmodel = () => {
  const viewmodel = useContext(ProvidersPageViewmodelContext)
  if (!viewmodel) throw new Error('useProvidersPageViewmodel must be used within ProvidersPageViewmodelContext')
  return viewmodel
}
