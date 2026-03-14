import { useMemo, useRef, useState } from 'react'
import { Response } from '@/components/ai-elements/response'
import { ResponseCodeBlock } from '@/components/ai-elements/response-code-block'
import { ExternalLink } from '@/components/molecules/external-link'
import { RelativeTime } from '@/components/molecules/relative-time'
import { ErrorMessage, InfoMessage, WarningMessage } from '@/components/panels/session/molecules/error-message'
import { FullImageModal } from '@/components/panels/session/molecules/full-image-modal'
import { MessageActionButtons } from '@/components/panels/session/molecules/message-action-buttons'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { MessageType } from '@/lib/models/messages/base-message.model'
import { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SignalMessageModel } from '@/lib/models/messages/signal-message.model'
import { cn } from '@/lib/utils/cn'

export const ConversationMessageCard = ({
  message,
}: {
  message: PromptMessageModel | ResponseMessageModel | SignalMessageModel
}) => {
  const sessionViewmodel = useSessionPanelViewmodel()
  const expaderRef = useRef<HTMLDivElement>(null)
  const [isExpanded, setIsExpanded] = useState(false)

  const headerLabel = useMemo(() => {
    if (message instanceof ResponseMessageModel) return 'blprnt'
    if (message instanceof SignalMessageModel) return message.signalType
    return message.role
  }, [message])

  const isUser = message instanceof PromptMessageModel && message.role === 'user'
  const isAssistant = message instanceof ResponseMessageModel
  const isSignal = message instanceof SignalMessageModel
  const isQueuedPrompt = message instanceof PromptMessageModel && message.status === 'pending'
  const tokenUsage = isAssistant ? sessionViewmodel.getTokenUsageFromMessage(message.id) : 0

  return (
    <div
      className={cn(
        'rounded-md border border-border bg-accent p-3 group flex flex-col gap-2',
        // (isAssistant || isSignal) && 'mr-8',
        // isUser && 'ml-8',
        (isAssistant || isSignal) && 'border-l-4',
        isUser && 'border-l-4 border-l-primary',
      )}
    >
      {!isSignal && (
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className={cn('text-sm font-medium text-muted-foreground', isAssistant && 'text-primary')}>
              {headerLabel}
            </span>
            {isQueuedPrompt && (
              <span className="rounded-full border border-amber-500/40 bg-amber-500/15 px-2 py-0.5 text-[11px] font-medium text-amber-700 dark:text-amber-300">
                Queued
              </span>
            )}
          </div>

          <RelativeTime timestamp={message.createdAt} />
        </div>
      )}
      <div className={cn('text-sm', !isExpanded && 'max-h-[300px] overflow-y-auto')}>
        <div ref={expaderRef}>
          <ConversationMessageCardContent message={message} />
        </div>
      </div>
      {!isSignal && (
        <div className="flex items-center justify-between">
          {isAssistant && tokenUsage > 0 ? (
            <span className="text-xs text-muted-foreground/60">{tokenUsage.toLocaleString()} tokens used</span>
          ) : (
            <div />
          )}

          <MessageActionButtons isExpanded={isExpanded} message={message} onExpand={() => setIsExpanded(!isExpanded)} />
        </div>
      )}
    </div>
  )
}

interface ConversationMessageCardContentProps {
  message: PromptMessageModel | ResponseMessageModel | SignalMessageModel
}

export const ConversationMessageCardContent = ({ message }: ConversationMessageCardContentProps) => {
  switch (message.type) {
    case MessageType.Response:
      return <ConversationMessageCardContentResponse message={message as ResponseMessageModel} />
    case MessageType.Prompt:
      return <ConversationMessageCardContentPrompt message={message as PromptMessageModel} />
    case MessageType.Signal:
      return <ConversationMessageCardContentSignal message={message as SignalMessageModel} />
  }
}

interface ConversationMessageCardContentResponseProps {
  message: ResponseMessageModel
}

export const ConversationMessageCardContentResponse = ({ message }: ConversationMessageCardContentResponseProps) => {
  return (
    <Response
      components={{
        a: ({ href, children }) => <ExternalLink href={href ?? ''}>{children}</ExternalLink>,
        code: ResponseCodeBlock,
      }}
    >
      {message.content}
    </Response>
  )
}

interface ConversationMessageCardContentPromptProps {
  message: PromptMessageModel
}

export const ConversationMessageCardContentPrompt = ({ message }: ConversationMessageCardContentPromptProps) => {
  const [fullImageUrl, setFullImageUrl] = useState<string | null>(null)

  return (
    <>
      <div className="flex flex-col gap-2">
        {message instanceof PromptMessageModel && message.imageUrls.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-2 pb-2 border-b border-dashed">
            {message.imageUrls.map((url) => (
              <div key={url} className="relative">
                <img
                  alt="Image"
                  className="size-20 rounded-md border object-cover cursor-pointer"
                  src={url}
                  onClick={() => setFullImageUrl(url)}
                />
              </div>
            ))}
          </div>
        )}
        <div className="whitespace-pre-wrap">{message.content}</div>
      </div>
      {fullImageUrl && <FullImageModal imageUrl={fullImageUrl} onClose={() => setFullImageUrl(null)} />}
    </>
  )
}

export const ConversationMessageCardContentSignal = ({ message }: { message: SignalMessageModel }) => {
  switch (message.signalType) {
    case 'error':
      return <ErrorMessage createdAt={message.createdAt} error={message.error!} />
    case 'info':
      return <InfoMessage createdAt={message.createdAt} message={message.content} />
    case 'warning':
      return <WarningMessage createdAt={message.createdAt} message={message.content} />
    default:
      return null
  }
}
