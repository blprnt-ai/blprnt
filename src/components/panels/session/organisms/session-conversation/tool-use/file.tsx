import { FileDiff, FileText } from 'lucide-react'

import { ChainOfThoughtStep } from '@/components/ai-elements/chain-of-thought'
import { ErrorBoundaryBasic } from '@/components/molecules/error-boundary-basic'
import type { ToolUseMessage } from '@/types'
import { createDescriptionWithError } from './utils'

interface ToolChainStepProps {
  message: ToolUseMessage
  isSubagent: boolean
}

interface ApplyPatchArgs {
  diff: string
}

export const FilesReadChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  const descriptionClassName = isSubagent ? 'text-xs!' : undefined
  if (!message.input) return null

  const label = 'Reading files'
  const args = message.input as {
    items: { path: string; line_start?: number | null; line_end?: number | null }[]
  }
  const description = (
    <>
      {args.items.map((item) => (
        <div key={`${item.path}-${item.line_start}-${item.line_end}`}>{item.path}</div>
      ))}
    </>
  )

  const descriptionWithError = createDescriptionWithError(description, message.result)

  return (
    <ChainOfThoughtStep
      key={message.id}
      className={textSizeClassName}
      description={descriptionWithError}
      descriptionClassName={descriptionClassName}
      icon={FileText}
      isSubagent={isSubagent}
      label={label}
    />
  )
}

export const ApplyPatchChainOfThoughtStep = ({ message, isSubagent }: ToolChainStepProps) => {
  const textSizeClassName = isSubagent ? 'text-xs' : undefined
  const descriptionClassName = isSubagent ? 'text-xs!' : undefined
  if (!message.input) return null

  const label = 'Applying patch'
  const args = message.input as ApplyPatchArgs
  const paths = args.diff ? extractPatchPaths(args.diff) : []
  const description =
    paths.length > 0 ? (
      <div>
        {paths.map((path) => (
          <div key={path}>{path}</div>
        ))}
      </div>
    ) : (
      'Patch update'
    )

  const descriptionWithError = createDescriptionWithError(description, message.result)

  return (
    <ErrorBoundaryBasic
      key={message.id}
      fallback={<pre>DILDO: {JSON.stringify(message, null, 2)}</pre>}
      onError={(error) => {
        console.error(error)
      }}
    >
      <ChainOfThoughtStep
        key={message.id}
        className={textSizeClassName}
        description={descriptionWithError}
        descriptionClassName={descriptionClassName}
        icon={FileDiff}
        isSubagent={isSubagent}
        label={label}
      />
    </ErrorBoundaryBasic>
  )
}

const extractPatchPaths = (diff: string) => {
  const paths = new Set<string>()

  diff
    .split('\n')
    .map((line) => line.trim())
    .forEach((line) => {
      if (line.startsWith('*** Add File: ')) paths.add(line.replace('*** Add File: ', ''))
      if (line.startsWith('*** Update File: ')) paths.add(line.replace('*** Update File: ', ''))
      if (line.startsWith('*** Delete File: ')) paths.add(line.replace('*** Delete File: ', ''))
    })

  return Array.from(paths)
}
