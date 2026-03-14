import { useMemo } from 'react'
import { Response } from '@/components/ai-elements/response'
import { ResponseCodeBlock } from '@/components/ai-elements/response-code-block'
import { ExternalLink } from '@/components/molecules/external-link'
import { RelativeTime } from '@/components/molecules/relative-time'
import { MessageType } from '@/lib/models/messages/base-message.model'
import type { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import type { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SignalMessageModel } from '@/lib/models/messages/signal-message.model'
import { cn } from '@/lib/utils/cn'
import { useSubagentConversationViewmodel } from './subagent-conversation-viewmodel'

export const ConversationMessageCard = ({
  message,
}: {
  message: PromptMessageModel | ResponseMessageModel | SignalMessageModel
}) => {
  const subagentViewmodel = useSubagentConversationViewmodel()
  const headerLabel = useMemo(() => {
    if (message.type === MessageType.Response) return 'assistant'
    if (message.type === MessageType.Signal && message instanceof SignalMessageModel) return message.signalType
    return 'Primary Agent'
  }, [message])

  const isAssistant = message.type === MessageType.Response
  const isSignal = message.type === MessageType.Signal

  const tokenUsage = isAssistant ? subagentViewmodel.getTokenUsageFromMessage(message.id) : 0

  return (
    <div className="rounded-md border border-border bg-accent/40 p-2 flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <span
          className={cn(
            'text-xs font-medium text-muted-foreground',
            isAssistant && 'text-primary',
            isSignal && 'text-warn',
          )}
        >
          {headerLabel}
        </span>

        <RelativeTime timestamp={message.createdAt} />
      </div>
      <div className="text-xs">
        {message.type === MessageType.Response ? (
          <Response
            components={{
              a: ({ href, children }) => <ExternalLink href={href ?? ''}>{children}</ExternalLink>,
              code: (props) => <ResponseCodeBlock {...props} className={cn(props.className, 'code-compact')} />,
            }}
          >
            {message.content}
          </Response>
        ) : (
          <div className="whitespace-pre-wrap">{message.content}</div>
        )}
      </div>
      {isAssistant && tokenUsage > 0 && (
        <span className="text-xs text-muted-foreground/60">{tokenUsage.toLocaleString()} tokens used</span>
      )}
    </div>
  )
}
