import { Cpu } from 'lucide-react'
import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import { upperFirst } from '@/lib/utils/string'
import type { ToolUseMessage } from '@/types'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent: boolean
}

export const McpToolUseChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const toolId = message.toolId as { unknown: string } | { mcp: string }
  const toolProp = 'unknown' in toolId ? toolId.unknown : toolId.mcp
  const [, serverId, toolName] = toolProp.split('__')

  const userServerName = serverId
    .split('_')
    .map((word) => upperFirst(word))
    .join(' ')
  const userToolName = toolName
    .split('-')
    .map((word) => upperFirst(word))
    .join(' ')
    .split('_')
    .map((word) => upperFirst(word))
    .join(' ')

  return (
    <ChainOfThoughtStep key={message.id} icon={Cpu} isSubagent={isSubagent}>
      <div className="flex items-center gap-2 text-muted-foreground">
        {userServerName}: {userToolName}
      </div>
    </ChainOfThoughtStep>
  )
}
