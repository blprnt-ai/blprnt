import { Edit, FolderCode, FolderOpen, LoaderCircleIcon, Trash2, X } from 'lucide-react'
import { useMemo, useRef, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { TreeExpander, TreeIcon, TreeLabel, TreeNode, TreeNodeTrigger } from '@/components/atoms/tree'
import { DeleteSessionDialog } from '@/components/dialogs/session/delete-session-dialog'
import { EditSessionDialog } from '@/components/dialogs/session/edit-session-dialog'
import { SessionTreeProvider, useSessionTreeViewmodel } from '@/components/organisms/trees/session-tree.provider'
import { useTextTruncated } from '@/hooks/use-is-text-truncated'
import type { SessionModel } from '@/lib/models/session.model'
import { cn } from '@/lib/utils/cn'
import { sessionNodeId } from './utils'

interface SessionTreeProps {
  session: SessionModel
  projectId: string
}

export const SessionTree = ({ session, projectId }: SessionTreeProps) => {
  return (
    <SessionTreeProvider projectId={projectId} session={session}>
      <SessionTreeContent />
    </SessionTreeProvider>
  )
}

const SessionTreeContent = () => {
  const viewmodel = useSessionTreeViewmodel()
  const { projectId, session } = viewmodel
  const [isExpanded, setIsExpanded] = useState(false)
  const truncatedRef = useRef<HTMLDivElement>(null)
  const isTruncated = useTextTruncated(truncatedRef)
  const [isEditSessionDialogOpen, setIsEditSessionDialogOpen] = useState(false)
  const [isDeleteSessionDialogOpen, setIsDeleteSessionDialogOpen] = useState(false)
  const { isPanelOpen, isPanelActive, closePanel } = viewmodel.panelState
  const isRunning = viewmodel.isRunning

  const labelClassName = useMemo(() => {
    return cn(
      'text-muted-foreground font-medium italic flex items-center justify-between',
      isPanelOpen && 'text-primary/60',
      isPanelActive && 'text-primary not-italic [&_button]:text-muted-foreground',
    )
  }, [isPanelActive, isPanelOpen])

  const handleOpenSession = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation()
    e.preventDefault()
    viewmodel.openSession()
  }

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

  const handleDeleteSession = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    setIsDeleteSessionDialogOpen(true)
  }

  const hasChildren = false

  const maybeOpenSession = (e: React.MouseEvent<HTMLDivElement>) => {
    if (hasChildren) return

    e.stopPropagation()
    e.preventDefault()
    viewmodel.openSession()
  }

  return (
    <>
      <TreeNode
        className="hover:[&_button]:opacity-80 hover:[&_div[data-hover-target]]:bg-primary/10"
        level={1}
        nodeId={sessionNodeId(projectId, session.id)}
      >
        <TreeNodeTrigger data-tour="sidebar-session" onClick={maybeOpenSession}>
          <div
            data-hover-target
            className={cn(
              'flex items-center rounded-md w-full mr-4 overflow-hidden',
              isPanelOpen && isPanelActive && 'bg-linear-to-r from-transparent to-primary/30',
              isRunning && 'animate-pulse',
            )}
          >
            <TreeExpander hasChildren={hasChildren} onChangeExpanded={setIsExpanded} />
            <TreeIcon
              hasChildren={hasChildren}
              icon={
                isRunning ? (
                  <LoaderCircleIcon className="animate-spin" />
                ) : (isExpanded && hasChildren) || isPanelOpen ? (
                  <FolderOpen />
                ) : (
                  <FolderCode />
                )
              }
            />
            <TreeLabel className={labelClassName}>
              <div
                ref={truncatedRef}
                className="overflow-hidden text-ellipsis transition-all duration-300 hover:translate-x-1"
                onClick={handleOpenSession}
              >
                <TooltipMacro withDelay disabled={!isTruncated} tooltip={session.name}>
                  {session.name}
                </TooltipMacro>
              </div>
              <div className="flex items-center">
                <TooltipMacro withDelay tooltip="Edit Session">
                  <Button
                    className="text-muted-foreground hover:text-foreground size-8 rounded-none opacity-0"
                    size="icon"
                    variant="ghost"
                    onClick={handleEditSession}
                  >
                    <Edit className="size-4" />
                  </Button>
                </TooltipMacro>
                {isPanelOpen && (
                  <TooltipMacro withDelay tooltip="Close Session Tab">
                    <Button
                      className="text-muted-foreground size-8 rounded-none opacity-0"
                      size="icon"
                      variant="destructive-ghost"
                      onClick={handleCloseSession}
                    >
                      <X className="size-4" />
                    </Button>
                  </TooltipMacro>
                )}
                {!isPanelOpen && (
                  <TooltipMacro withDelay tooltip="Delete Session">
                    <Button
                      className="text-muted-foreground hover:text-foreground size-8 rounded-none opacity-0"
                      size="icon"
                      variant="destructive-ghost"
                      onClick={handleDeleteSession}
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  </TooltipMacro>
                )}
              </div>
            </TreeLabel>
          </div>
        </TreeNodeTrigger>
      </TreeNode>

      {isEditSessionDialogOpen && (
        <EditSessionDialog
          isOpen={isEditSessionDialogOpen}
          sessionId={session.id}
          onOpenChange={setIsEditSessionDialogOpen}
        />
      )}

      {isDeleteSessionDialogOpen && (
        <DeleteSessionDialog
          isOpen={isDeleteSessionDialogOpen}
          sessionId={session.id}
          onOpenChange={setIsDeleteSessionDialogOpen}
        />
      )}
    </>
  )
}
