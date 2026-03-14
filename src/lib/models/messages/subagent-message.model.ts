import { makeObservable, observable } from 'mobx'
import type { SubAgentArgs, SubagentDetails, ToolUseResponse } from '@/bindings'
import type { MessageStatus, SubAgentMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class SubAgentMessageModel extends BaseMessageModel {
  public turnId: string
  public stepId: string
  public status: MessageStatus
  public results?: ToolUseResponse[]
  public input: SubAgentArgs
  public subagentDetails: SubagentDetails

  constructor(model: SubAgentMessage) {
    super(model.id, MessageType.SubAgent, model.tokenUsage ?? 0, model.createdAt)
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.status = model.status
    this.results = model.results
    this.input = model.input
    this.subagentDetails = model.subagentDetails

    makeObservable(this, {
      input: observable,
      results: observable,
      status: observable,
      stepId: observable,
      subagentDetails: observable,
      turnId: observable,
    })
  }

  updateFrom = (model: SubAgentMessage) => {
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.status = model.status
    this.results = model.results
    this.input = model.input
    this.subagentDetails = model.subagentDetails
    this.tokenUsage = model.tokenUsage ?? 0
  }
}
