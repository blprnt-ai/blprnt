import { makeAutoObservable, runInAction } from 'mobx'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import { runsApi } from '@/lib/api/runs'
import { AppModel } from '@/models/app.model'
import type { RunsViewmodel } from '@/runs.viewmodel'

interface RunPageViewmodelOptions {
  employeeId?: string
  onRunCreated?: (runId: string) => Promise<void> | void
  runId?: string
  runs: RunsViewmodel
}

export class RunPageViewmodel {
  public composerValue = ''
  public composerReasoningEffort: ReasoningEffort | null = null
  public errorMessage: string | null = null
  public isCancelling = false
  public isLoading = true
  public isSending = false
  private readonly draftEmployeeId: string | null
  private runId: string | null
  private readonly runs: RunsViewmodel
  private readonly onRunCreated?: (runId: string) => Promise<void> | void

  constructor({ employeeId, onRunCreated, runId, runs }: RunPageViewmodelOptions) {
    this.draftEmployeeId = employeeId ?? null
    this.onRunCreated = onRunCreated
    this.runId = runId ?? null
    this.runs = runs

    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get canCancel() {
    return this.run?.status === 'Pending' || this.run?.status === 'Running'
  }

  public get canSendMessage() {
    if (this.isSending) return false
    if (!this.composerValue.trim()) return false
    if (this.isDraft) return Boolean(this.employeeId)

    return this.run?.status === 'Completed'
  }

  public get composerPlaceholder() {
    return this.isDraft ? 'Send the first message...' : 'Continue the conversation...'
  }

  public get employeeReasoningEffort() {
    return (
      AppModel.instance.employees.find((employee) => employee.id === this.employeeId)?.runtime_config?.reasoning_effort ??
      null
    )
  }

  public get employeeId() {
    return this.run?.employeeId ?? this.draftEmployeeId
  }

  public get isDraft() {
    return !this.runId
  }

  public get run() {
    return this.runId ? this.runs.getRun(this.runId) : null
  }

  public get reasoningSelectValue() {
    return this.composerReasoningEffort ?? null
  }

  public get showComposer() {
    return this.isDraft || this.run?.status === 'Completed'
  }

  public async init() {
    if (!this.runId) {
      runInAction(() => {
        this.composerReasoningEffort = null
        this.errorMessage = null
        this.isLoading = false
      })
      return
    }

    runInAction(() => {
      this.isLoading = !this.run
      this.errorMessage = null
    })

    try {
      await this.runs.loadRun(this.runId)
      runInAction(() => {
        this.composerReasoningEffort = this.run?.turns.at(-1)?.reasoningEffort ?? null
      })
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

  public setComposerValue(value: string) {
    this.composerValue = value
  }

  public setComposerReasoningEffort(value: ReasoningEffort | null) {
    this.composerReasoningEffort = value
  }

  public async sendMessage() {
    if (!this.canSendMessage) return false

    const prompt = this.composerValue.trim()
    if (!prompt) return false

    runInAction(() => {
      this.errorMessage = null
      this.isSending = true
    })

    try {
      if (this.isDraft) {
        if (!this.employeeId) throw new Error('Missing employee for this conversation.')

        const run = await runsApi.trigger({
          employee_id: this.employeeId,
          prompt,
          reasoning_effort: this.composerReasoningEffort,
          trigger: 'conversation',
        })
        this.runs.upsertRun(run)

        runInAction(() => {
          this.composerValue = ''
          this.runId = run.id
        })

        await this.onRunCreated?.(run.id)
        return true
      }

      const run = this.run
      if (!run) return false

      const nextRun = await runsApi.appendMessage(run.id, {
        prompt,
        reasoning_effort: this.composerReasoningEffort,
      })
      this.runs.upsertRun(nextRun)

      runInAction(() => {
        this.composerValue = ''
      })

      return true
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to send this message.'
      })

      return false
    } finally {
      runInAction(() => {
        this.isSending = false
      })
    }
  }

  public async cancel() {
    if (!this.runId || !this.canCancel || this.isCancelling) return false

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
