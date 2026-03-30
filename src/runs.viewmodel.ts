import { makeAutoObservable, runInAction } from 'mobx'
import type { RunDto } from '@/bindings/RunDto'
import type { RunStreamMessageDto } from '@/bindings/RunStreamMessageDto'
import type { RunSummaryDto } from '@/bindings/RunSummaryDto'
import { runsApi } from '@/lib/api/runs'
import { RunModel } from '@/models/run.model'
import { RunSummaryModel } from '@/models/run-summary.model'

const API_BASE_URL = import.meta.env?.VITE_API_URL ?? 'http://localhost:9171/api/v1'

export class RunsViewmodel {
  public summaries = new Map<string, RunSummaryModel>()
  public details = new Map<string, RunModel>()
  public recentRunIds: string[] = []
  public runningRunIds: string[] = []
  public isConnected = false
  public connectionError: string | null = null
  private socket: WebSocket | null = null
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private reconnectDelayMs = 1000
  private employeeId: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get recentRuns() {
    return this.recentRunIds.map((id) => this.summaries.get(id)).filter((run): run is RunSummaryModel => Boolean(run))
  }

  public get runningRuns() {
    return this.runningRunIds.map((id) => this.summaries.get(id)).filter((run): run is RunSummaryModel => Boolean(run))
  }

  public connect(employeeId: string) {
    this.employeeId = employeeId
    this.disconnect(false)
    this.openSocket()
  }

  public disconnect(resetEmployeeId = true) {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer)
    this.reconnectTimer = null
    if (this.socket) this.socket.close()
    this.socket = null

    runInAction(() => {
      this.isConnected = false
    })

    if (resetEmployeeId) this.employeeId = null
  }

  public async loadPage(page: number, perPage: number) {
    const response = await runsApi.list(page, perPage)
    runInAction(() => {
      response.items.forEach((item) => this.upsertSummary(item))
    })

    return response
  }

  public async loadRun(id: string) {
    const run = await runsApi.get(id)
    runInAction(() => {
      this.upsertDetail(run)
    })

    return this.details.get(id) ?? null
  }

  public getRun(id: string) {
    return this.details.get(id) ?? null
  }

  public getSummary(id: string) {
    return this.summaries.get(id) ?? null
  }

  public latestActivity(runId: string) {
    const run = this.details.get(runId)
    const lastTurn = run?.turns.at(-1)
    const lastStep = lastTurn?.steps.at(-1)
    const lastContent = lastStep?.response.contents.at(-1) ?? lastStep?.request.contents.at(-1)
    if (!lastContent) return null

    if ('Text' in lastContent) return lastContent.Text.text
    if ('Thinking' in lastContent) return 'Thinking'
    if ('ToolUse' in lastContent) return `Calling ${lastContent.ToolUse.tool_id}`
    if ('ToolResult' in lastContent) return `Finished ${lastContent.ToolResult.tool_id}`
    if ('Image64' in lastContent) return 'Shared image'

    return null
  }

  private openSocket() {
    if (!this.employeeId) return

    const url = new URL(API_BASE_URL)
    url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
    url.pathname = `${url.pathname.replace(/\/$/, '')}/runs/stream`
    url.searchParams.set('employee_id', this.employeeId)

    const socket = new WebSocket(url.toString())
    this.socket = socket

    socket.onopen = () => {
      runInAction(() => {
        this.isConnected = true
        this.connectionError = null
        this.reconnectDelayMs = 1000
      })
    }

    socket.onmessage = (event) => {
      const payload = JSON.parse(event.data) as RunStreamMessageDto
      runInAction(() => {
        this.applyMessage(payload)
      })
    }

    socket.onerror = () => {
      runInAction(() => {
        this.connectionError = 'Live run updates disconnected.'
      })
    }

    socket.onclose = () => {
      runInAction(() => {
        this.isConnected = false
      })
      this.scheduleReconnect()
    }
  }

  private scheduleReconnect() {
    if (!this.employeeId) return
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer)

    this.reconnectTimer = setTimeout(() => {
      this.openSocket()
    }, this.reconnectDelayMs)
    this.reconnectDelayMs = Math.min(this.reconnectDelayMs * 2, 10_000)
  }

  private applyMessage(message: RunStreamMessageDto) {
    switch (message.type) {
      case 'snapshot':
        message.snapshot.recent_runs.forEach((run) => this.upsertSummary(run))
        message.snapshot.running_runs.forEach((run) => this.upsertSummary(run))
        message.snapshot.running_run_details.forEach((run) => this.upsertDetail(run))
        this.recomputeRecentRuns()
        this.recomputeRunningRuns()
        return
      case 'summary_upsert':
        this.upsertSummary(message.run)
        this.recomputeRecentRuns()
        this.recomputeRunningRuns()
        return
      case 'detail_upsert':
        this.upsertDetail(message.run)
        this.recomputeRecentRuns()
        this.recomputeRunningRuns()
        return
    }
  }

  private upsertSummary(run: RunSummaryDto) {
    this.summaries.set(run.id, new RunSummaryModel(run))
  }

  private upsertDetail(run: RunDto) {
    this.details.set(run.id, new RunModel(run))
    this.upsertSummary({
      id:           run.id,
      employee_id:  run.employee_id,
      status:       run.status,
      trigger:      run.trigger,
      created_at:   run.created_at,
      started_at:   run.started_at,
      completed_at: run.completed_at,
    })
  }

  private recomputeRecentRuns() {
    this.recentRunIds = [...this.summaries.values()]
      .sort((left, right) => right.createdAt.getTime() - left.createdAt.getTime())
      .slice(0, 5)
      .map((run) => run.id)
  }

  private recomputeRunningRuns() {
    this.runningRunIds = [...this.summaries.values()]
      .filter((run) => run.status === 'Running')
      .sort(
        (left, right) => (right.startedAt ?? right.createdAt).getTime() - (left.startedAt ?? left.createdAt).getTime(),
      )
      .map((run) => run.id)
  }
}
