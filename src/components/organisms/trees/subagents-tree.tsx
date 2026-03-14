import { Edit, FolderCode, X } from 'lucide-react'
import { useMemo, useRef, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { TreeExpander, TreeIcon, TreeLabel, TreeNode, TreeNodeTrigger } from '@/components/atoms/tree'
import { EditSessionDialog } from '@/components/dialogs/session/edit-session-dialog'
import { useSidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import { useTextTruncated } from '@/hooks/use-is-text-truncated'
import type { SessionModel } from '@/lib/models/session.model'
import { cn } from '@/lib/utils/cn'

interface SubagentTreeProps {
  session: SessionModel
  projectId: string
}

export const SubagentTree = ({ session, projectId }: SubagentTreeProps) => {
  const viewmodel = useSidebarViewmodel()
  const ref = useRef<HTMLDivElement>(null)
  const isTruncated = useTextTruncated(ref)
  const [isEditSessionDialogOpen, setIsEditSessionDialogOpen] = useState(false)

  const {
    isPanelOpen: isOpen,
    isPanelActive: isActive,
    closePanel,
  } = viewmodel.getSessionPanelState(projectId, session.id)

  const handleOpenSession = () => viewmodel.openSession(projectId, session.id)

  const labelClassName = useMemo(() => {
    return cn(
      'text-muted-foreground font-medium italic flex items-center justify-between',
      isOpen && 'text-primary/80',
      isActive && 'text-primary not-italic [&_button]:text-muted-foreground',
    )
  }, [isActive, isOpen])

  const handleEditSession = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    setIsEditSessionDialogOpen(true)
  }

  const handleCloseSession = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    closePanel()
  }

  return (
    <>
      <TreeNode className="hover:[&_button]:opacity-100" level={3}>
        <TreeNodeTrigger onClick={handleOpenSession}>
          <TreeExpander />
          <TreeIcon icon={<FolderCode />} />
          <TreeLabel className={labelClassName}>
            <TooltipMacro
              withDelay
              disabled={!isTruncated}
              tooltip={session.name || 'Empty Session'}
              onClick={handleOpenSession}
            >
              <div ref={ref} className="whitespace-nowrap text-ellipsis overflow-hidden">
                {session.name || 'Empty Session'}
              </div>
            </TooltipMacro>
            <div className="flex items-center">
              <Button
                className="text-muted-foreground hover:text-foreground size-8 rounded-none opacity-0"
                size="icon"
                variant="ghost"
                onClick={handleEditSession}
              >
                <Edit className="size-4" />
              </Button>
              {isOpen && (
                <Button
                  className="text-muted-foreground size-8 rounded-none opacity-0"
                  size="icon"
                  variant="destructive-ghost"
                  onClick={handleCloseSession}
                >
                  <X className="size-4" />
                </Button>
              )}
            </div>
          </TreeLabel>
        </TreeNodeTrigger>
      </TreeNode>

      {isEditSessionDialogOpen && (
        <EditSessionDialog
          isOpen={isEditSessionDialogOpen}
          sessionId={session.id}
          onOpenChange={setIsEditSessionDialogOpen}
        />
      )}
    </>
  )
}
