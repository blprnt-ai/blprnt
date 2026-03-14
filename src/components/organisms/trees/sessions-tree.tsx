import { ArrowDownUp, FolderPlus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/atoms/button'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import { TreeExpander, TreeIcon, TreeLabel, TreeNode, TreeNodeTrigger } from '@/components/atoms/tree'
import { NewSessionDialog } from '@/components/dialogs/session/new-session-dialog'
import { SessionsTreeProvider, useSessionsTreeViewmodel } from '@/components/organisms/trees/sessions-tree.provider'
import type { SessionModel } from '@/lib/models/session.model'
import { SessionTree } from './session-tree'

interface SessionsTreeProps {
  projectId: string
}

export const SessionsTree = ({ projectId }: SessionsTreeProps) => {
  return (
    <SessionsTreeProvider projectId={projectId}>
      <SessionsTreeContent />
    </SessionsTreeProvider>
  )
}

const SessionsTreeContent = () => {
  const viewmodel = useSessionsTreeViewmodel()
  const [createSession, setCreateSession] = useState(false)

  const handleCreateSession = () => {
    setCreateSession(true)
  }

  const handleAfterCreateSession = (session: SessionModel | { id: string }) => viewmodel.openSession(session.id)

  const handleSortSessions = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    viewmodel.sortSessions()
  }

  return (
    <>
      <TreeNode level={1}>
        <TreeNodeTrigger data-tour="sidebar-create-new-session" onClick={handleCreateSession}>
          <TreeExpander />
          <TreeIcon icon={<FolderPlus />} />
          <TreeLabel className="hover:[&_button]:opacity-100 transition-colors duration-300">
            <div className="flex items-center justify-between">
              <div>Create New Session</div>
              <TooltipMacro withDelay tooltip={viewmodel.sortColumn === 'name' ? 'Sort by Date' : 'Sort by Name'}>
                <Button
                  className="text-muted-foreground hover:text-foreground size-8 rounded-none opacity-40"
                  size="icon"
                  variant="ghost"
                  onClick={handleSortSessions}
                >
                  <ArrowDownUp />
                </Button>
              </TooltipMacro>
            </div>
          </TreeLabel>
        </TreeNodeTrigger>
      </TreeNode>

      {viewmodel.visibleSessions.map((session) => (
        <SessionTree key={session.id} projectId={viewmodel.projectId} session={session} />
      ))}

      {createSession && (
        <NewSessionDialog
          initialProjectId={viewmodel.projectId}
          isOpen={createSession}
          onAfterCreate={handleAfterCreateSession}
          onOpenChange={setCreateSession}
        />
      )}
    </>
  )
}
