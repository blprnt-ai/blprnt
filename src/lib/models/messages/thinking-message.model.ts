import { makeObservable, observable } from 'mobx'
import type { MessageRole, MessageStatus, ThinkingMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class ThinkingMessageModel extends BaseMessageModel {
  public turnId: string
  public stepId: string
  public role: MessageRole
  public status: MessageStatus
  public content: string

  constructor(model: ThinkingMessage) {
    super(model.id, MessageType.Thinking, model.tokenUsage ?? 0, model.createdAt)
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.role = model.role
    this.status = model.status
    this.content = model.content

    makeObservable(this, {
      content: observable,
      role: observable,
      status: observable,
      stepId: observable,
      turnId: observable,
    })
  }

  updateFrom = (model: ThinkingMessage) => {
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.role = model.role
    this.status = model.status
    this.content = model.content
  }
}
