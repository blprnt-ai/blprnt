import dayjs from 'dayjs'
import type {
  AskQuestionPayload,
  HistoryVisibility,
  MessageRecord,
  TerminalPayload,
  ToolId,
  ToolUseResponse,
} from '@/bindings'
import { tauriSessionApi } from '@/lib/api/tauri/session.api'
import { toDayJs } from '@/lib/utils/misc'
import {
  toImage64Message,
  toPromptMessage,
  toResponseMessage,
  toSignalMessage,
  toThinkingMessage,
  toToolUseMessage,
} from '@/lib/utils/session-utils'
import type { MessageType as MessageTypeLegacy, SubAgentMessage, TerminalMessage } from '@/types'
import { PromptMessageModel } from './prompt-message.model'
import { QuestionAnswerMessageModel } from './question-answer-message.model'
import { ResponseMessageModel } from './response-message.model'
import { SignalMessageModel } from './signal-message.model'
import { SubAgentMessageModel } from './subagent-message.model'
import { TerminalMessageModel } from './terminal-message.model'
import { ThinkingMessageModel } from './thinking-message.model'
import { ToolUseMessageModel } from './tool-use-message.model'

export type MessageModel =
  | PromptMessageModel
  | ResponseMessageModel
  | ThinkingMessageModel
  | ToolUseMessageModel
  | SubAgentMessageModel
  | SignalMessageModel
  | QuestionAnswerMessageModel
  | TerminalMessageModel

const hiddenToolIds = new Set<ToolId>(['apply_skill', 'list_skills', 'get_reference'])

const toMessage = (model: MessageRecord): MessageTypeLegacy | undefined => {
  switch (model.content.type) {
    case 'text':
      if (model.role === 'user') return toPromptMessage(model)
      return toResponseMessage(model)
    case 'image64':
      return toImage64Message(model)
    case 'thinking':
      return toThinkingMessage(model)
    case 'tool_use':
      return toToolUseMessage(model)
    case 'info':
    case 'warning':
    case 'error':
      return toSignalMessage(model)
    default:
      return undefined
  }
}

export const createMessageModel = (message: MessageTypeLegacy): MessageModel => {
  switch (message.type) {
    case 'prompt':
      return new PromptMessageModel(message)
    case 'response':
      return new ResponseMessageModel(message)
    case 'thinking':
      return new ThinkingMessageModel(message)
    case 'tool_use':
      return new ToolUseMessageModel(message)
    case 'subagent':
      return new SubAgentMessageModel(message)
    case 'signal':
      return new SignalMessageModel(message)
    case 'question_answer':
      return new QuestionAnswerMessageModel(message)
    case 'terminal':
      return new TerminalMessageModel(message)
    default:
      throw new Error(`Unknown message type: ${message.type}`)
  }
}

export const getHistory = async (sessionId: string) => tauriSessionApi.listMessages(sessionId)

export const listMessages = (history: MessageRecord[], isSubagent = false): [string, MessageModel][] => {
  const assistantVisibility = 'assistant' as HistoryVisibility
  const filteredHistory = history.filter((message) => message.visibility !== assistantVisibility)
  const thirtyMinutesAgo = dayjs().subtract(30, 'minutes')

  const toolResults = filteredHistory.reduce(
    (acc, message) => {
      if (message.content.type !== 'tool_result') return acc
      acc[message.content.tool_use_id] = message.content.content

      return acc
    },
    {} as Record<string, ToolUseResponse>,
  )

  const sortedResults = !isSubagent
    ? filteredHistory
    : filteredHistory.toSorted((a, b) => (toDayJs(a.updated_at).isAfter(toDayJs(b.updated_at)) ? 1 : -1))

  const seenSubagentSessionIds = new Set<string>()
  const seenTerminalIds = new Set<string>()

  const messages = sortedResults.reduce((acc, model) => {
    const message = toMessage(model)
    if (!message) return acc

    if (message.type === 'image64' && acc.at(-1)?.type === 'prompt') {
      const lastMessage = acc.at(-1) as PromptMessageModel
      lastMessage.imageUrls.push(message.content)
      return acc
    }

    if (message.type === 'subagent') {
      const toolResult = toolResults[message.id]
      const subagentSessionId = message.subagentDetails?.sessionId

      if (!subagentSessionId) {
        acc.push(message)
        return acc
      }

      if (seenSubagentSessionIds.has(subagentSessionId)) {
        const lastSubagentMessageIndex = acc.findIndex(
          (entry) => entry.type === 'subagent' && entry.subagentDetails.sessionId === subagentSessionId,
        )

        if (lastSubagentMessageIndex !== -1) {
          const lastSubagentMessage = acc[lastSubagentMessageIndex] as SubAgentMessage
          if (toolResult) lastSubagentMessage.results?.push(toolResult)

          acc.splice(lastSubagentMessageIndex, 1)
          acc.push(lastSubagentMessage)
        }
      } else {
        seenSubagentSessionIds.add(subagentSessionId)

        if (toolResult) (message as SubAgentMessage).results = [toolResult]
        acc.push(message)
      }
    } else if (message.type === 'terminal') {
      const toolResult = toolResults[message.id]
      const isError = toolResult?.type === 'error'
      if (!toolResult || isError) {
        acc.push(message)
        return acc
      }

      const payload = toolResult.data as TerminalPayload
      const terminalId = payload.terminal_id
      if (seenTerminalIds.has(terminalId)) {
        const lastTerminalMessageIndex = acc.findIndex(
          (entry) => entry.type === 'terminal' && entry.terminalId === terminalId,
        )
        if (lastTerminalMessageIndex !== -1) {
          const lastTerminalMessage = acc[lastTerminalMessageIndex] as TerminalMessage
          lastTerminalMessage.lines = payload.snapshot?.lines ?? []
          lastTerminalMessage.cols = Number(payload.snapshot?.cols ?? 0)
          lastTerminalMessage.rows = Number(payload.snapshot?.rows ?? 0)
          lastTerminalMessage.createdAt = toDayJs(model.created_at)

          acc.splice(lastTerminalMessageIndex, 1)
          acc.push(lastTerminalMessage)
        }
      } else {
        seenTerminalIds.add(terminalId)
        message.terminalId = terminalId
        message.lines = payload.snapshot?.lines ?? []
        message.cols = Number(payload.snapshot?.cols ?? 0)
        message.rows = Number(payload.snapshot?.rows ?? 0)
        acc.push(message)
      }
    } else {
      acc.push(message)
    }

    return acc
  }, [] as MessageTypeLegacy[])

  return messages
    .map((message) => {
      if (!message || message.type === 'image64') return undefined
      if (message.type === 'tool_use' && hiddenToolIds.has(message.toolId)) return undefined

      if (message.type === 'subagent') {
        const mostRecentResult = message.results?.at(-1)
        if (!mostRecentResult) {
          if (message.createdAt && dayjs(message.createdAt).isBefore(thirtyMinutesAgo)) {
            message.status = 'error'
          } else {
            message.status = 'in_progress'
          }
        } else {
          const isError = mostRecentResult.type === 'error'
          message.status = isError ? 'error' : 'completed'
        }
      } else if (message.type === 'question_answer') {
        const toolResult = toolResults[message.id]
        if (!toolResult) {
          message.answer = undefined
        } else {
          const isError = toolResult.type === 'error'
          if (isError) return undefined

          const payload = toolResult.data as AskQuestionPayload
          message.answer = payload.answer ?? ''
        }
      } else if (message.type === 'tool_use') {
        message.result = toolResults[message.id]
      }

      return [message.id, createMessageModel(message)] as const
    })
    .filter((message): message is [string, MessageModel] => message !== undefined) as [string, MessageModel][]
}
