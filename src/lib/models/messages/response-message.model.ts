import { makeObservable, observable } from 'mobx'
import type { MessageRole, MessageStatus, ResponseMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class ResponseMessageModel extends BaseMessageModel {
  public turnId: string
  public stepId: string
  public role: MessageRole
  public status: MessageStatus
  public content: string

  constructor(model: ResponseMessage) {
    super(model.id, MessageType.Response, model.tokenUsage ?? 0, model.createdAt)
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

  updateFrom = (model: ResponseMessage) => {
    this.turnId = model.turnId
    this.stepId = model.stepId
    this.role = model.role
    this.status = model.status
    this.content = model.content
    this.tokenUsage = model.tokenUsage ?? 0
  }
}
