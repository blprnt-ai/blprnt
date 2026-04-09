import { makeAutoObservable, runInAction } from 'mobx'
import type { MinionDto } from '@/bindings/MinionDto'
import { MinionFormViewmodel } from './minion.viewmodel'

export class MinionSheetViewmodel {
  public editor = new MinionFormViewmodel()
  public isOpen = false
  private readonly onSaved?: (minion: MinionDto) => Promise<void> | void

  constructor(onSaved?: (minion: MinionDto) => Promise<void> | void) {
    this.onSaved = onSaved
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get actionLabel() {
    if (this.editor.minion.isReadOnly) return 'Close'
    return this.editor.minion.isNew ? 'Create minion' : 'Save minion'
  }

  public get description() {
    if (this.editor.minion.isSystem) {
      return 'Built-in minion definitions are fixed. You can still enable or disable this minion.'
    }
    if (this.editor.minion.isReadOnly) return 'This minion cannot be edited.'
    return this.editor.minion.isNew ? 'Create a custom owner-managed minion.' : 'Update this custom minion.'
  }

  public get title() {
    if (this.editor.minion.isSystem || !this.editor.minion.isReadOnly)
      return this.editor.minion.isNew ? 'New minion' : `Manage ${this.editor.minion.displayName}`
    if (this.editor.minion.isReadOnly) return `View ${this.editor.minion.displayName}`
    return this.editor.minion.isNew ? 'New minion' : `Manage ${this.editor.minion.displayName}`
  }

  public openForCreate() {
    this.editor.reset()
    this.isOpen = true
  }

  public openForEdit(minion: MinionDto) {
    this.editor.setMinion(minion)
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
    const minion = await this.editor.save()
    if (!minion) return null

    await this.onSaved?.(minion)

    runInAction(() => {
      this.isOpen = false
      this.editor.reset()
    })

    return minion
  }
}
