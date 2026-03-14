import type { Dayjs } from 'dayjs'
import { makeObservable, observable } from 'mobx'
import { tauriSessionApi } from '@/lib/api/tauri/session.api'

export enum MessageType {
  Prompt = 'prompt',
  Response = 'response',
  Terminal = 'terminal',
  Thinking = 'thinking',
  ToolUse = 'tool_use',
  SubAgent = 'subagent',
  Signal = 'signal',
  QuestionAnswer = 'question_answer',
}

export class BaseMessageModel {
  public id: string
  public type: MessageType
  public tokenUsage: number
  public createdAt: Dayjs

  constructor(id: string, type: MessageType, tokenUsage: number, createdAt: Dayjs) {
    this.id = id
    this.type = type
    this.tokenUsage = tokenUsage
    this.createdAt = createdAt

    makeObservable(this, {
      createdAt: observable,
      id: observable,
      tokenUsage: observable,
      type: observable,
    })
  }

  delete = async () => {
    await tauriSessionApi.deleteMessage(this.id)
  }
}
