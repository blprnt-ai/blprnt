import { Search } from 'lucide-react'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import type { ToolUseMessage } from '@/types'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent?: boolean
}

export const DirTreeChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Getting directory tree'

  return <ChainOfThoughtStep key={message.id} className={textSizeClassName} icon={Search} label={label} />
}

export const DirSearchChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  if (!message.input) return null

  const label = 'Searching directory'

  return <ChainOfThoughtStep key={message.id} className={textSizeClassName} icon={Search} label={label} />
}
