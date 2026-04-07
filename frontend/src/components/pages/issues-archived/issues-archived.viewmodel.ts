import { makeAutoObservable, runInAction } from 'mobx'
import type { IssueDto } from '@/bindings/IssueDto'
import { issuesApi } from '@/lib/api/issues'
import { AppModel } from '@/models/app.model'

export class ArchivedIssuesViewmodel {
  public issues: IssueDto[] = []
  public isLoading = true
  public errorMessage: string | null = null
  public searchQuery = ''

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get filteredIssues() {
    const query = this.searchQuery.trim().toLowerCase()
    if (!query) return this.issues

    return this.issues.filter((issue) => {
      const haystack = [
        issue.identifier,
        issue.title,
        issue.description,
        issue.labels.map((label) => label.name).join(' '),
        AppModel.instance.resolveProjectName(issue.project) ?? '',
        AppModel.instance.resolveEmployeeName(issue.assignee) ?? '',
      ]
        .join(' ')
        .toLowerCase()

      return haystack.includes(query)
    })
  }

  public setSearchQuery(value: string) {
    this.searchQuery = value
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