import { makeAutoObservable, runInAction } from 'mobx'
import type { IssueDto } from '@/bindings/IssueDto'
import { issuesApi } from '@/lib/api/issues'

export class ArchivedIssuesViewmodel {
  public issues: IssueDto[] = []
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
      const issues = await issuesApi.listArchived()
      runInAction(() => {
        this.issues = issues
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load archived issues.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }
}