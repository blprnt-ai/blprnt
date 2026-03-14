import { commands, type UpsertProviderArgs } from '@/bindings'
import type { ProviderId } from '@/lib/models/provider.model'
import type { AllProviders } from '@/types'

class TauriProvidersApi {
  public async listEnabled() {
    const result = await commands.listEnabledProviders()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async list() {
    const result = await commands.listProviders()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async create(provider: AllProviders, apiKey: string) {
    const result = await commands.createProvider(provider, apiKey)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async createFnf(provider: AllProviders) {
    const result = await commands.createProviderFnf(provider)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async upsert(args: UpsertProviderArgs) {
    const result = await commands.upsertProvider(args)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async delete(providerId: ProviderId) {
    const result = await commands.deleteProvider(providerId)
    if (result.status === 'error') throw result.error
  }

  public async getModels() {
    const result = await commands.getModelsCatalog()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async linkCodexAccount() {
    const result = await commands.linkCodexAccount()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async unlinkCodexAccount() {
    const result = await commands.unlinkCodexAccount()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async linkClaudeAccount() {
    const result = await commands.linkClaudeAccount()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async unlinkClaudeAccount() {
    const result = await commands.unlinkClaudeAccount()
    if (result.status === 'error') throw result.error

    return result.data
  }
}

export const tauriProvidersApi = new TauriProvidersApi()
