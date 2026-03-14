import { makeObservable, observable } from 'mobx'
import type { ToolId, ToolUseResponse } from '@/bindings'
import type { MessageStatus, ToolUseMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class ToolUseMessageModel extends BaseMessageModel {
  public turnId: string
  public stepId: string
  public toolId: ToolId
  public input: unknown
  public result?: ToolUseResponse
  public status: MessageStatus

  constructor(model: ToolUseMessage) {
    super(model.id, MessageType.ToolUse, model.tokenUsage ?? 0, model.createdAt)
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.toolId = model.toolId
    this.input = model.input
    this.result = model.result
    this.status = model.status

    makeObservable(this, {
      input: observable,
      result: observable,
      status: observable,
      stepId: observable,
      toolId: observable,
      turnId: observable,
    })
  }

  updateFrom = (model: ToolUseMessage) => {
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.toolId = model.toolId
    this.input = model.input
    this.result = model.result
    this.status = model.status
  }
}
