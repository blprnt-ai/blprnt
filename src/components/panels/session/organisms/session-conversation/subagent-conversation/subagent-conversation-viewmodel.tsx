import { flow, makeAutoObservable, observable } from 'mobx'
import { createContext, useContext } from 'react'
import type {
  AskQuestionArgs,
  AskQuestionPayload,
  LlmEvent,
  ReasoningDone,
  ReasoningFinal,
  ReasoningStarted,
  ReasoningTextDelta,
  Response,
  ResponseDelta,
  ResponseDone,
  ResponseStarted,
  SubAgentArgs,
  ToolCallCompleted,
  ToolCallStarted,
  ToolUseResponse,
} from '@/bindings'
import { EventType, globalEventBus } from '@/lib/events'
import { createMessageModel, getHistory, listMessages, type MessageModel } from '@/lib/models/messages/message-factory'
import { QuestionAnswerMessageModel } from '@/lib/models/messages/question-answer-message.model'
import { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SubAgentMessageModel } from '@/lib/models/messages/subagent-message.model'
import { ThinkingMessageModel } from '@/lib/models/messages/thinking-message.model'
import { ToolUseMessageModel } from '@/lib/models/messages/tool-use-message.model'
import { bucketizeMessages, type MessageBucket } from '@/lib/utils/message-utils'
import { asyncWait, toDayJs } from '@/lib/utils/misc'
import { parseInput, sanitizeContent } from '@/lib/utils/session-utils'
import type {
  MessageType,
  QuestionAnswerMessage,
  ResponseMessage,
  SubAgentMessage,
  ThinkingMessage,
  ToolUseMessage,
} from '@/types'

export class SubagentConversationViewmodel {
  public isLoading = true
  public isAtBottom = true
  public scrollTop = 0
  public buckets = observable.array<MessageBucket>()
  public messages = observable.map<string, MessageModel>()
  private subagentToolCallToMessageId = new Map<string, string>()
  public unsubscribers: Array<() => void> = []

  constructor(public readonly sessionId: string) {
    makeAutoObservable(this)
  }

  init = async () => {
    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionLlm, (event) => {
        if (event.payload.sessionId !== this.sessionId) return
        this.handleLlmEvent(event.payload.event as LlmEvent)
      }),
    )

    await asyncWait(500)
    await this.list()
  }

  destroy = () => {
    this.unsubscribers.forEach((unsubscribe) => unsubscribe())
    this.unsubscribers = []
  }

  list = flow(function* (this: SubagentConversationViewmodel) {
    const history = yield getHistory(this.sessionId)
    const messages: [string, MessageModel][] = yield listMessages(history)
    this.messages.replace(messages)

    const bucketEntries: [string, MessageType['type']][] = []
    this.messages.forEach((message, key) => {
      bucketEntries.push([key, message.type as MessageType['type']])
    })
    this.buckets.replace(bucketizeMessages(bucketEntries))
    this.isLoading = false
  })

  getTokenUsageFromMessage = (messageId: string) => {
    const originalMessage = this.messages.get(messageId)
    if (!originalMessage) return 0
    const messages = this.messages
      .values()
      .toArray()
      .filter((message) => message.createdAt <= originalMessage.createdAt)
      .toReversed()

    for (const message of messages) {
      if (message.tokenUsage > 0) return message.tokenUsage
    }

    return 0
  }

  private handleLlmEvent = (event: LlmEvent) => {
    switch (event.type) {
      case 'responseStarted':
        this.handleResponseStarted(event as ResponseStarted)
        break
      case 'responseDelta':
        this.handleResponseDelta(event as ResponseDelta)
        break
      case 'response':
        this.handleResponse(event as unknown as Response)
        break
      case 'responseDone':
        this.handleResponseDone(event as ResponseDone)
        break
      case 'reasoningStarted':
        this.handleReasoningStarted(event as ReasoningStarted)
        break
      case 'reasoningDelta':
        this.handleReasoningDelta(event as ReasoningTextDelta)
        break
      case 'reasoning':
        this.handleReasoning(event as ReasoningFinal)
        break
      case 'reasoningDone':
        this.handleReasoningDone(event as ReasoningDone)
        break
      case 'toolCallStarted':
        this.handleToolCallStarted(event as ToolCallStarted)
        break
      case 'toolCallCompleted':
        this.handleToolCallCompleted(event as ToolCallCompleted)
        break
    }
  }

  setIsAtBottom = (isAtBottom: boolean) => {
    this.isAtBottom = isAtBottom
  }

  setScrollTop = (scrollTop: number) => {
    this.scrollTop = scrollTop
  }

  private addMessage = (id: string, message: MessageModel) => {
    this.messages.set(id, message)
    const entries = Array.from(this.messages.entries()).map(
      ([key, message]) => [key, message.type as MessageType['type']] as [string, MessageType['type']],
    )

    this.buckets.replace(bucketizeMessages(entries))
  }

  private handleResponseStarted = (payload: ResponseStarted) => {
    if (this.messages.has(payload.id)) return

    const message: ResponseMessage = {
      ...payload,
      content: '',
      createdAt: toDayJs(new Date().toISOString()),
      role: 'assistant',
      status: 'pending',
      type: 'response',
    }

    this.addMessage(message.id, createMessageModel(message))
  }

  private handleResponseDelta = (payload: ResponseDelta) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ResponseMessageModel)) return

    message.content += payload.delta
  }

  private handleResponse = (payload: Response) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ResponseMessageModel)) return

    message.content = sanitizeContent(payload.content)
  }

  private handleResponseDone = (payload: ResponseDone) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ResponseMessageModel)) return

    message.status = 'completed'
  }

  private handleReasoningStarted = (payload: ReasoningStarted) => {
    if (this.messages.has(payload.id)) return

    const message: ThinkingMessage = {
      ...payload,
      content: '',
      createdAt: toDayJs(new Date().toISOString()),
      role: 'assistant',
      status: 'pending',
      type: 'thinking',
    }

    this.addMessage(message.id, createMessageModel(message))
  }

  private handleReasoningDelta = (payload: ReasoningTextDelta) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ThinkingMessageModel)) return

    message.status = 'in_progress'
    message.content += payload.delta
  }

  private handleReasoning = (payload: ReasoningFinal) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ThinkingMessageModel)) return

    message.content = payload.reasoning
  }

  private handleReasoningDone = (payload: ReasoningDone) => {
    const message = this.messages.get(payload.id)
    if (!(message instanceof ThinkingMessageModel)) return

    message.status = 'completed'
  }

  private handleToolCallStarted = (payload: ToolCallStarted) => {
    const input = parseInput(payload.args) as unknown as SubAgentArgs | AskQuestionArgs

    if (payload.toolId === 'ask_question') {
      const questionId = payload.questionId ?? payload.id
      if (this.messages.has(questionId)) return

      const questionArgs = input as AskQuestionArgs
      const message: QuestionAnswerMessage = {
        answer: '',
        createdAt: toDayJs(new Date().toISOString()),
        details: questionArgs.details,
        id: questionId,
        options: questionArgs.options,
        question: questionArgs.question,
        type: 'question_answer',
      }

      this.addMessage(message.id, createMessageModel(message))
      return
    }

    if (this.messages.has(payload.id)) return

    if (payload.toolId === 'subagent') {
      const existing = this.findSubagentMessage(payload.subagentDetails?.sessionId)
      if (existing) {
        existing.status = 'in_progress'
        this.subagentToolCallToMessageId.set(payload.id, existing.id)
        return
      }

      const message: SubAgentMessage = {
        ...payload,
        createdAt: toDayJs(new Date().toISOString()),
        input: input as SubAgentArgs,
        status: 'in_progress',
        subagentDetails: payload.subagentDetails!,
        type: 'subagent',
      }

      this.subagentToolCallToMessageId.set(payload.id, message.id)
      this.addMessage(message.id, createMessageModel(message))
      return
    }

    const message: ToolUseMessage = {
      ...payload,
      createdAt: toDayJs(new Date().toISOString()),
      input,
      status: 'in_progress',
      type: 'tool_use',
    }

    this.addMessage(message.id, createMessageModel(message))
  }

  private handleToolCallCompleted = (payload: ToolCallCompleted) => {
    const toolResult = payload.content as ToolUseResponse
    const canonicalId = this.subagentToolCallToMessageId.get(payload.id) ?? payload.id
    const message = this.messages.get(canonicalId) ?? this.messages.get(payload.itemId)

    if (!message) return

    if (message instanceof SubAgentMessageModel) {
      message.results = message.results ? [...message.results, toolResult] : [toolResult]
      message.status = toolResult.type === 'error' ? 'error' : 'completed'
      return
    }

    if (message instanceof QuestionAnswerMessageModel) {
      if (toolResult.type === 'error') return
      const payloadData = toolResult.data as AskQuestionPayload
      message.answer = payloadData.answer ?? ''
      return
    }

    if (message instanceof ToolUseMessageModel) {
      message.result = toolResult
      message.status = toolResult.type === 'error' ? 'error' : 'completed'
    }
  }

  private findSubagentMessage = (sessionId?: string | null) => {
    if (!sessionId) return undefined

    return Array.from(this.messages.values()).find(
      (message): message is SubAgentMessageModel =>
        message instanceof SubAgentMessageModel && message.subagentDetails.sessionId === sessionId,
    )
  }
}

export const SubagentConversationViewmodelContext = createContext<SubagentConversationViewmodel | null>(null)

export const useSubagentConversationViewmodel = () => {
  const viewmodel = useContext(SubagentConversationViewmodelContext)
  if (!viewmodel) throw new Error('useSubagentConversationViewmodel must be used within SubagentConversationProvider')

  return viewmodel
}
