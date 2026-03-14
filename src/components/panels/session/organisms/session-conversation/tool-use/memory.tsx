import { Save, Search } from 'lucide-react'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import type { ToolUseMessage } from '@/types'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent?: boolean
}

export const MemoryWriteChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Writing memory'

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      icon={Save}
      isSubagent={isSubagent}
      label={label}
    />
  )
}

export const MemorySearchChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Searching memories...'

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      icon={Search}
      isSubagent={isSubagent}
      label={label}
    />
  )
}
