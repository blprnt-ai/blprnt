import type { RunStatus } from '@/bindings/RunStatus'
import type { RunSummaryDto } from '@/bindings/RunSummaryDto'
import type { RunTrigger } from '@/bindings/RunTrigger'
import { UsageMetricsModel } from './usage-metrics.model'

export class RunSummaryModel {
  public id: string
  public employeeId: string
  public status: RunStatus
  public trigger: RunTrigger
  public createdAt: Date
  public usage: UsageMetricsModel
  public startedAt: Date | null
  public completedAt: Date | null

  constructor(run: RunSummaryDto) {
    this.id = run.id
    this.employeeId = run.employee_id
    this.status = run.status
    this.trigger = run.trigger
    this.createdAt = new Date(run.created_at)
    this.usage = new UsageMetricsModel(run.usage)
    this.startedAt = run.started_at ? new Date(run.started_at) : null
    this.completedAt = run.completed_at ? new Date(run.completed_at) : null
  }
}
