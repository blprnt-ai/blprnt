import type { ToolUseMessage } from '@/types'
import { ApplyPatchChainOfThoughtStep, FilesReadChainOfThoughtStep } from './file'
import { McpToolUseChainOfThoughtStep } from './mcp'
import { MemorySearchChainOfThoughtStep, MemoryWriteChainOfThoughtStep } from './memory'
import { GetPrimerChainOfThoughtStep, UpdatePrimerChainOfThoughtStep } from './primer'
import { RgSearchChainOfThoughtStep } from './rg'
import { ShellChainOfThoughtStep } from './shell'

export const ToolUseChainOfThoughtStep = ({
  message,
  isSubagent = false,
}: {
  message: ToolUseMessage
  isSubagent?: boolean
}) => {
  const stepProps = { isSubagent, message }

  if (
    typeof message.toolId === 'object' &&
    (('mcp' in message.toolId && message.toolId.mcp.startsWith('mcp__')) ||
      ('unknown' in message.toolId && message.toolId.unknown.startsWith('mcp__')))
  ) {
    return <McpToolUseChainOfThoughtStep {...stepProps} />
  }

  switch (message.toolId) {
    case 'files_read':
      return <FilesReadChainOfThoughtStep {...stepProps} />
    case 'apply_patch':
      return <ApplyPatchChainOfThoughtStep {...stepProps} />
    case 'shell':
      return <ShellChainOfThoughtStep {...stepProps} />
    case 'primer_get':
      return <GetPrimerChainOfThoughtStep {...stepProps} />
    case 'primer_update':
      return <UpdatePrimerChainOfThoughtStep {...stepProps} />
    case 'memory_write':
      return <MemoryWriteChainOfThoughtStep {...stepProps} />
    case 'memory_search':
      return <MemorySearchChainOfThoughtStep {...stepProps} />
    case 'rg':
      return <RgSearchChainOfThoughtStep {...stepProps} />
    default:
      return null
  }
}
