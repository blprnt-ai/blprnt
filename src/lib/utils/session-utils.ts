import type {
  AskQuestionArgs,
  JsonValue,
  MessageRecord,
  PlanCreateArgs,
  PlanTodoItem,
  SubAgentArgs,
  ToolId,
  ToolUseResponse,
} from '@/bindings'
import { tauriProjectApi } from '@/lib/api/tauri/project.api'
import type {
  CompactSummaryMessage,
  Image64Message,
  PromptMessage,
  QuestionAnswerMessage,
  ResponseMessage,
  SignalMessage,
  SubAgentMessage,
  TerminalMessage,
  ThinkingMessage,
  ToolUseMessage,
} from '@/types'
import { toDayJs } from './misc'

export const toCompactSummaryMessage = (model: MessageRecord): CompactSummaryMessage | undefined => {
  if (model.content.type !== 'text') return

  return {
    content: model.content.text,
    createdAt: toDayJs(model.created_at),
    id: model.id,
    type: 'compact_summary',
  } as CompactSummaryMessage
}

export const toPromptMessage = (model: MessageRecord): PromptMessage | undefined => {
  if (model.content.type !== 'text') return

  return {
    content: model.content.text,
    createdAt: toDayJs(model.created_at),
    id: model.id,
    imageUrls: [],
    role: 'user',
    status: 'completed',
    tokenUsage: model.token_usage ?? 0,
    turnId: model.turn_id,
    type: 'prompt',
  }
}

export const toResponseMessage = (model: MessageRecord): ResponseMessage | undefined => {
  if (model.content.type !== 'text') return

  return {
    content: sanitizeContent(model.content.text),
    createdAt: toDayJs(model.created_at),
    id: model.id,
    role: 'assistant',
    status: 'completed',
    stepId: model.step_id,
    tokenUsage: model.token_usage ?? 0,
    turnId: model.turn_id,
    type: 'response',
  }
}

export const toImage64Message = (model: MessageRecord): Image64Message | undefined => {
  if (model.content.type !== 'image64') return

  return {
    content: model.content.image_64,
    tokenUsage: model.token_usage ?? 0,
    type: 'image64',
  }
}

export const toThinkingMessage = (model: MessageRecord): ThinkingMessage | undefined => {
  if (model.content.type !== 'thinking') return

  return {
    content: model.content.thinking,
    createdAt: toDayJs(model.created_at),
    id: model.id,
    role: 'assistant',
    status: 'completed',
    stepId: model.step_id,
    tokenUsage: model.token_usage ?? 0,
    turnId: model.turn_id,
    type: 'thinking',
  }
}

const planToolIds = new Set<ToolId>(['plan_create', 'plan_list', 'plan_get', 'plan_update', 'plan_delete'])

export const toToolUseMessage = (
  model: MessageRecord,
): ToolUseMessage | SubAgentMessage | QuestionAnswerMessage | TerminalMessage | undefined => {
  if (model.content.type !== 'tool_use' || planToolIds.has(model.content.tool_id)) return

  const parsed = parseInput(model.content.input)

  if (model.content.tool_id === 'subagent') {
    const input = parsed as SubAgentArgs
    if (!model.content.subagent_details) {
      return {
        createdAt: toDayJs(model.created_at),
        id: model.content.id,
        input,
        status: 'completed',
        stepId: model.step_id,
        tokenUsage: model.token_usage ?? 0,
        toolId: model.content.tool_id,
        turnId: model.turn_id,
        type: 'tool_use',
      }
    }

    return {
      createdAt: toDayJs(model.created_at),
      id: model.content.id,
      input,
      status: 'in_progress',
      stepId: model.step_id,
      subagentDetails: model.content.subagent_details,
      tokenUsage: model.token_usage ?? 0,
      turnId: model.turn_id,
      type: 'subagent',
    }
  } else if (model.content.tool_id === 'ask_question') {
    const input = parsed as AskQuestionArgs

    return {
      answer: '',
      createdAt: toDayJs(model.created_at),
      details: input.details,
      id: model.content.id,
      options: input.options,
      question: input.question,
      tokenUsage: model.token_usage ?? 0,
      type: 'question_answer',
    }
  } else if (model.content.tool_id === 'terminal') {
    return {
      cols: 120,
      createdAt: toDayJs(model.created_at),
      id: model.content.id,
      lines: [],
      rows: 10,
      terminalId: '',
      type: 'terminal',
    }
  } else {
    return {
      createdAt: toDayJs(model.created_at),
      id: model.content.id,
      input: parsed,
      status: 'completed',
      stepId: model.step_id,
      tokenUsage: model.token_usage ?? 0,
      toolId: model.content.tool_id,
      turnId: model.turn_id,
      type: 'tool_use',
    }
  }
}

export interface PlanSummaryFromHistory {
  planId: string
  name: string
  description: string
  content?: string
  todos?: PlanTodoItem[] | null
  inProgress: boolean
}

export const getPlansFromHistory = async (
  projectId: string,
  models: MessageRecord[],
): Promise<PlanSummaryFromHistory[]> => {
  const toolResults = models.reduce(
    (acc, model) => {
      if (model.content.type !== 'tool_result') return acc
      acc[model.content.tool_use_id] = model.content.content as ToolUseResponse
      return acc
    },
    {} as Record<string, ToolUseResponse>,
  )

  const filteredModels = models.filter(
    (model) => model.content.type === 'tool_use' && model.content.tool_id === 'plan_create',
  )

  const plans = await Promise.all(
    filteredModels.map(async (model) => {
      if (model.content.type !== 'tool_use') return undefined
      const toolUseId = model.content.id
      const toolResult = toolResults[toolUseId]
      if (!toolResult || toolResult.type !== 'success' || toolResult.data.type !== 'plan_create') return undefined
      const input = parseInput(model.content.input) as PlanCreateArgs

      try {
        await tauriProjectApi.planGet(projectId, toolResult.data.id)
        return {
          content: input.content,
          description: input.description,
          name: input.name,
          planId: toolResult.data.id,
          todos: input.todos ?? null,
        }
      } catch (error) {
        console.error(error)
        return undefined
      }
    }),
  )

  return plans.filter(Boolean) as PlanSummaryFromHistory[]
}

export const parseInput = (input: JsonValue): unknown => {
  if (typeof input !== 'string') return input

  try {
    const parsed = JSON.parse(input)
    if (typeof parsed === 'string') return parseInput(parsed)

    return parsed
  } catch {
    return input
  }
}

export const toSignalMessage = (model: MessageRecord): SignalMessage | undefined => {
  if (model.content.type !== 'info' && model.content.type !== 'warning' && model.content.type !== 'error') return

  return {
    content: model.content.message,
    createdAt: toDayJs(model.created_at),
    error: model.content.error ?? null,
    id: model.id,
    role: 'system',
    signalType: model.content.type,
    type: 'signal',
  }
}

export const sanitizeContent = (content: string) => {
  const lines = content.split('\n')

  let insideCodeBlock = false

  return lines
    .map((line) => {
      if (line.startsWith('```')) insideCodeBlock = !insideCodeBlock
      if (insideCodeBlock) return line
      if (line.includes('`')) return line

      return line.replace(/<([^>]*)>?/g, '\u003C$1\u003E')
    })
    .join('\n')
}
