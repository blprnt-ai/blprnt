import { AnimatePresence, motion } from 'framer-motion'
import { EmptyOutputPanel } from '@/components/panels/session/organisms/empty-output-panel'
import { SessionConversation } from '@/components/panels/session/organisms/session-conversation'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'

export const SessionOutput = () => {
  const viewmodel = useSessionPanelViewmodel()
  const session = viewmodel.session

  if (!session) return null

  return (
    <AnimatePresence mode="popLayout">
      <motion.div
        animate={{ opacity: 1 }}
        className="flex min-h-0 flex-1 flex-col max-h-full overflow-hidden relative pl-2 pr-1"
        initial={{ opacity: 0 }}
        transition={{ delay: 0.2, duration: 0.4 }}
      >
        {viewmodel.isEmpty && <EmptyOutputPanel />}
        {!viewmodel.isEmpty && <SessionConversation />}
      </motion.div>
    </AnimatePresence>
  )
}
