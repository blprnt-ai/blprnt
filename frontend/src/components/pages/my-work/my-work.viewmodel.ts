import { makeAutoObservable, runInAction } from 'mobx'
import type { MyWorkItemDto } from '@/bindings/MyWorkItemDto'
import { issuesApi } from '@/lib/api/issues'

export class MyWorkViewmodel {
  public assigned: MyWorkItemDto[] = []
  public mentioned: MyWorkItemDto[] = []
  public isLoading = true
  public errorMessage: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const response = await issuesApi.getMyWork()
      runInAction(() => {
        this.assigned = response.assigned
        this.mentioned = response.mentioned
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load My Work.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public get totalItems() {
    return this.assigned.length + this.mentioned.length
  }

  public get newestItem() {
    return (
      [...this.assigned, ...this.mentioned].sort(
        (left, right) => new Date(right.relevant_at).getTime() - new Date(left.relevant_at).getTime(),
      )[0] ?? null
    )
  }
}
