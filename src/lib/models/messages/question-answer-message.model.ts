import { makeObservable, observable } from 'mobx'
import type { QuestionAnswerMessage } from '@/types'
import { BaseMessageModel, MessageType } from './base-message.model'

export class QuestionAnswerMessageModel extends BaseMessageModel {
  public question: string
  public options: string[]
  public details: string
  public answer?: string

  constructor(model: QuestionAnswerMessage) {
    super(model.id, MessageType.QuestionAnswer, model.tokenUsage ?? 0, model.createdAt)
    this.question = model.question
    this.options = model.options
    this.details = model.details
    this.answer = model.answer

    makeObservable(this, {
      answer: observable,
      details: observable,
      options: observable,
      question: observable,
    })
  }

  updateFrom = (model: QuestionAnswerMessage) => {
    this.question = model.question
    this.options = model.options
    this.details = model.details
    this.answer = model.answer
  }
}
