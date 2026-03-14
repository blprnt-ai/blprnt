import { Building, Edit, FilePlus, FolderPlus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/atoms/button'
import { TooltipMacro } from '@/components/atoms/tooltip-macro'
import {
  TreeExpander,
  TreeIcon,
  TreeLabel,
  TreeNode,
  TreeNodeContent,
  TreeNodeTrigger,
  TreeProvider,
  TreeView,
} from '@/components/atoms/tree'
import { DeleteProjectDialog } from '@/components/dialogs/project/delete-project-dialog'
import { NewSessionDialog } from '@/components/dialogs/session/new-session-dialog'
import { useSidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import { ProjectTreeProvider, useProjectTreeViewmodel } from '@/components/organisms/trees/project-tree.provider'
import type { SessionModel } from '@/lib/models/session.model'
import { cn } from '@/lib/utils/cn'
import { SessionsTree } from './sessions-tree'
import { projectNodeId } from './utils'

export const ProjectsTree = () => {
  const viewmodel = useSidebarViewmodel()
  const projects = viewmodel.projects

  const handleCreateNewProject = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    viewmodel.openNewProject()
  }

  return (
    <TreeProvider
      className="overflow-y-auto max-h-[calc(100%-2.5rem)]"
      defaultExpandedIds={viewmodel.defaultExpandedIds}
      indent={6}
    >
      <TreeView className="h-full pb-4">
        <TreeNode>
          <TreeNodeTrigger className="hover:bg-transparent cursor-default" data-tour="sidebar-projects">
            <TreeExpander />
            <TreeIcon icon={<Building />} />
            <TreeLabel className={'hover:[&_button]:opacity-100 transition-colors duration-300'}>
              <div className="flex items-center justify-between">
                <div>Projects</div>
                <TooltipMacro withDelay tooltip="Create New Project">
                  <Button
                    className="text-muted-foreground hover:text-foreground size-8 rounded-none opacity-40"
                    size="icon"
                    variant="ghost"
                    onClick={handleCreateNewProject}
                  >
                    <FolderPlus />
                  </Button>
                </TooltipMacro>
              </div>
            </TreeLabel>
          </TreeNodeTrigger>
        </TreeNode>
        {projects.map((project) => (
          <ProjectTreeProvider key={project.id} project={project}>
            <ProjectTreeNode />
          </ProjectTreeProvider>
        ))}
      </TreeView>
    </TreeProvider>
  )
}

const ProjectTreeNode = () => {
  const viewmodel = useSidebarViewmodel()
  const projectViewmodel = useProjectTreeViewmodel()
  const project = projectViewmodel.project
  const [isNewSessionDialogOpen, setIsNewSessionDialogOpen] = useState(false)
  const [isDeleteProjectDialogOpen, setIsDeleteProjectDialogOpen] = useState(false)

  const handleOpenSession = (session: SessionModel | { id: string }) => viewmodel.openSession(project.id, session.id)
  const handleEditProject = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    viewmodel.openProject(project.id)
  }

  const handleCreateNewSession = (e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation()
    e.preventDefault()
    setIsNewSessionDialogOpen(true)
  }

  // const handleOpenPreview = (e: React.MouseEvent<HTMLButtonElement>) => {
  //   e.stopPropagation()
  //   e.preventDefault()
  //   panelsStore.openPreviewPanel(project.id)
  // }

  const { hasActive, hasOpen } = viewmodel.getProjectState(project.id)

  return (
    <>
      <TreeNode key={project.id} nodeId={projectNodeId(project.id)}>
        <TreeNodeTrigger data-tour="sidebar-project">
          <TreeExpander hasChildren />
          <TreeIcon hasChildren />
          <TreeLabel
            className={cn(
              'flex items-center justify-between text-foreground/60 hover:[&_button]:opacity-100 transition-colors duration-300',
              hasOpen && 'text-primary/80',
              hasActive && 'text-primary',
            )}
          >
            <div className="whitespace-nowrap overflow-hidden text-ellipsis">{project.name}</div>
            <div className="flex items-center">
              {/* <Button
                className="text-muted-foreground hover:text-foreground rounded-none opacity-40"
                size="icon-sm"
                variant="ghost"
                onClick={handleOpenPreview}
              >
                <Eye className="size-4" />
              </Button> */}
              <TooltipMacro withDelay tooltip="Create New Session">
                <Button
                  className="text-muted-foreground hover:text-foreground rounded-none opacity-40"
                  size="icon-sm"
                  variant="ghost"
                  onClick={handleCreateNewSession}
                >
                  <FilePlus />
                </Button>
              </TooltipMacro>
              <TooltipMacro withDelay tooltip="Edit Project">
                <Button
                  className="text-muted-foreground hover:text-foreground rounded-none opacity-40"
                  size="icon-sm"
                  variant="ghost"
                  onClick={handleEditProject}
                >
                  <Edit />
                </Button>
              </TooltipMacro>
            </div>
          </TreeLabel>
        </TreeNodeTrigger>
        <TreeNodeContent hasChildren>
          <SessionsTree projectId={project.id} />
        </TreeNodeContent>
      </TreeNode>

      {isNewSessionDialogOpen && (
        <NewSessionDialog
          initialProjectId={project.id}
          isOpen={isNewSessionDialogOpen}
          onAfterCreate={handleOpenSession}
          onOpenChange={setIsNewSessionDialogOpen}
        />
      )}

      {isDeleteProjectDialogOpen && (
        <DeleteProjectDialog
          isOpen={isDeleteProjectDialogOpen}
          projectId={project.id}
          onOpenChange={setIsDeleteProjectDialogOpen}
        />
      )}
    </>
  )
}
