import { Terminal } from 'lucide-react'
import type { ShellArgs, ShellPayload, ToolUseResponse } from '@/bindings'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import type { ToolUseMessage } from '@/types'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent?: boolean
}

export const ShellChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  const descriptionClassName = isSubagent ? 'text-xs!' : undefined
  if (!message.input) return null

  const label = 'Executing shell command'
  const args = message.input as ShellArgs
  const result = getShellError(message.result)

  const description = (
    <div className="flex flex-col gap-1">
      <pre>
        {args.command} {args.args?.join(' ') || ''}
      </pre>
      {result && <span className="text-destructive/60">{result}</span>}
    </div>
  )

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      description={description}
      descriptionClassName={descriptionClassName}
      icon={Terminal}
      isSubagent={isSubagent}
      label={label}
    />
  )
}

const getShellError = (result: ToolUseResponse | undefined) => {
  if (!result) return null

  if (result.type === 'error') return result.error
  const data = result.data as ShellPayload
  if (data.exit_code !== 0) return `Command failed with exit code ${data.exit_code}`

  return null
}
