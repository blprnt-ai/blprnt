import type { IMapEntry } from 'mobx'
import type { MessageType } from '@/types'

export enum MessageBucketType {
  Conversation = 'conversation',
  ChainOfThought = 'chain_of_thought',
  Terminal = 'terminal',
  SubAgent = 'subagent',
  CompactSummary = 'compact_summary',
  QuestionAnswer = 'question_answer',
}

export interface MessageBucket {
  type: MessageBucketType
  messageKeys: string[]
}

const isConversationMessage = (messageType: MessageType['type']) => {
  return messageType === 'prompt' || messageType === 'response' || messageType === 'signal'
}

const isSubAgentMessage = (messageType: MessageType['type']) => {
  return messageType === 'subagent'
}

const isCompactSummaryMessage = (messageType: MessageType['type']) => {
  return messageType === 'compact_summary'
}

const isQuestionAnswerMessage = (messageType: MessageType['type']) => {
  return messageType === 'question_answer'
}

const isTerminalMessage = (messageType: MessageType['type']) => {
  return messageType === 'terminal'
}

const getMessageBucketType = (messageType: MessageType['type']) => {
  if (isConversationMessage(messageType)) {
    return MessageBucketType.Conversation
  } else if (isSubAgentMessage(messageType)) {
    return MessageBucketType.SubAgent
  } else if (isCompactSummaryMessage(messageType)) {
    return MessageBucketType.CompactSummary
  } else if (isQuestionAnswerMessage(messageType)) {
    return MessageBucketType.QuestionAnswer
  } else if (isTerminalMessage(messageType)) {
    return MessageBucketType.Terminal
  } else {
    return MessageBucketType.ChainOfThought
  }
}

export const bucketizeMessages = (messageKeys: IMapEntry<string, MessageType['type']>[]): MessageBucket[] => {
  return messageKeys.reduce((acc, [messageKey, messageType]) => {
    const messageBucketType = getMessageBucketType(messageType)

    if (acc.length === 0) {
      acc.push({
        messageKeys: [messageKey],
        type: messageBucketType,
      })

      return acc
    }

    const current = acc[acc.length - 1]
    const isSameBucketType = current.type === messageBucketType

    if (isSameBucketType) {
      current.messageKeys.push(messageKey)
    } else {
      acc.push({
        messageKeys: [messageKey],
        type: messageBucketType,
      })
    }

    return acc
  }, [] as MessageBucket[])
}
