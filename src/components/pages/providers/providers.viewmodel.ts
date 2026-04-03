import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import {
  PROVIDER_OPTIONS,
  type ProviderOption,
  type SupportedProvider,
} from '@/components/forms/provider/provider-catalog'
import { ProviderSheetViewmodel } from '@/components/forms/provider/provider-sheet.viewmodel'
import { providersApi } from '@/lib/api/providers'

export class ProvidersViewmodel {
  public errorMessage: string | null = null
  public isDeletingProviderId: string | null = null
  public isLoading = true
  public providers: ProviderDto[] = []
  public readonly sheet: ProviderSheetViewmodel

  constructor() {
    this.sheet = new ProviderSheetViewmodel((provider) => this.handleProviderSaved(provider))

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get availableOptions() {
    return PROVIDER_OPTIONS.filter((option) => !this.findProvider(option.provider))
  }

  public get catalogEntries() {
    return PROVIDER_OPTIONS.map((option) => ({
      option,
      provider: this.findProvider(option.provider),
    }))
  }

  public get connectedProviders() {
    return this.catalogEntries.flatMap((entry) => (entry.provider ? [entry.provider] : []))
  }

  public async init() {
    runInAction(() => {
      this.errorMessage = null
      this.isLoading = true
    })

    try {
      const providers = await providersApi.list()

      runInAction(() => {
        this.providers = sortProviders(providers)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load providers.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public findProvider(provider: Provider) {
    return this.providers.find((candidate) => candidate.provider === provider) ?? null
  }

  public openProvider(option: ProviderOption) {
    const provider = this.findProvider(option.provider)

    if (provider) {
      this.sheet.openForEdit(provider)
      return
    }

    this.sheet.openForCreate(option.provider)
  }

  public async deleteProvider(providerId: string) {
    if (this.isDeletingProviderId) return

    runInAction(() => {
      this.errorMessage = null
      this.isDeletingProviderId = providerId
    })

    try {
      await providersApi.delete(providerId)

      runInAction(() => {
        this.providers = this.providers.filter((provider) => provider.id !== providerId)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to remove this provider.')
      })
    } finally {
      runInAction(() => {
        this.isDeletingProviderId = null
      })
    }
  }

  private handleProviderSaved(provider: ProviderDto) {
    const index = this.providers.findIndex((candidate) => candidate.id === provider.id)

    if (index === -1) {
      this.providers = sortProviders([...this.providers, provider])
      return
    }

    this.providers = sortProviders(
      this.providers.map((candidate) => (candidate.id === provider.id ? provider : candidate)),
    )
  }
}

export const ProvidersViewmodelContext = createContext<ProvidersViewmodel | null>(null)

export const useProvidersViewmodel = () => {
  const viewmodel = useContext(ProvidersViewmodelContext)
  if (!viewmodel) throw new Error('ProvidersViewmodel not found')

  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message

  return fallback
}

const sortProviders = (providers: ProviderDto[]) => {
  return [...providers].sort((left, right) => providerOrder(left.provider) - providerOrder(right.provider))
}

const providerOrder = (provider: Provider) => {
  const index = PROVIDER_OPTIONS.findIndex((option) => option.provider === (provider as SupportedProvider))

  return index === -1 ? Number.MAX_SAFE_INTEGER : index
}
