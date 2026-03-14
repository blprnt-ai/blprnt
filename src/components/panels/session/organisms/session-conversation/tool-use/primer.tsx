import { HatGlasses } from 'lucide-react'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import type { ToolUseMessage } from '@/types'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent?: boolean
}

export const GetPrimerChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Get Primer'

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      icon={HatGlasses}
      isSubagent={isSubagent}
      label={label}
    />
  )
}

export const UpdatePrimerChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Update Primer'

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      icon={HatGlasses}
      isSubagent={isSubagent}
      label={label}
    />
  )
}
