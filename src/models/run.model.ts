import type { RunDto } from '@/bindings/RunDto'
import type { RunStatus } from '@/bindings/RunStatus'
import type { RunTrigger } from '@/bindings/RunTrigger'
import { TurnModel } from './turn.model'

export class RunModel {
  public id: string
  public employeeId: string
  public createdAt: Date
  public status: RunStatus
  public trigger: RunTrigger
  public turns: TurnModel[]
  public startedAt: Date | null
  public completedAt: Date | null

  constructor(run: RunDto) {
    this.id = run.id
    this.employeeId = run.employee_id
    this.createdAt = new Date(run.created_at)
    this.status = run.status
    this.trigger = run.trigger
    this.turns = run.turns.map((turn) => new TurnModel(turn))
    this.startedAt = run.started_at ? new Date(run.started_at) : null
    this.completedAt = run.completed_at ? new Date(run.completed_at) : null
  }
}
