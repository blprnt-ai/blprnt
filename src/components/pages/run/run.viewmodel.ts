import { makeAutoObservable, runInAction } from 'mobx'
import { RunsViewmodel } from '@/runs.viewmodel'

export class RunPageViewmodel {
  public isLoading = true
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
}
