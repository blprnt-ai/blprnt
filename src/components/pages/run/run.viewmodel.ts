import { makeAutoObservable, runInAction } from 'mobx'
import { runsApi } from '@/lib/api/runs'
import type { RunsViewmodel } from '@/runs.viewmodel'

export class RunPageViewmodel {
  public isLoading = true
  public isCancelling = false
  public errorMessage: string | null = null
  private readonly runId: string
  private readonly runs: RunsViewmodel

  constructor(runId: string, runs: RunsViewmodel) {
    this.runId = runId
    this.runs = runs

    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get run() {
    return this.runs.getRun(this.runId)
  }

  public get canCancel() {
    return this.run?.status === 'Pending' || this.run?.status === 'Running'
  }

  public async init() {
    runInAction(() => {
      this.isLoading = !this.run
      this.errorMessage = null
    })

    try {
      await this.runs.loadRun(this.runId)
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load run.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  public async cancel() {
    if (!this.canCancel || this.isCancelling) return false

    runInAction(() => {
      this.isCancelling = true
      this.errorMessage = null
    })

    try {
      await runsApi.cancel(this.runId)
      await this.runs.loadRun(this.runId)
      return true
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to cancel this run.'
      })

      return false
    } finally {
      runInAction(() => {
        this.isCancelling = false
      })
    }
  }
}
