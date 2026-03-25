import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { providersApi } from '@/lib/api/providers'
import { ProviderModel } from '@/models/provider.model'

export class ProviderFormViewmodel {
  public provider: ProviderModel = new ProviderModel()
  public isSaving = false

  constructor() {
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

  public save = async () => {
    if (!this.provider.isDirty) return
    this.isSaving = true

    try {
      if (!this.provider.id) await this.createProvider()
      else await this.updateProvider()
    } catch (error) {
      console.error(error)
      toast.error('Failed to save provider')
    } finally {
      this.isSaving = false
    }
  }

  private createProvider = async () => {
    const provider = await providersApi.create(this.provider.toPayload())
    this.setProvider(provider)
  }

  private updateProvider = async () => {
    const provider = await providersApi.update(this.provider.id!, this.provider.toPayloadPatch())
    this.setProvider(provider)
  }
}
