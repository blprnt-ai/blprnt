import type { MessageModel } from '@/lib/models/messages/message-factory'
import { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SignalMessageModel } from '@/lib/models/messages/signal-message.model'
import { ConversationMessageCard } from './conversation-message-card'

export const ConversationBucket = ({
  messageKeys,
  messages,
}: {
  messageKeys: string[]
  messages: Map<string, MessageModel>
}) => {
  return (
    <>
      {messageKeys.map((messageKey) => {
        const message = messages.get(messageKey)
        if (!message) return null
        if (
          message instanceof PromptMessageModel ||
          message instanceof ResponseMessageModel ||
          message instanceof SignalMessageModel
        ) {
          return <ConversationMessageCard key={messageKey} message={message} />
        }
        return null
      })}
    </>
  )
}
