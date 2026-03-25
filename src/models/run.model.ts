import type { RunDto } from '@/bindings/RunDto'
import type { RunStatus } from '@/bindings/RunStatus'
import type { RunTrigger } from '@/bindings/RunTrigger'
import { TurnModel } from './turn.model'

export class RunModel {
  public id: string
  public createdAt: Date
  public status: RunStatus
  public trigger: RunTrigger
  public turns: TurnModel[]
  public startedAt: Date
  public completedAt: Date

  constructor(run: RunDto) {
    this.id = run.id
    this.createdAt = new Date(run.created_at)
    this.status = run.status
    this.trigger = run.trigger
    this.turns = run.turns.map((turn) => new TurnModel(turn))
    this.startedAt = new Date(run.started_at ?? '')
    this.completedAt = new Date(run.completed_at ?? '')
  }
}
