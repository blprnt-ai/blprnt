import { Brain, ListTodo, Trash, Wrench } from 'lucide-react'
import { useEffect, useState } from 'react'
import { DeleteProjectDialog } from '@/components/dialogs/project/delete-project-dialog'
import { type Action, Page, type Tab } from '@/components/organisms/page/page'
import { ProjectAgentPrimerV2 } from '@/components/organisms/project/project-agent-primer-v2'
import { ProjectDetailsV2 } from '@/components/organisms/project/project-details-v2'
import {
  ProjectEditorViewModel,
  ProjectEditorViewModelContext,
} from '@/components/organisms/project/project-editor.viewmodel'
import { ProjectMemoryFiles } from '@/components/organisms/project/project-memory-files'
import { ProjectPlansListV2 } from '@/components/organisms/project/project-plans-list-v2'

interface ProjectPanelProps {
  projectId: string
}

export const ProjectPanel = ({ projectId }: ProjectPanelProps) => {
  const [viewmodel, setViewmodel] = useState<ProjectEditorViewModel | null>(null)

  const [isDeleteProjectDialogOpen, setIsDeleteProjectDialogOpen] = useState(false)

  useEffect(() => {
    const viewmodel = new ProjectEditorViewModel(projectId)
    viewmodel.init(true).then(() => setViewmodel(viewmodel))

    return () => viewmodel.destroy()
  }, [projectId])

  if (!viewmodel) return null

  const tabs: Tab[] = [
    {
      content: <ProjectDetailsV2 />,
      icon: <Wrench />,
      label: 'Settings',
      path: 'details',
      title: 'Project Settings',
    },
    {
      content: <ProjectAgentPrimerV2 />,
      icon: <Brain />,
      label: 'Agent Primer',
      path: 'agent-primer',
      title: 'Agent Primer',
    },
    {
      content: <ProjectPlansListV2 projectId={viewmodel.id} />,
      icon: <ListTodo />,
      label: 'Plans',
      path: 'plans',
      title: 'Plans',
    },
    {
      content: <ProjectMemoryFiles projectId={viewmodel.id} />,
      icon: <Brain />,
      label: 'Memory',
      path: 'memory',
      title: 'Memory',
    },
  ]

  const actions: Action[] = [
    {
      enabled: true,
      icon: <Trash />,
      label: 'Delete Project',
      onClick: () => setIsDeleteProjectDialogOpen(true),
      variant: 'danger',
    },
  ]

  return (
    <>
      <ProjectEditorViewModelContext.Provider value={viewmodel}>
        <Page actions={actions} subtitle="Manage your project." tabs={tabs} title={viewmodel.name} />
      </ProjectEditorViewModelContext.Provider>

      {viewmodel && isDeleteProjectDialogOpen && (
        <DeleteProjectDialog
          isOpen={isDeleteProjectDialogOpen}
          projectId={projectId}
          onOpenChange={setIsDeleteProjectDialogOpen}
        />
      )}
    </>
  )
}
