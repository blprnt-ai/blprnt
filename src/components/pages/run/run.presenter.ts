import type { JsonValue } from '@/bindings/serde_json/JsonValue'
import type { ToolUseResponse } from '@/bindings/ToolUseResponse'
import type { TurnStepContent } from '@/bindings/TurnStepContent'
import type { ToolId } from '@/bindings/ToolId'
import type { RunModel } from '@/models/run.model'
import type { TurnModel } from '@/models/turn.model'

export const formatToolId = (toolId: ToolId) => {
  if (typeof toolId === 'string') return toolId
  if ('mcp' in toolId) return `mcp:${toolId.mcp}`
  if ('unknown' in toolId) return toolId.unknown
  return 'unknown'
}

export const formatAbsoluteRunTime = (date: Date | null) => {
  if (!date || Number.isNaN(date.getTime())) return 'Not available'

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

export const formatStepStatus = (status: string) => {
  return status.replace('_', ' ')
}

export const getRunStats = (run: RunModel) => {
  const steps = run.turns.flatMap((turn) => turn.steps)

  return {
    turnCount: run.turns.length,
    stepCount: steps.length,
    completedStepCount: steps.filter((step) => step.status === 'completed').length,
    toolCallCount: steps.reduce((count, step) => count + getToolUses(step.response.contents).length, 0),
  }
}

export const getTurnSummary = (turn: TurnModel, index: number) => {
  return {
    label: `Turn ${index + 1}`,
    createdAtLabel: formatAbsoluteRunTime(turn.createdAt),
    stepCount: turn.steps.length,
  }
}

export const getTextContents = (contents: TurnStepContent[]) => {
  return contents.flatMap((content) => ('Text' in content && content.Text.text.trim() ? [content.Text.text] : []))
}

export const getThinkingContents = (contents: TurnStepContent[]) => {
  return contents.flatMap((content) =>
    'Thinking' in content && content.Thinking.thinking.trim() ? [content.Thinking.thinking] : [],
  )
}

export const getToolUses = (contents: TurnStepContent[]) => {
  return contents.flatMap((content) => ('ToolUse' in content ? [content.ToolUse] : []))
}

export const getToolResults = (contents: TurnStepContent[]) => {
  return contents.flatMap((content) => ('ToolResult' in content ? [content.ToolResult] : []))
}

export const stringifyJson = (value: unknown) => JSON.stringify(value, null, 2)

export const summarizePrompt = (turn: TurnModel) => {
  const firstStep = turn.steps[0]
  const firstPrompt = firstStep ? getTextContents(firstStep.request.contents)[0] : null

  return firstPrompt?.trim() || 'Prompt unavailable.'
}

export const getToolResultLookup = (turn: TurnModel) => {
  const results = new Map<string, ReturnType<typeof getToolResults>[number][]>()

  for (const step of turn.steps) {
    for (const result of getToolResults(step.request.contents)) {
      const existing = results.get(result.tool_use_id) ?? []
      existing.push(result)
      results.set(result.tool_use_id, existing)
    }
  }

  return results
}

export const summarizeToolInput = (input: JsonValue) => {
  if (!input || typeof input !== 'object' || Array.isArray(input)) return 'No structured input'

  const entries = Object.entries(input).slice(0, 3)
  if (entries.length === 0) return 'No structured input'

  return entries
    .map(([key, value]) => `${humanizeKey(key)}: ${summarizeScalar(value)}`)
    .join(' • ')
}

export const summarizeToolResult = (result: ToolUseResponse) => {
  if (result.type === 'error') return result.error

  const data = result.data

  switch (data.type) {
    case 'files_read':
      return `${data.files.length} file${data.files.length === 1 ? '' : 's'} read`
    case 'apply_patch':
      return `${data.paths.length} file${data.paths.length === 1 ? '' : 's'} updated`
    case 'shell':
      return data.exit_code === 0 ? 'Command completed successfully' : `Command failed with exit code ${data.exit_code}`
    case 'terminal':
      return data.snapshot ? `${data.snapshot.lines.length} terminal line${data.snapshot.lines.length === 1 ? '' : 's'} captured` : 'Terminal session updated'
    case 'mcp_tool':
      return `${data.server_id} · ${data.name}`
    case 'unknown':
      return data.error
    default:
      return 'Tool completed'
  }
}

const summarizeScalar = (value: JsonValue) => {
  if (typeof value === 'string') return value.length > 36 ? `${value.slice(0, 33)}...` : value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (value === null) return 'null'
  if (Array.isArray(value)) return `${value.length} item${value.length === 1 ? '' : 's'}`
  return `${Object.keys(value).length} field${Object.keys(value).length === 1 ? '' : 's'}`
}

const humanizeKey = (value: string) => value.replace(/_/g, ' ')
