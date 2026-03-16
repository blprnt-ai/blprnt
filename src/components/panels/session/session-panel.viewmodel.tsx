import type { Dayjs } from 'dayjs'
import { flow, makeAutoObservable, observable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type {
  AskQuestionArgs,
  AskQuestionPayload,
  ControlEvent,
  LlmEvent,
  LlmModel,
  MessageRecord,
  PlanCreatePayload,
  PromptDeleted,
  PromptEvent,
  PromptQueued,
  PromptStarted,
  ReasoningDone,
  ReasoningFinal,
  ReasoningStarted,
  ReasoningTextDelta,
  Response,
  ResponseDelta,
  ResponseDone,
  ResponseStarted,
  SignalEvent,
  SubAgentArgs,
  TerminalSnapshot,
  TokenUsage,
  ToolCallCompleted,
  ToolCallStarted,
  ToolUseResponse,
} from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
// eslint-disable-next-line
import { listSlashCommands } from '@/lib/api/slash-commands.api'

import { EventType, globalEventBus } from '@/lib/events'
import { llmModelsModel } from '@/lib/models/llm-models.model'
import { createMessageModel, getHistory, listMessages, type MessageModel } from '@/lib/models/messages/message-factory'
import { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import { QuestionAnswerMessageModel } from '@/lib/models/messages/question-answer-message.model'
import { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SubAgentMessageModel } from '@/lib/models/messages/subagent-message.model'
import { TerminalMessageModel } from '@/lib/models/messages/terminal-message.model'
import { ThinkingMessageModel } from '@/lib/models/messages/thinking-message.model'
import { ToolUseMessageModel } from '@/lib/models/messages/tool-use-message.model'
import { PlanModel } from '@/lib/models/plan.model'
import { SessionModel } from '@/lib/models/session.model'
import type { SlashCommand } from '@/lib/models/slash-command.types'
import type { MessageBucket } from '@/lib/utils/message-utils'
import { bucketizeMessages } from '@/lib/utils/message-utils'
import { asyncWait, toDayJs } from '@/lib/utils/misc'
import { notify } from '@/lib/utils/notifications'
import { parseInput, sanitizeContent } from '@/lib/utils/session-utils'
import type {
  MessageType,
  PromptMessage,
  QuestionAnswerMessage,
  ResponseMessage,
  SignalMessage,
  SubAgentMessage,
  TerminalMessage,
  ThinkingMessage,
  ToolUseMessage,
} from '@/types'

type PendingPromptSubmission = {
  createdAt: string
  localId: string
  prompt: string
  sequence: number
  imageUrls: string[]
}

type QueuedPromptStateItem = {
  queueItemId: string
  createdAt: string
  localId: string
  prompt: string
  sequence: number
  imageUrls: string[]
  isDeleting: boolean
}

export type QueuedPromptListItem = {
  createdAt: Dayjs
  id: string
  imageUrls: string[]
  content: string
  isDeleting: boolean
}

export class SessionPanelViewmodel {
  private readonly llmModelsModel = llmModelsModel
  public session: SessionModel | null = null
  public messages = observable.map<string, MessageModel>()
  public terminals = observable.map<string, TerminalMessageModel>()
  public buckets = observable.array<MessageBucket>()
  public visibleBucketSize = 25
  public prompt = ''
  public slashCommands = observable.array<SlashCommand>()
  public isSlashPickerOpen = false
  public slashQuery = ''
  public slashHighlightIndex = 0
  public isSlashCommandsLoading = false
  private unsubscribers: Array<() => void> = []
  private subagentToolCallToMessageId = new Map<string, string>()
  private pendingPromptQueue: PendingPromptSubmission[] = []
  private queuedPromptByQueueItemId = new Map<string, QueuedPromptStateItem>()
  private pendingPromptSequence = 0
  public imageUrls = observable.map<string, string>()
  public plan: PlanModel | null = null

  public chosenModel: LlmModel | null = null
  private _tokenUsage = 0

  constructor(private readonly id: string) {
    makeAutoObservable(this, { session: false }, { autoBind: true })
  }

  init = flow(function* (this: SessionPanelViewmodel) {
    console.log('init session panel viewmodel')
    this.session = yield SessionModel.get(this.id)
    if (!this.session) return

    yield this.llmModelsModel.loadModels()

    this.chosenModel = this.llmModelsModel.models.find((m) => m.slug === this.session!.modelOverride) ?? null

    yield this.loadSlashCommands()

    const history: MessageRecord[] = yield getHistory(this.id)
    const allMessages = listMessages(history)
    const messages = allMessages.filter(([_, message]) => message.type !== 'terminal')

    this.messages.replace(messages)
    this.setBucketsFromMessages()

    yield this.loadPlansFromSession()

    this.startListening()
  })

  get models() {
    return this.llmModelsModel.models
  }

  loadSlashCommands = flow(function* (this: SessionPanelViewmodel) {
    this.isSlashCommandsLoading = true
    try {
      const commands: SlashCommand[] = yield listSlashCommands()
      this.slashCommands.replace(commands)
    } finally {
      this.isSlashCommandsLoading = false
    }
  })

  reloadSessionPlans = flow(function* (this: SessionPanelViewmodel) {
    if (!this.session) return

    yield this.loadPlansFromSession()
  })

  setSessionPlanAssignment = flow(function* (this: SessionPanelViewmodel, selectedPlanId: string | null) {
    if (!this.session?.projectId) return

    const sessionId = this.session.id
    const projectId = this.session.projectId
    const currentPlanId = this.session.plan?.id ?? null

    if (currentPlanId === selectedPlanId) return

    const affectedPlanIds = new Set<string>()

    if (currentPlanId) {
      yield PlanModel.unassignFromSession(sessionId, currentPlanId)
      affectedPlanIds.add(currentPlanId)
    }

    if (selectedPlanId) {
      yield PlanModel.assignToSession(sessionId, selectedPlanId)
      affectedPlanIds.add(selectedPlanId)
    }

    affectedPlanIds.forEach((planId) => {
      globalEventBus.emit(EventType.Internal, {
        event: {
          planId,
          projectId,
          type: 'plan_updated',
        },
      })
    })

    yield this.reloadSessionPlans()
  })

  destroy = () => {
    this.session?.destroy()
    this.session = null
    this.unsubscribers.forEach((unsub) => unsub())
    this.unsubscribers = []
    this.subagentToolCallToMessageId.clear()
    this.pendingPromptQueue = []
    this.queuedPromptByQueueItemId.clear()
    this.messages.clear()
    this.buckets.clear()
  }

  setPrompt = (prompt: string) => {
    this.prompt = prompt
  }

  get isSlashOpen() {
    // return this.isSlashPickerOpen
    return false
  }

  get slashPickerQuery() {
    return this.slashQuery
  }

  get slashHighlight() {
    return this.slashHighlightIndex
  }

  get isSlashLoading() {
    return this.isSlashCommandsLoading
  }

  get filteredSlashCommands() {
    const query = this.slashQuery.trim().toLowerCase()
    if (!query) return this.slashCommands
    return this.slashCommands.filter((command) => this.isSlashCommandMatch(command, query))
  }

  openSlashPicker = () => {
    this.isSlashPickerOpen = true
    this.slashQuery = ''
    this.slashHighlightIndex = 0
  }

  closeSlashPicker = () => {
    this.isSlashPickerOpen = false
  }

  setSlashQuery = (query: string) => {
    this.slashQuery = query
    this.slashHighlightIndex = 0
  }

  moveSlashHighlight = (delta: number) => {
    const total = this.filteredSlashCommands.length
    if (total === 0) {
      this.slashHighlightIndex = 0
      return
    }

    const nextIndex = this.slashHighlightIndex + delta
    this.slashHighlightIndex = Math.min(Math.max(nextIndex, 0), total - 1)
  }

  autocompleteSlashHighlighted = () => {
    const command = this.filteredSlashCommands[this.slashHighlightIndex]
    if (!command) return
    this.slashQuery = command.name
    this.slashHighlightIndex = 0
  }

  runSlashHighlighted = () => {
    const command = this.filteredSlashCommands[this.slashHighlightIndex]
    if (!command) return
    this.runSlashCommand(command.name)
  }

  runSlashCommand = (commandId: SlashCommand['name']) => {
    const command = this.slashCommands.find((entry) => entry.name === commandId)
    if (!command) return
    this.appendSignalMessage(`Ran /${command.name} (stub)`, 'info')
    this.closeSlashPicker()
  }

  addImageUrl = (url: string, base64: string) => {
    this.imageUrls.set(url, base64)
  }

  removeImageUrl = (url: string) => {
    this.imageUrls.delete(url)
  }

  get isRunning() {
    return !!this.session?.isRunning
  }

  get hasPlan() {
    return !!this.session?.plan?.id
  }

  get isPlanInProgress() {
    return this.plan?.inProgress || this.plan?.todos.some((todo) => todo.status === 'in_progress')
  }

  get totalTokenUsage() {
    if (this._tokenUsage > 0) return this._tokenUsage

    return (
      this.messages
        .values()
        .toArray()
        .filter((message) => message.tokenUsage > 0)
        .map((message) => message.tokenUsage)
        .toReversed()
        .at(0) ?? 0
    )
  }

  getTokenUsageFromMessage = (messageId: string) => {
    const originalMessage = this.messages.get(messageId)
    if (!originalMessage) return 0

    if (originalMessage.tokenUsage > 0) return originalMessage.tokenUsage

    // console.log('originalMessage', (originalMessage as ResponseMessageModel).content)
    const messages = this.messages
      .values()
      .toArray()
      .filter((message) => message.createdAt < originalMessage.createdAt)
      .toReversed()

    for (const message of messages) {
      // console.log('message', (message as ResponseMessageModel).content, message.tokenUsage)
      if (message.tokenUsage > 0) return message.tokenUsage
    }

    return 0
  }

  get percentRemaining() {
    if (!this.chosenModel) return 0

    return 100 - (Number(this.totalTokenUsage) / Number(this.chosenModel.context_length)) * 100
  }

  setChosenModel = (model: LlmModel) => {
    this.chosenModel = model
  }

  submitPrompt = flow(function* (this: SessionPanelViewmodel) {
    if (!this.session) return
    if (!this.prompt.trim()) return

    const prompt = this.prompt
    const imageEntries = Array.from(this.imageUrls.entries())
    const imageUrls = imageEntries.map(([_, base64]) => base64)
    const pendingSubmission: PendingPromptSubmission = {
      createdAt: new Date().toISOString(),
      imageUrls,
      localId: crypto.randomUUID(),
      prompt,
      sequence: this.pendingPromptSequence++,
    }
    this.pendingPromptQueue.push(pendingSubmission)
    this.prompt = ''
    this.imageUrls.clear()

    if (!this.session.status) this.session.status = 'Thinking...'

    try {
      yield this.session.sendPrompt(prompt, imageUrls)
    } catch (error) {
      const pendingIndex = this.pendingPromptQueue.indexOf(pendingSubmission)
      if (pendingIndex !== -1) this.pendingPromptQueue.splice(pendingIndex, 1)

      for (const [queueItemId, queuedPrompt] of this.queuedPromptByQueueItemId.entries()) {
        if (queuedPrompt.localId !== pendingSubmission.localId) continue
        this.clearPendingPromptTracking(queueItemId)
      }

      this.prompt = prompt
      imageEntries.forEach(([url, base64]) => this.imageUrls.set(url, base64))
      throw error
    }
  })

  interrupt = flow(function* (this: SessionPanelViewmodel) {
    if (!this.session) return
    yield this.session.sendInterrupt()

    yield asyncWait(2000)
    const history = yield getHistory(this.id)
    const messages: [string, MessageModel][] = yield listMessages(history)
    this.messages.replace(messages)
    this.setBucketsFromMessages()
  })

  submitAnswer = flow(function* (this: SessionPanelViewmodel, messageId: string, answer: string) {
    if (!this.session) return
    if (!answer.trim()) return

    const result = yield this.session.submitAnswer(messageId, answer)

    if (result?.outcome === 'accepted') {
      const message = this.messages.get(messageId)
      if (message instanceof QuestionAnswerMessageModel) {
        message.answer = answer
      }
      return
    }

    const history = yield getHistory(this.id)
    const messages: [string, MessageModel][] = yield listMessages(history)
    this.messages.replace(messages)
    this.setBucketsFromMessages()
  })

  addMessage = (id: string, message: MessageModel) => {
    const wasAtBottom = !this.hasMoreBuckets
    this.messages.set(id, message)
    this.setBucketsFromMessages()
    if (wasAtBottom) {
      this.visibleBucketSize = this.buckets.length
    }
  }

  removeMessage = (id: string) => {
    this.clearPendingPromptTracking(id)

    if (!this.messages.has(id)) return
    this.messages.delete(id)
    this.removeFromBuckets(id)
  }

  getMessageByKey = (key: string) => {
    return this.messages.get(key)
  }

  get isEmpty() {
    return this.messages.size === 0
  }

  get queuedPrompts() {
    const mappedLocalIds = new Set(
      Array.from(this.queuedPromptByQueueItemId.values()).map((queuedPrompt) => queuedPrompt.localId),
    )

    const queuedFromMappedPending = Array.from(this.queuedPromptByQueueItemId.values()).map((queuedPrompt) => ({
      content: queuedPrompt.prompt,
      createdAt: toDayJs(queuedPrompt.createdAt),
      id: queuedPrompt.queueItemId,
      imageUrls: queuedPrompt.imageUrls,
      isDeleting: queuedPrompt.isDeleting,
      sequence: queuedPrompt.sequence,
    }))

    const queuedFromPendingBuffer = this.pendingPromptQueue.map((submission) => ({
      content: submission.prompt,
      createdAt: toDayJs(submission.createdAt),
      id: submission.localId,
      imageUrls: submission.imageUrls,
      isDeleting: false,
      sequence: submission.sequence,
    }))
    const queuedFromUnmaterializedPendingBuffer = queuedFromPendingBuffer.filter(
      (pendingSubmission) => !mappedLocalIds.has(pendingSubmission.id),
    )

    return [...queuedFromMappedPending, ...queuedFromUnmaterializedPendingBuffer]
      .sort((left, right) => {
        const sequenceDiff = left.sequence - right.sequence
        if (sequenceDiff !== 0) return sequenceDiff
        const timeDiff = left.createdAt.valueOf() - right.createdAt.valueOf()
        if (timeDiff !== 0) return timeDiff
        return left.id.localeCompare(right.id)
      })
      .map(({ content, createdAt, id, imageUrls, isDeleting }) => ({
        content,
        createdAt,
        id,
        imageUrls,
        isDeleting,
      })) as QueuedPromptListItem[]
  }

  deleteMessage = flow(function* (this: SessionPanelViewmodel, messageId: string) {
    if (this.isRunning) return
    const message = this.messages.get(messageId)
    if (!message) return

    yield message.delete()
    this.removeMessage(messageId)
  })

  deleteQueuedPrompt = flow(function* (this: SessionPanelViewmodel, queueItemId: string) {
    const normalizedQueueItemId = this.getPromptQueueItemId({ id: queueItemId, queue_item_id: queueItemId })
    if (!normalizedQueueItemId || !this.session) return

    this.setQueuedPromptDeleting(normalizedQueueItemId, true)

    try {
      const outcome = yield this.session.deleteQueuedPrompt(normalizedQueueItemId)

      switch (outcome) {
        case 'Deleted':
          this.clearPendingPromptTracking(normalizedQueueItemId)
          break
        case 'AlreadyStarted':
          this.setQueuedPromptDeleting(normalizedQueueItemId, false)
          break
        case 'NotFound':
          this.clearPendingPromptTracking(normalizedQueueItemId)
          break
      }
    } catch {
      this.setQueuedPromptDeleting(normalizedQueueItemId, false)
    }
  })

  rewindToMessage = flow(function* (this: SessionPanelViewmodel, messageId: string) {
    if (this.isRunning || !this.session) return

    const didTruncate = this.truncateMessagesTo(messageId)
    if (didTruncate) this.setBucketsFromMessages()

    try {
      yield this.session.rewindTo(messageId)
    } catch {
    } finally {
      const history = yield getHistory(this.id)
      const messages = yield listMessages(history)
      this.messages.replace(messages)
      this.setBucketsFromMessages()
    }
  })

  get visibleBuckets() {
    const startIndex = Math.max(0, this.buckets.length - this.visibleBucketSize)
    return this.buckets.slice(startIndex)
  }

  get terminalBuckets() {
    return this.terminals.values().toArray()
  }

  get hasMoreBuckets() {
    return this.buckets.length > this.visibleBucketSize
  }

  bumpBucketSize = () => {
    const diff = this.buckets.length - this.visibleBucketSize
    if (diff <= 0) return
    this.visibleBucketSize += diff > 25 ? 25 : diff
  }

  setPlan = (plan: PlanModel | null) => {
    this.plan = plan
    if (!this.session) return

    this.session.plan = plan
  }

  getTerminalSnapshot = flow(function* (this: SessionPanelViewmodel, messageId: string, terminalId: string) {
    if (!this.session) return

    try {
      const snapshot: TerminalSnapshot = yield this.session.getTerminalSnapshot(terminalId)
      const terminal = this.terminals.get(messageId) ?? this.terminals.get(terminalId)
      if (terminal instanceof TerminalMessageModel) terminal.updateFrom(snapshot)

      return snapshot.lines.length > 0
    } catch (e) {
      // basicToast.error({ title: 'Failed to get terminal snapshot' })
      console.error(e)
      throw e
    }
  })

  closeTerminal = flow(function* (this: SessionPanelViewmodel, messageId: string, terminalId: string) {
    if (!this.session) return
    try {
      yield this.session.closeTerminal(terminalId)
      this.terminals.delete(messageId)
    } catch (e) {
      basicToast.error({ title: 'Failed to close terminal' })
      console.error(e)
    }
  })

  private loadPlansFromSession = flow(async function* (this: SessionPanelViewmodel) {
    if (!this.session?.projectId) {
      this.setPlan(null)
      return
    }

    const plan = this.session.plan?.id ? yield PlanModel.get(this.session!.projectId!, this.session.plan.id) : null

    this.setPlan(plan)
  })

  private startListening = () => {
    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionLlm, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handleLlmEvent(event.payload.event)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionPrompt, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handlePromptEvent(event.payload.event)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionSignal, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handleSignalEvent(event.payload.event)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionControl, (event) => {
        if (event.payload.sessionId !== this.id) return
        this.handleControlEvent(event.payload.event)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.SessionEvent, (event) => {
        if (![event.payload.sessionId, event.payload.parentId].includes(this.id)) return
        if (event.payload.eventData.type !== 'toolCallCompleted') return
        const eventData = event.payload.eventData as ToolCallCompleted
        if (eventData.content.type !== 'success' || eventData.content.data.type !== 'plan_create') return

        this.handlePlanCreated(eventData.content.data as PlanCreatePayload)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.Internal, (event) => {
        if (event.payload.event.type !== 'plan_completed') return
        if (event.payload.event.sessionId !== this.id) return
        this.handlePlanCompleted(event.payload.event.planId)
      }),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.Internal, (event) => {
        if (event.payload.event.type !== 'plan_updated') return
        if (!this.session?.projectId) return
        if (event.payload.event.projectId !== this.session.projectId) return
        void this.reloadSessionPlans()
      }),
    )
  }

  private setBucketsFromMessages = () => {
    const entries = Array.from(this.messages.entries()).map(
      ([key, message]) => [key, message.type as MessageType['type']] as [string, MessageType['type']],
    )

    this.buckets.replace(bucketizeMessages(entries))
    if (this.visibleBucketSize > this.buckets.length) this.visibleBucketSize = this.buckets.length
  }

  private removeFromBuckets = (id: string) => {
    const bucketIndex = this.buckets.findIndex((bucket) => bucket.messageKeys.includes(id))
    if (bucketIndex === -1) return

    const bucket = this.buckets[bucketIndex]
    bucket.messageKeys = bucket.messageKeys.filter((key) => key !== id)

    if (bucket.messageKeys.length > 0) return

    this.buckets.splice(bucketIndex, 1)

    const prev = this.buckets[bucketIndex - 1]
    const next = this.buckets[bucketIndex]
    if (!prev || !next) return
    if (prev.type !== next.type) return

    prev.messageKeys.push(...next.messageKeys)
    this.buckets.splice(bucketIndex, 1)
  }

  private handlePromptEvent = (event: PromptEvent) => {
    switch (event.type) {
      case 'started':
        this.handlePromptStarted(event as PromptStarted)
        break
      case 'queued':
        this.handlePromptQueued(event as PromptQueued)
        break
      case 'deleted':
        this.handlePromptDeleted(event as PromptDeleted)
        break
    }
  }

  private handleSignalEvent = (event: SignalEvent) => {
    this.appendSignalMessage(event.message, event.type, event.error ?? null, event.id)
  }

  private handleControlEvent = (event: ControlEvent) => {
    if (!this.session) return

    switch (event.type) {
      case 'turnStop': {
        const lastMessage = Array.from(this.messages.values()).at(-1)
        const notificationMessage = lastMessage instanceof ResponseMessageModel ? lastMessage.content : 'Session Done'
        const title = this.session?.name ? `blprnt - ${this.session?.name}` : 'blprnt'
        void this.notifyWhenUnfocused(title, notificationMessage)
        break
      }
    }
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
        this.handleResponse(event as Response)
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
      case 'tokenUsage':
        this.handleTokenUsage(event as TokenUsage)
        break
    }
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
      const questionArgs = input as AskQuestionArgs
      if (this.messages.has(questionId)) return

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
        runInAction(() => {
          this.messages.delete(existing.id)
          this.addMessage(existing.id, existing)
        })

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

    if (payload.toolId === 'terminal') return

    const message: ToolUseMessage = {
      ...payload,
      createdAt: toDayJs(new Date().toISOString()),
      input,
      status: 'in_progress',
      type: 'tool_use',
    }

    this.addMessage(message.id, createMessageModel(message))
  }

  private handlePromptStarted = (payload: PromptStarted) => {
    const queueItemId = this.getPromptQueueItemId(payload)
    const pendingSubmission = this.consumePendingSubmissionForPromptId(queueItemId)
    const queuedMessage = this.messages.get(queueItemId)
    if (queuedMessage instanceof PromptMessageModel) {
      queuedMessage.content = payload.prompt || queuedMessage.content
      queuedMessage.updateFromStarted(payload)
      if (pendingSubmission && queuedMessage.imageUrls.length === 0) {
        queuedMessage.imageUrls = pendingSubmission.imageUrls
      }
      this.clearPendingPromptTracking(queueItemId)
      return
    }

    if (!payload.prompt) return

    this.clearPendingPromptTracking(queueItemId)

    const message: PromptMessage = {
      content: payload.prompt,
      createdAt: toDayJs(new Date().toISOString()),
      id: payload.id,
      imageUrls: pendingSubmission?.imageUrls ?? [],
      role: 'user',
      status: 'completed',
      turnId: payload.turnId ?? '',
      type: 'prompt',
    }

    const newMessage = createMessageModel(message)
    this.addMessage(newMessage.id, newMessage)
  }

  private handlePromptQueued = (payload: PromptQueued) => {
    const queueItemId = this.getPromptQueueItemId(payload)
    const promptFromPayload = this.getPromptFromQueuedPayload(payload)

    const existingQueuedPrompt = this.queuedPromptByQueueItemId.get(queueItemId)
    if (existingQueuedPrompt) {
      if (promptFromPayload && existingQueuedPrompt.prompt === 'Queued prompt') {
        existingQueuedPrompt.prompt = promptFromPayload
      }
      return
    }

    const pendingSubmission = this.pendingPromptQueue.shift()
    if (pendingSubmission) {
      const queuedPrompt = this.createQueuedPromptState(queueItemId, pendingSubmission)
      this.queuedPromptByQueueItemId.set(queueItemId, queuedPrompt)
      return
    }

    const sequence = this.reservePendingPromptSequence()
    const syntheticPendingSubmission: PendingPromptSubmission = {
      createdAt: new Date().toISOString(),
      imageUrls: [],
      localId: `queued-${queueItemId}`,
      prompt: promptFromPayload ?? 'Queued prompt',
      sequence,
    }

    const queuedPrompt = this.createQueuedPromptState(queueItemId, syntheticPendingSubmission)
    this.queuedPromptByQueueItemId.set(queueItemId, queuedPrompt)
  }

  private handlePromptDeleted = (payload: PromptDeleted) => {
    const queueItemId = this.getPromptQueueItemId(payload)
    this.clearPendingPromptTracking(queueItemId)
  }

  setQueuedPromptDeleting = (queueItemId: string, isDeleting: boolean) => {
    const normalizedQueueItemId = this.getPromptQueueItemId({ id: queueItemId, queue_item_id: queueItemId })
    const queuedPrompt = this.queuedPromptByQueueItemId.get(normalizedQueueItemId)
    if (!queuedPrompt) return
    queuedPrompt.isDeleting = isDeleting
  }

  private getPromptQueueItemId = (payload: { queue_item_id?: string | null; id?: string }) => {
    const queueItemId = payload.queue_item_id?.trim()
    if (queueItemId) return queueItemId
    return payload.id ?? ''
  }

  private getPromptFromQueuedPayload = (payload: PromptQueued) => {
    const promptValue = (payload as unknown as { prompt?: unknown }).prompt
    if (typeof promptValue !== 'string') return null

    const normalizedPrompt = promptValue.trim()
    if (!normalizedPrompt) return null
    return normalizedPrompt
  }

  private consumePendingSubmissionForPromptId = (promptId: string) => {
    const existingById = this.queuedPromptByQueueItemId.get(promptId)
    if (existingById) {
      this.queuedPromptByQueueItemId.delete(promptId)
      this.removePendingSubmissionFromQueue(existingById.localId)
      return existingById
    }

    const fromQueue = this.pendingPromptQueue.shift()
    if (!fromQueue) return undefined
    return fromQueue
  }

  private removePendingSubmissionFromQueue = (localId: string) => {
    const index = this.pendingPromptQueue.findIndex((submission) => submission.localId === localId)
    if (index === -1) return
    this.pendingPromptQueue.splice(index, 1)
  }

  private clearPendingPromptTracking = (promptId: string) => {
    const pendingSubmission = this.queuedPromptByQueueItemId.get(promptId)
    if (pendingSubmission) {
      this.removePendingSubmissionFromQueue(pendingSubmission.localId)
    }

    this.queuedPromptByQueueItemId.delete(promptId)
  }

  private createQueuedPromptState = (
    queueItemId: string,
    pendingSubmission: PendingPromptSubmission,
  ): QueuedPromptStateItem => {
    this.pendingPromptSequence = Math.max(this.pendingPromptSequence, pendingSubmission.sequence + 1)
    return {
      createdAt: pendingSubmission.createdAt,
      imageUrls: pendingSubmission.imageUrls,
      isDeleting: false,
      localId: pendingSubmission.localId,
      prompt: pendingSubmission.prompt,
      queueItemId,
      sequence: pendingSubmission.sequence,
    }
  }

  private reservePendingPromptSequence = () => {
    const sequence = this.pendingPromptSequence
    this.pendingPromptSequence += 1
    return sequence
  }

  private handleToolCallCompleted = (payload: ToolCallCompleted) => {
    if (payload.content.type === 'success' && payload.content.data.type === 'plan_create') return

    const toolResult = payload.content as ToolUseResponse

    if (toolResult.type === 'success' && toolResult.data.type === 'terminal') {
      const lastMessage = this.terminals.get(toolResult.data.terminal_id)

      if (lastMessage instanceof TerminalMessageModel) {
        lastMessage.lines = toolResult.data.snapshot?.lines ?? []
        lastMessage.cols = Number(toolResult.data.snapshot?.cols ?? 0)
        lastMessage.rows = Number(toolResult.data.snapshot?.rows ?? 0)
      } else {
        const message: TerminalMessage = {
          cols: Number(toolResult.data.snapshot?.cols ?? 0),
          createdAt: toDayJs(new Date().toISOString()),
          id: payload.id,
          lines: toolResult.data.snapshot?.lines ?? [],
          rows: Number(toolResult.data.snapshot?.rows ?? 0),
          terminalId: toolResult.data.terminal_id,
          type: 'terminal',
        }

        this.terminals.set(toolResult.data.terminal_id, createMessageModel(message) as TerminalMessageModel)
      }

      return
    }

    const canonicalId = this.subagentToolCallToMessageId.get(payload.id) ?? payload.id
    const message = this.messages.get(canonicalId) ?? this.messages.get(payload.itemId)

    if (!message) return

    if (message instanceof SubAgentMessageModel) {
      const previousStatus = message.status
      message.results = message.results ? [...message.results, toolResult] : [toolResult]
      const newStatus = toolResult.type === 'error' ? 'error' : 'completed'
      message.status = newStatus

      if (previousStatus === message.status) return
      const notificationMessage = newStatus === 'error' ? 'Subagent failed' : 'Subagent completed'

      const title = this.session?.name ? `blprnt - ${this.session?.name}` : 'blprnt'
      void this.notifyWhenUnfocused(title, notificationMessage)

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

  private handleTokenUsage = (event: TokenUsage) => {
    this._tokenUsage = event.inputTokens
  }

  private handlePlanCreated = async (planPayload: PlanCreatePayload) => {
    const existing = this.plan?.id === planPayload.id
    if (existing) return

    void notify('Plan created', `${planPayload.name}`)

    const plan = await PlanModel.get(this.session!.projectId!, planPayload.id)
    if (!plan) return

    this.setPlan(plan)
  }

  private handlePlanCompleted = (planId: string) => {
    const plan = this.plan?.id === planId ? this.plan : null
    if (!plan) return
    plan.setStatus('completed')
    this.setPlan(null)
  }

  private findSubagentMessage = (sessionId?: string | null) => {
    if (!sessionId) return undefined

    return Array.from(this.messages.values()).find(
      (message): message is SubAgentMessageModel =>
        message instanceof SubAgentMessageModel && message.subagentDetails.sessionId === sessionId,
    )
  }

  private isWindowFocused = () => {
    return document.hasFocus()
  }

  private notifyWhenUnfocused = async (title: string, body: string) => {
    if (this.isWindowFocused()) return

    await notify(title, body)
  }

  private truncateMessagesTo = (messageId: string) => {
    const rewindMessage = this.messages.get(messageId)
    const keepKeys = new Set<string>()
    let found = false

    for (const bucket of this.buckets) {
      for (const key of bucket.messageKeys) {
        if (key === messageId) {
          found = true
          break
        }
        keepKeys.add(key)
      }
      if (found) break
    }

    if (!found) return false

    Array.from(this.messages.keys()).forEach((key) => {
      if (!keepKeys.has(key)) this.messages.delete(key)
    })

    if (rewindMessage instanceof PromptMessageModel) this.prompt = rewindMessage.content

    return true
  }

  private appendSignalMessage = (
    content: string,
    signalType: SignalMessage['signalType'],
    error: SignalMessage['error'] = null,
    id: string | null = null,
  ) => {
    const message: SignalMessage = {
      content,
      createdAt: toDayJs(new Date().toISOString()),
      error,
      id: id ?? crypto.randomUUID(),
      role: 'system',
      signalType,
      type: 'signal',
    }

    if (this.messages.has(message.id)) return

    this.addMessage(message.id, createMessageModel(message))
  }

  private isSlashCommandMatch = (command: SlashCommand, query: string) => {
    if (command.name.toLowerCase().includes(query)) return true
    if (command.description.toLowerCase().includes(query)) return true
    if (!command.keywords?.length) return false
    return command.keywords.some((keyword) => keyword.toLowerCase().includes(query))
  }
}

export const SessionPanelViewmodelContext = createContext<SessionPanelViewmodel | null>(null)

export const useSessionPanelViewmodel = () => {
  const context = useContext(SessionPanelViewmodelContext)
  if (!context) throw new Error('SessionPanelViewmodelContext is not available')
  return context
}
