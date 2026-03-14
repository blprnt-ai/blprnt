import type { Dayjs } from 'dayjs'
import type {
  ErrorEvent,
  Provider,
  ReasoningEffort,
  SubAgentArgs,
  SubagentDetails,
  ToolId,
  ToolUseResponse,
} from './bindings'

export type Simplify<T> = { [K in keyof T]: T[K] } & {}

export type SignalType = 'info' | 'warning' | 'error'
export type MessageRole = 'user' | 'assistant' | 'system'
export type MessageStatus = 'pending' | 'in_progress' | 'completed' | 'error'

export interface PromptMessage {
  type: 'prompt'
  id: string
  turnId: string
  content: string
  status: MessageStatus
  role: MessageRole
  tokenUsage?: number
  createdAt: Dayjs
  imageUrls?: string[]
}

export interface ResponseMessage {
  type: 'response'
  id: string
  turnId: string
  stepId: string
  tokenUsage?: number
  role: MessageRole
  status: MessageStatus
  content: string
  createdAt: Dayjs
}

export interface Image64Message {
  type: 'image64'
  content: string
  tokenUsage?: number
}

export interface ThinkingMessage {
  type: 'thinking'
  id: string
  turnId: string
  stepId: string
  tokenUsage?: number
  role: MessageRole
  status: MessageStatus
  content: string
  createdAt: Dayjs
}

export interface TerminalMessage {
  type: 'terminal'
  id: string
  terminalId: string
  rows: number
  cols: number
  lines: string[]
  createdAt: Dayjs
}

export interface ToolUseMessage {
  type: 'tool_use'
  id: string
  turnId: string
  stepId: string
  toolId: ToolId
  tokenUsage?: number
  input: unknown
  result?: ToolUseResponse
  status: MessageStatus
  createdAt: Dayjs
}

export interface SubAgentMessage {
  type: 'subagent'
  id: string
  turnId: string
  stepId: string
  status: MessageStatus
  tokenUsage?: number
  results?: ToolUseResponse[]
  input: SubAgentArgs
  subagentDetails: SubagentDetails
  createdAt: Dayjs
}

export interface CompactSummaryMessage {
  id: string
  type: 'compact_summary'
  content: string
  createdAt: Dayjs
}

export interface SignalMessage {
  type: 'signal'
  id: string
  content: string
  error: ErrorEvent | null
  signalType: SignalType
  role: MessageRole
  createdAt: Dayjs
  deleteId?: string
}

export interface QuestionAnswerMessage {
  type: 'question_answer'
  id: string
  question: string
  options: string[]
  details: string
  answer?: string
  tokenUsage?: number
  createdAt: Dayjs
}

export interface WebSearchMessage {
  type: 'web_search'
  id: string
  url: string
  title: string
  status: MessageStatus
  startIndex: bigint
  endIndex: bigint
  createdAt: Dayjs
  tokenUsage?: number
}

export type MessageType =
  | PromptMessage
  | Image64Message
  | ResponseMessage
  | ThinkingMessage
  | TerminalMessage
  | ToolUseMessage
  | SubAgentMessage
  | CompactSummaryMessage
  | SignalMessage
  | QuestionAnswerMessage
  | WebSearchMessage

export enum RouteType {
  Root = 'root',
  Onboarding = 'onboarding',
  Dashboard = 'dashboard',
  Sessions = 'sessions',
  Project = 'project',
  ProjectNew = 'project-new',
  ProjectEdit = 'project-edit',
  Settings = 'settings',
}

export enum ReasoningEffortEnum {
  High = 'high',
  Medium = 'medium',
  Low = 'low',
  Minimal = 'minimal',
  None = 'none',
}

export const getReasoningEffortLabel = (reasoningEffort: ReasoningEffort) => {
  switch (reasoningEffort) {
    case ReasoningEffortEnum.High:
      return 'High'
    case ReasoningEffortEnum.Medium:
      return 'Medium'
    case ReasoningEffortEnum.Low:
      return 'Low'
    case ReasoningEffortEnum.Minimal:
      return 'Minimal'
    case ReasoningEffortEnum.None:
      return 'None'
  }
}

export type AllProviders = Provider | 'anthropic_fnf' | 'openai_fnf'
