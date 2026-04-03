import { makeAutoObservable, runInAction } from 'mobx'
import type { ProviderDto } from '@/bindings/ProviderDto'
import { formatProvider } from '@/components/pages/employee/utils'
import { ProviderFormViewmodel } from './provider.viewmodel'
import type { SupportedProvider } from './provider-catalog'

export class ProviderSheetViewmodel {
  public editor = new ProviderFormViewmodel()
  public isOpen = false
  private readonly onSaved?: (provider: ProviderDto) => Promise<void> | void

  constructor(onSaved?: (provider: ProviderDto) => Promise<void> | void) {
    this.onSaved = onSaved

    makeAutoObservable(
      this,
      {},
      {
        autoBind: true,
      },
    )
  }

  public get actionLabel() {
    return this.editor.provider.id ? 'Save provider' : 'Connect provider'
  }

  public get description() {
    return this.editor.provider.id
      ? 'Update credentials or relink this provider.'
      : 'Add this provider to your workspace.'
  }

  public get title() {
    const provider = formatProvider(this.editor.provider.provider)

    return `${this.editor.provider.id ? 'Manage' : 'Connect'} ${provider}`
  }

  public openForCreate(provider: SupportedProvider) {
    this.editor.reset(provider)
    this.isOpen = true
  }

  public openForEdit(provider: ProviderDto) {
    this.editor.setProvider(provider)
    this.isOpen = true
  }

  public close() {
    if (this.editor.isSaving) return

    this.isOpen = false
    this.editor.reset()
  }

  public setOpen(isOpen: boolean) {
    if (isOpen) return

    this.close()
  }

  public async save() {
    const provider = await this.editor.save()
    if (!provider) return null

    await this.onSaved?.(provider)

    runInAction(() => {
      this.isOpen = false
      this.editor.reset()
    })

    return provider
  }
}
