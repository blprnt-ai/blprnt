import { makeAutoObservable } from 'mobx'
import type { ProviderDto, UpsertProviderArgs } from '@/bindings'
import { tauriProvidersApi } from '@/lib/api/tauri/providers.api'
import type { AllProviders } from '@/types'

export type ProviderId = string

export interface CreateProviderArgs {
  provider: AllProviders
  apiKey: string
}

export class ProviderModel {
  public id: string
  public provider: AllProviders
  public apiKey: string = ''
  public baseUrl: string = ''
  public createdAt: number
  public updatedAt: number

  constructor(model: ProviderDto) {
    this.id = model.id
    this.provider = model.provider as AllProviders
    this.apiKey = model.api_key
    this.baseUrl = model.base_url ?? ''
    this.createdAt = model.created_at
    this.updatedAt = model.updated_at

    makeAutoObservable(this, {}, { autoBind: true })
  }

  static listEnabled = async () => {
    return tauriProvidersApi.listEnabled()
  }

  static list = async () => {
    const result = await tauriProvidersApi.list()
    return result.map((item) => new ProviderModel(item))
  }

  static create = async ({ provider, apiKey }: CreateProviderArgs) => {
    const result = await tauriProvidersApi.create(provider, apiKey)
    return new ProviderModel({ ...result, api_key: apiKey })
  }

  static upsert = async (args: UpsertProviderArgs) => {
    const result = await tauriProvidersApi.upsert(args)
    return new ProviderModel({ ...result, api_key: args.api_key })
  }

  static createFnf = async ({ provider }: CreateProviderArgs) => {
    const result = await tauriProvidersApi.createFnf(provider)
    return new ProviderModel({ ...result, api_key: '' })
  }

  static linkCodexAccount = async () => {
    await tauriProvidersApi.linkCodexAccount()
  }

  static unlinkCodexAccount = async () => {
    await tauriProvidersApi.unlinkCodexAccount()
  }

  static linkClaudeAccount = async () => {
    await tauriProvidersApi.linkClaudeAccount()
  }

  static unlinkClaudeAccount = async () => {
    await tauriProvidersApi.unlinkClaudeAccount()
  }

  delete = async () => {
    await tauriProvidersApi.delete(this.id)
  }
}
