import { Search } from 'lucide-react'
import type { RgSearchArgs } from '@/bindings'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import type { ToolUseMessage } from '@/types'
import { createDescriptionWithError } from './utils'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent?: boolean
}

export const RgSearchChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  const descriptionClassName = isSubagent ? 'text-xs!' : undefined
  if (!message.input) return null

  const args = message.input as RgSearchArgs
  const target = args.path ?? ''
  const directory = target === '.' ? ' ' : `${target} `
  const flags = args.flags?.length ? args.flags.join(' ') : ''
  const description = `Searching ${directory}for “${args.pattern}” ${flags}`
  const descriptionWithError = createDescriptionWithError(description, message.result)

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      description={descriptionWithError}
      descriptionClassName={descriptionClassName}
      icon={Search}
      isSubagent={isSubagent}
    />
  )
}
