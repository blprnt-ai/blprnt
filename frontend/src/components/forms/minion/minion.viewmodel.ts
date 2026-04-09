import { makeAutoObservable } from 'mobx'
import { toast } from 'sonner'
import type { MinionDto } from '@/bindings/MinionDto'
import { minionsApi } from '@/lib/api/minions'
import { MinionModel } from '@/models/minion.model'

export class MinionFormViewmodel {
  public isSaving = false
  public minion: MinionModel

  constructor(minion?: MinionDto | MinionModel) {
    this.minion = minion instanceof MinionModel ? minion : new MinionModel(minion)
    makeAutoObservable(this)
  }

  public get canSave() {
    if (this.minion.isReadOnly) return false
    return this.minion.isValid && !this.isSaving && (this.minion.isNew || this.minion.isDirty)
  }

  public setMinion(minion: MinionDto | MinionModel) {
    this.minion = minion instanceof MinionModel ? minion : new MinionModel(minion)
  }

  public reset() {
    this.minion = new MinionModel()
  }

  public async save(): Promise<MinionDto | null> {
    if (!this.canSave) return null

    this.isSaving = true

    try {
      const minion = this.minion.isNew
        ? await minionsApi.create(this.minion.toPayload())
        : await minionsApi.update(this.minion.id, this.minion.toPayloadPatch())

      this.minion = new MinionModel(minion)
      return minion
    } catch (error) {
      console.error(error)
      toast.error(error instanceof Error ? error.message : 'Failed to save minion')
      return null
    } finally {
      this.isSaving = false
    }
  }
}
