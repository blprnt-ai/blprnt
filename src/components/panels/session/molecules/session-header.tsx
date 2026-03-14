import { motion } from 'framer-motion'
import { SettingsIcon } from 'lucide-react'
import { useState } from 'react'
import { EditSessionDialog } from '@/components/dialogs/session/edit-session-dialog'
import { SessionChip } from '@/components/panels/session/atoms/session-chip'
import { useSessionPanelViewmodel } from '@/components/panels/session/session-panel.viewmodel'
import { useLlmModels } from '@/hooks/use-llm-models'
import { cn } from '@/lib/utils/cn'
import { upperFirst } from '@/lib/utils/string'

export const SessionHeader = () => {
  const { enabledModels } = useLlmModels()
  const viewmodel = useSessionPanelViewmodel()
  const session = viewmodel.session
  const [isDialogOpen, setIsDialogOpen] = useState(false)

  if (!session) return null

  const isSessionValid = enabledModels.some((m) => m.toggledOn && m.auto_router) || Boolean(session.modelOverride)

  const projectId = session.projectId

  return (
    <>
      <div className={cn('flex flex-row items-center justify-between h-8 shrink-0 border-b bg-accent relative z-10')}>
        <div className="flex flex-row items-center h-full">
          <SessionChip label="Model" value={viewmodel.chosenModel?.name ?? undefined} />
          <SessionChip label="Effort" value={session.reasoningEffort ?? undefined} />
          <SessionChip label="Prompt Mode" value={upperFirst(session.queueMode ?? '')} />
          <SessionChip label="Context Free" value={`${viewmodel.percentRemaining.toFixed(0)}%`} />

          {!isSessionValid && <SessionChip className="text-red-500" value="Invalid settings. Please select a model." />}
        </div>

        <motion.div
          animate={{ opacity: 1, x: 0 }}
          className="border-l border-border"
          data-tour="session-edit-session"
          initial={{ opacity: 0, x: 100 }}
          transition={{ delay: 0.3, duration: 0.25 }}
        >
          <div
            className="flex items-center justify-center hover:bg-primary/50 border-primary-dimmed size-8 border-none transition-colors duration-300"
            onClick={() => setIsDialogOpen(true)}
          >
            <SettingsIcon className="size-4 text-primary transition-colors duration-300" />
          </div>
        </motion.div>
      </div>

      {projectId && <EditSessionDialog isOpen={isDialogOpen} sessionId={session.id} onOpenChange={setIsDialogOpen} />}
    </>
  )
}
