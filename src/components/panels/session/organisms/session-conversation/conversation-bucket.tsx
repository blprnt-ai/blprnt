import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { PromptMessageModel } from '@/lib/models/messages/prompt-message.model'
import { ResponseMessageModel } from '@/lib/models/messages/response-message.model'
import { SignalMessageModel } from '@/lib/models/messages/signal-message.model'
import { ConversationMessageCard } from './conversation-message-card'

export const ConversationBucket = ({ messageKeys }: { messageKeys: string[] }) => {
  const viewmodel = useSessionPanelViewmodel()

  return (
    <>
      {messageKeys.map((messageKey) => {
        const message = viewmodel.getMessageByKey(messageKey)
        if (!message) return null
        if (message instanceof PromptMessageModel && message.status === 'pending') return null
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
