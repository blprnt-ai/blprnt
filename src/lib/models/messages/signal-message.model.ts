import { makeObservable, observable } from 'mobx'
import type { MessageRole, SignalMessage, SignalType } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class SignalMessageModel extends BaseMessageModel {
  public content: string
  public error: SignalMessage['error']
  public signalType: SignalType
  public role: MessageRole
  public deleteId?: string

  constructor(model: SignalMessage) {
    super(model.id, MessageType.Signal, 0, model.createdAt)
    this.content = model.content
    this.error = model.error
    this.signalType = model.signalType
    this.role = model.role
    this.deleteId = model.deleteId

    makeObservable(this, {
      content: observable,
      deleteId: observable,
      error: observable,
      role: observable,
      signalType: observable,
    })
  }

  updateFrom = (model: SignalMessage) => {
    this.content = model.content
    this.error = model.error
    this.signalType = model.signalType
    this.role = model.role
    this.deleteId = model.deleteId
  }
}
