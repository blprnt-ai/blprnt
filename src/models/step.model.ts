import type { TurnStep } from '@/bindings/TurnStep'
import type { TurnStepContents } from '@/bindings/TurnStepContents'
import type { TurnStepStatus } from '@/bindings/TurnStepStatus'

export class StepModel {
  public contents: TurnStepContents
  public status: TurnStepStatus
  public createdAt: Date
  public completedAt: Date

  constructor(step: TurnStep) {
    this.contents = step.contents
    this.status = step.status
    this.createdAt = new Date(step.created_at)
    this.completedAt = new Date(step.completed_at ?? '')
  }
}
