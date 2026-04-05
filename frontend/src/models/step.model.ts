import type { TurnStep } from '@/bindings/TurnStep'
import type { TurnStepContents } from '@/bindings/TurnStepContents'
import type { TurnStepStatus } from '@/bindings/TurnStepStatus'
import { UsageMetricsModel } from './usage-metrics.model'

export class StepModel {
  public request: TurnStepContents
  public response: TurnStepContents
  public status: TurnStepStatus
  public usage: UsageMetricsModel
  public createdAt: Date
  public completedAt: Date | null

  constructor(step: TurnStep) {
    this.request = step.request
    this.response = step.response
    this.status = step.status
    this.usage = new UsageMetricsModel(step.usage)
    this.createdAt = new Date(step.created_at)
    this.completedAt = step.completed_at ? new Date(step.completed_at) : null
  }
}
