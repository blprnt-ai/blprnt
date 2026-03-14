import { SubagentCard } from '@/components/panels/session/organisms/session-conversation/subagent-card'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { SubAgentMessageModel } from '@/lib/models/messages/subagent-message.model'

export const SubAgentBucket = ({ messageKeys }: { messageKeys: string[] }) => {
  const viewmodel = useSessionPanelViewmodel()
  if (!messageKeys.length) return null

  return (
    <div className="flex flex-col gap-2">
      {messageKeys.map((messageKey) => {
        const message = viewmodel.getMessageByKey(messageKey)
        if (!(message instanceof SubAgentMessageModel)) return null
        return <SubagentCard key={messageKey} messageKey={messageKey} />
      })}
    </div>
  )
}
