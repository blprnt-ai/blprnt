import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
  ChainOfThoughtStep,
} from '@/components/ai-elements/chain-of-thought'
import { Response } from '@/components/ai-elements/response'
import { ThinkingIcon } from '@/components/ai-elements/thinking'
import { ToolUseChainOfThoughtStep } from '@/components/panels/session/organisms/session-conversation/tool-use'
import type { MessageModel } from '@/lib/models/messages/message-factory'
import { ThinkingMessageModel } from '@/lib/models/messages/thinking-message.model'
import { ToolUseMessageModel } from '@/lib/models/messages/tool-use-message.model'
import type { ToolUseMessage } from '@/types'

export const ChainBucket = ({
  messageKeys,
  isLast,
  messages,
}: {
  messageKeys: string[]
  isLast: boolean
  messages: Map<string, MessageModel>
}) => {
  if (!messageKeys.length) return null

  return (
    <ChainOfThought isRounded className="text-xs" defaultOpen={isLast}>
      <ChainOfThoughtHeader className="text-xs" />
      <ChainOfThoughtContent isSubagent>
        {messageKeys.map((messageKey) => {
          const message = messages.get(messageKey)
          if (!message) return null
          if (message instanceof ThinkingMessageModel && !!message.content.trim()) {
            return (
              <ChainOfThoughtStep key={messageKey} isSubagent className="text-xs" icon={ThinkingIcon}>
                <div className="flex items-center gap-2 text-muted-foreground">Thinking</div>
                <div className="mt-2">
                  <Response>{message.content}</Response>
                </div>
              </ChainOfThoughtStep>
            )
          }
          if (message instanceof ToolUseMessageModel) {
            return (
              <ToolUseChainOfThoughtStep
                key={messageKey}
                isSubagent
                message={
                  {
                    createdAt: message.createdAt,
                    id: message.id,
                    input: message.input,
                    result: message.result,
                    status: message.status,
                    stepId: message.stepId,
                    toolId: message.toolId,
                    turnId: message.turnId,
                    type: 'tool_use',
                  } as ToolUseMessage
                }
              />
            )
          }
          return null
        })}
      </ChainOfThoughtContent>
    </ChainOfThought>
  )
}
