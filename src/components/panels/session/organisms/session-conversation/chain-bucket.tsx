import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
  ChainOfThoughtStep,
} from '@/components/ai-elements/chain-of-thought'
import { Response } from '@/components/ai-elements/response'
import { ThinkingIcon } from '@/components/ai-elements/thinking'
import { ErrorBoundaryBasic } from '@/components/molecules/error-boundary-basic'
import { ToolUseChainOfThoughtStep } from '@/components/panels/session/organisms/session-conversation/tool-use'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { ThinkingMessageModel } from '@/lib/models/messages/thinking-message.model'
import { ToolUseMessageModel } from '@/lib/models/messages/tool-use-message.model'
import type { ToolUseMessage } from '@/types'

export const ChainBucket = ({ messageKeys, isLast }: { messageKeys: string[]; isLast: boolean }) => {
  const viewmodel = useSessionPanelViewmodel()
  if (!messageKeys.length) return null

  const messages = messageKeys
    .map((messageKey) => viewmodel.getMessageByKey(messageKey))
    .filter((message) => message instanceof ThinkingMessageModel || message instanceof ToolUseMessageModel) as (
    | ThinkingMessageModel
    | ToolUseMessageModel
  )[]
  const filteredMessages = messages.filter(
    (message) =>
      ('content' in message && !!message.content?.trim()) ||
      (message instanceof ToolUseMessageModel && message.toolId !== 'subagent'),
  )

  if (!filteredMessages.length) return null

  return (
    <ChainOfThought isRounded className="border-l-4" defaultOpen={isLast}>
      <ChainOfThoughtHeader />
      <ChainOfThoughtContent className="pl-2">
        {filteredMessages.map((message) => {
          if (message instanceof ThinkingMessageModel) {
            return (
              <ChainOfThoughtStep key={message.id} icon={ThinkingIcon}>
                <div className="flex items-center gap-2 text-muted-foreground">Thinking</div>
                <div className="mt-2">
                  <Response>{message.content}</Response>
                </div>
              </ChainOfThoughtStep>
            )
          }
          if (message instanceof ToolUseMessageModel) {
            return (
              <ErrorBoundaryBasic
                key={message.id}
                fallback={<pre>{JSON.stringify(message, null, 2)}</pre>}
                onError={(error) => {
                  console.error(error)
                }}
              >
                <ToolUseChainOfThoughtStep
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
              </ErrorBoundaryBasic>
            )
          }
          return null
        })}
      </ChainOfThoughtContent>
    </ChainOfThought>
  )
}
