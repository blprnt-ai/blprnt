import { makeAutoObservable, runInAction } from 'mobx'
import type { RunStatusFilter } from '@/lib/api/runs'
import type { RunSummaryModel } from '@/models/run-summary.model'
import type { RunsViewmodel } from '@/runs.viewmodel'

export interface RunsPageFilters {
  employeeId: string | null
  status: RunStatusFilter | null
}

export class RunsPageViewmodel {
  public ids: string[] = []
  public isLoading = true
  public errorMessage: string | null = null
  public page = 1
  public perPage = 20
  public total = 0n
  public totalPages = 1
  private readonly runs: RunsViewmodel
  public readonly filters: RunsPageFilters

  constructor(runs: RunsViewmodel, page: number, filters: RunsPageFilters) {
    this.runs = runs
    this.page = page
    this.filters = filters

    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get items() {
    return this.ids.map((id) => this.runs.getSummary(id)).filter((run): run is RunSummaryModel => Boolean(run))
  }

  public get hasActiveFilters() {
    return Boolean(this.filters.employeeId || this.filters.status)
  }

  public async init() {
    runInAction(() => {
      this.isLoading = true
      this.errorMessage = null
    })

    try {
      const response = await this.runs.loadPage(this.page, this.perPage, this.filters)
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
