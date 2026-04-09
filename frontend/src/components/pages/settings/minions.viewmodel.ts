import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { MinionDto } from '@/bindings/MinionDto'
import { MinionSheetViewmodel } from '@/components/forms/minion/minion-sheet.viewmodel'
import { minionsApi } from '@/lib/api/minions'

export class MinionsViewmodel {
  public errorMessage: string | null = null
  public isDeletingMinionId: string | null = null
  public isLoading = true
  public minions: MinionDto[] = []
  public readonly sheet: MinionSheetViewmodel

  constructor() {
    this.sheet = new MinionSheetViewmodel((minion) => this.handleMinionSaved(minion))
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get systemMinions() {
    return this.minions.filter((minion) => minion.source === 'system')
  }

  public get customMinions() {
    return this.minions.filter((minion) => minion.source === 'custom')
  }

  public async init() {
    runInAction(() => {
      this.errorMessage = null
      this.isLoading = true
    })

    try {
      const minions = await minionsApi.list()
      runInAction(() => {
        this.minions = sortMinions(minions)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to load minions.')
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public openCreate() {
    this.sheet.openForCreate()
  }

  public openEdit(minion: MinionDto) {
    this.sheet.openForEdit(minion)
  }

  public async deleteMinion(minionId: string) {
    if (this.isDeletingMinionId) return

    runInAction(() => {
      this.errorMessage = null
      this.isDeletingMinionId = minionId
    })

    try {
      await minionsApi.delete(minionId)
      runInAction(() => {
        this.minions = this.minions.filter((minion) => minion.id !== minionId)
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = getErrorMessage(error, 'Unable to delete this minion.')
      })
    } finally {
      runInAction(() => {
        this.isDeletingMinionId = null
      })
    }
  }

  private handleMinionSaved(minion: MinionDto) {
    const index = this.minions.findIndex((candidate) => candidate.id === minion.id)
    this.minions = index === -1 ? sortMinions([...this.minions, minion]) : sortMinions(this.minions.map((candidate) => (candidate.id === minion.id ? minion : candidate)))
  }
}

export const MinionsViewmodelContext = createContext<MinionsViewmodel | null>(null)

export const useMinionsViewmodel = () => {
  const viewmodel = useContext(MinionsViewmodelContext)
  if (!viewmodel) throw new Error('MinionsViewmodel not found')
  return viewmodel
}

const getErrorMessage = (error: unknown, fallback: string) => {
  if (error instanceof Error && error.message.trim().length > 0) return error.message
  return fallback
}

const sortMinions = (minions: MinionDto[]) => {
  return [...minions].sort((left, right) => {
    if (left.source !== right.source) return left.source === 'system' ? -1 : 1
    return left.display_name.localeCompare(right.display_name)
  })
}