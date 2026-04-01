import type { ReasoningEffort } from '@/bindings/ReasoningEffort'

export const DEFAULT_REASONING_OPTION = '__default__'

export const reasoningEffortOptions: { label: string; value: ReasoningEffort }[] = [
  { label: 'Minimal', value: 'minimal' },
  { label: 'Low', value: 'low' },
  { label: 'Medium', value: 'medium' },
  { label: 'High', value: 'high' },
  { label: 'Max', value: 'xhigh' },
  { label: 'None', value: 'none' },
]

export const formatReasoningEffort = (value: ReasoningEffort | null | undefined) => {
  if (!value) return 'Default'

  return reasoningEffortOptions.find((option) => option.value === value)?.label ?? value
}

export const formatDefaultReasoningLabel = (value: ReasoningEffort | null | undefined) => {
  if (!value) return 'Default'

  return `Default (${formatReasoningEffort(value)})`
}
