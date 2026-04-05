import type { TurnDto } from '@/bindings/TurnDto'
import type { ReasoningEffort } from '@/bindings/ReasoningEffort'
import { StepModel } from './step.model'
import { UsageMetricsModel } from './usage-metrics.model'

export class TurnModel {
  public id: string
  public steps: StepModel[]
  public createdAt: Date
  public reasoningEffort: ReasoningEffort | null
  public usage: UsageMetricsModel

  constructor(turn: TurnDto) {
    this.id = turn.id
    this.steps = turn.steps.map((step) => new StepModel(step))
    this.createdAt = new Date(turn.created_at)
    this.reasoningEffort = turn.reasoning_effort ?? null
    this.usage = new UsageMetricsModel(turn.usage)
  }
}
