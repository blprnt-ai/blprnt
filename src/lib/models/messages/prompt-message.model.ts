import { makeObservable, observable } from 'mobx'
import type { PromptStarted } from '@/bindings'
import type { MessageRole, MessageStatus, PromptMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class PromptMessageModel extends BaseMessageModel {
  public turnId: string
  public content: string
  public status: MessageStatus
  public role: MessageRole
  public imageUrls: string[]

  constructor(model: PromptMessage) {
    super(model.id, MessageType.Prompt, model.tokenUsage ?? 0, model.createdAt)

    this.turnId = model.turnId
    this.content = model.content
    this.status = model.status
    this.role = model.role
    this.imageUrls = model.imageUrls ?? []

    makeObservable(this, {
      content: observable,
      imageUrls: observable,
      role: observable,
      status: observable,
      turnId: observable,
    })
  }

  updateFrom = (model: PromptMessage) => {
    this.turnId = model.turnId
    this.content = model.content
    this.status = model.status
    this.role = model.role
    this.imageUrls = model.imageUrls ?? []
    this.tokenUsage = model.tokenUsage ?? 0
  }

  updateFromStarted = (model: PromptStarted) => {
    this.id = model.id
    this.turnId = model.turnId
    this.status = 'completed'
  }
}
