import type { TurnDto } from '@/bindings/TurnDto'
import { StepModel } from './step.model'

export class TurnModel {
  public id: string
  public steps: StepModel[]
  public createdAt: Date

  constructor(turn: TurnDto) {
    this.id = turn.id
    this.steps = turn.steps.map((step) => new StepModel(step))
    this.createdAt = new Date(turn.created_at)
  }
}
