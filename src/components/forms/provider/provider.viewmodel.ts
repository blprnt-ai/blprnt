import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
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

  public init = async (providerId?: string) => {
    if (!providerId) return

    const provider = await providersApi.get(providerId)
    this.setProvider(provider)
  }

  private setProvider = (provider: ProviderDto) => {
    this.provider = new ProviderModel(provider)
  }

  public save = async (): Promise<ProviderDto | null> => {
    if (!this.provider.isDirty) return null
    this.isSaving = true

    try {
      if (!this.provider.id) return await this.createProvider()

      return await this.updateProvider()
    } catch (error) {
      console.error(error)
      toast.error('Failed to save provider')
      return null
    } finally {
      this.isSaving = false
    }
  }

  private createProvider = async () => {
    const provider = await providersApi.create(this.provider.toPayload())
    this.setProvider(provider)

    return provider
  }

  private updateProvider = async () => {
    const provider = await providersApi.update(this.provider.id!, this.provider.toPayloadPatch())
    this.setProvider(provider)

    return provider
  }
}
