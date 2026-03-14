import { useEffect, useState } from 'react'
import type { MessageStatus } from '@/types'
import { SubagentConversation } from './subagent-conversation'
import { SubagentConversationViewmodel, SubagentConversationViewmodelContext } from './subagent-conversation-viewmodel'

interface SubagentConversationProviderProps {
  sessionId: string
  status: MessageStatus
}

export const SubagentConversationProvider = ({ sessionId, status }: SubagentConversationProviderProps) => {
  const [viewmodel, setViewModel] = useState<SubagentConversationViewmodel | null>(null)

  useEffect(() => {
    const viewmodel = new SubagentConversationViewmodel(sessionId)
    viewmodel.init().then(() => setViewModel(viewmodel))

    return () => viewmodel.destroy()
  }, [sessionId])

  if (!viewmodel) return null

  return (
    <SubagentConversationViewmodelContext.Provider value={viewmodel}>
      <SubagentConversation status={status} />
    </SubagentConversationViewmodelContext.Provider>
  )
}
