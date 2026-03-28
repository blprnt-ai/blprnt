import { makeAutoObservable } from 'mobx'
import type { CreateProviderPayload } from '@/bindings/CreateProviderPayload'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import type { UpdateProviderPayload } from '@/bindings/UpdateProviderPayload'
import { ModelField } from './model-field'

export class ProviderModel {
  public id: string
  private _apiKey: ModelField<string>
  private _provider: ModelField<Provider>
  private _baseUrl: ModelField<string>
  public createdAt: Date
  public updatedAt: Date

  constructor(provider?: ProviderDto) {
    this.id = provider?.id ?? ''
    this._apiKey = new ModelField('')
    this._provider = new ModelField(provider?.provider ?? 'claude_code')
    this._baseUrl = new ModelField(provider?.base_url ?? '')
    this.createdAt = new Date(provider?.created_at ?? '')
    this.updatedAt = new Date(provider?.updated_at ?? '')

    makeAutoObservable(this)
  }

  public get isDirty() {
    return this._provider.isDirty || this._baseUrl.isDirty
  }

  public get isValid() {
    return this.isOauthProvider || this.apiKey.length > 0
  }

  public get apiKey() {
    return this._apiKey.value
  }

  public get isOauthProvider() {
    return this._provider.value === 'claude_code' || this._provider.value === 'codex'
  }

  public set apiKey(apiKey: string) {
    this._apiKey.value = apiKey
  }

  public get provider() {
    return this._provider.value
  }

  public set provider(provider: Provider) {
    this._provider.value = provider
  }

  public get baseUrl() {
    return this._baseUrl.value
  }

  public set baseUrl(baseUrl: string) {
    this._baseUrl.value = baseUrl
  }

  public toPayload(): CreateProviderPayload {
    return {
      api_key: !this.isOauthProvider ? this._apiKey.value : null,
      base_url: !this.isOauthProvider ? this._baseUrl.value : null,
      provider: this._provider.value,
    }
  }

  public toPayloadPatch(): UpdateProviderPayload {
    return {
      api_key: !this.isOauthProvider ? this._apiKey.dirtyValue : null,
      base_url: !this.isOauthProvider ? this._baseUrl.dirtyValue : null,
      provider: this._provider.value,
    }
  }
}
