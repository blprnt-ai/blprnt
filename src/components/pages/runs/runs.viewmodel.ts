import { makeAutoObservable, runInAction } from 'mobx'
import { RunSummaryModel } from '@/models/run-summary.model'
import { RunsViewmodel } from '@/runs.viewmodel'

export class RunsPageViewmodel {
  public ids: string[] = []
  public isLoading = true
  public errorMessage: string | null = null
  public page = 1
  public perPage = 20
  public total = 0
  public totalPages = 1
  private readonly runs: RunsViewmodel

  constructor(runs: RunsViewmodel, page: number) {
    this.runs = runs
    this.page = page

    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get items() {
    return this.ids
      .map((id) => this.runs.getSummary(id))
      .filter((run): run is RunSummaryModel => Boolean(run))
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const response = await this.runs.loadPage(this.page, this.perPage)
      runInAction(() => {
        this.ids = response.items.map((item) => item.id)
        this.total = response.total
        this.totalPages = response.total_pages
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load runs.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }
}
