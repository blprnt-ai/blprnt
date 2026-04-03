import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
import type { Provider } from '@/bindings/Provider'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { providersApi } from '@/lib/api/providers'
import { ProviderModel } from '@/models/provider.model'

export class ProviderFormViewmodel {
  public provider: ProviderModel
  public isSaving = false

  constructor(provider?: ProviderDto | ProviderModel) {
    this.provider = provider instanceof ProviderModel ? provider : new ProviderModel(provider)
    makeAutoObservable(this)
  }

  public get canSave() {
    return this.provider.isValid && !this.isSaving
  }

  public init = async (providerId?: string) => {
    if (!providerId) return

    const provider = await providersApi.get(providerId)
    this.setProvider(provider)
  }

  public setProvider = (provider: ProviderDto | ProviderModel) => {
    this.provider = provider instanceof ProviderModel ? provider : new ProviderModel(provider)
  }

  public reset = (provider?: Provider) => {
    this.provider = new ProviderModel()
    if (provider) this.provider.provider = provider
  }

  public save = async (): Promise<ProviderDto | null> => {
    if (!this.canSave) return null

    this.setIsSaving(true)

    try {
      if (this.provider.isNew) return await this.createProvider()

      return await this.updateProvider()
    } catch (error) {
      console.error(error)
      toast.error('Failed to save provider')
      return null
    } finally {
      this.setIsSaving(false)
    }
  }

  private setProviderRecord = (provider: ProviderDto) => {
    this.provider = new ProviderModel(provider)
  }

  private setIsSaving = (isSaving: boolean) => {
    this.isSaving = isSaving
  }

  private createProvider = async () => {
    const provider = await providersApi.create(this.provider.toPayload())
    this.setProviderRecord(provider)

    return provider
  }

  private updateProvider = async () => {
    const provider = await providersApi.update(this.provider.id!, this.provider.toPayloadPatch())
    this.setProviderRecord(provider)

    return provider
  }
}
