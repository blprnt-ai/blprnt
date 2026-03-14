import { makeObservable, observable } from 'mobx'
import type { TerminalSnapshot } from '@/bindings'
import type { TerminalMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class TerminalMessageModel extends BaseMessageModel {
  public rows: number
  public cols: number
  public lines: string[]
  public terminalId: string

  constructor(model: TerminalMessage) {
    super(model.id, MessageType.Terminal, 0, model.createdAt)

    this.rows = model.rows
    this.cols = model.cols
    this.lines = model.lines
    this.terminalId = model.terminalId

    makeObservable(this, {
      cols: observable,
      lines: observable,
      rows: observable,
    })
  }

  updateFrom = (model: TerminalSnapshot) => {
    this.rows = Number(model.rows)
    this.cols = Number(model.cols)
    this.lines = model.lines
  }
}
