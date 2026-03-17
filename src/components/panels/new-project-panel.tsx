import { Brain, Save, Wrench } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/atoms/button'
import { updateToast as toast } from '@/components/atoms/toaster'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { type Action, Page, type Tab } from '@/components/organisms/page/page'
import { Section } from '@/components/organisms/page/section'
import { ProjectAgentPrimerV2 } from '@/components/organisms/project/project-agent-primer-v2'
import { ProjectDetailsV2 } from '@/components/organisms/project/project-details-v2'
import {
  ProjectEditorViewModel,
  ProjectEditorViewModelContext,
  useProjectEditorViewModel,
} from '@/components/organisms/project/project-editor.viewmodel'
import { newProjectId } from '@/lib/utils/default-models'
import { projectPanelId } from '@/lib/utils/dockview-utils'
import { reportError } from '@/lib/utils/error-reporting'

enum TabPath {
  Details = 'details',
  Folders = 'folders',
  AgentPrimer = 'agent-primer',
}

export const NewProjectPanel = () => {
  const [viewmodel, setViewmodel] = useState<ProjectEditorViewModel | null>(null)

  useEffect(() => {
    const viewmodel = new ProjectEditorViewModel(newProjectId)
    viewmodel.init(false).then(() => setViewmodel(viewmodel))

    return () => viewmodel.destroy()
  }, [])

  const [activeTabPath, setActiveTabPath] = useState<string>('details')
  const dockviewLayout = useDockviewLayoutViewModel()
  const panelId = useMemo(() => projectPanelId(newProjectId), [])

  const handleCreate = async () => {
    if (!viewmodel?.isValid) return

    try {
      toast.loading({ title: 'Creating project...' })
      const result = await viewmodel.create()
      if (!result) return toast.error({ title: 'Failed to create project' })

      toast.success({ title: 'Project created successfully' })
      await dockviewLayout.closePanel(panelId)
      dockviewLayout.openPanel({
        component: DockviewContentComponent.Project,
        id: projectPanelId(result.id),
        params: { projectId: result.id },
        title: result.name,
      })
    } catch (error) {
      reportError(error, 'creating project')
      toast.error({ title: `Failed to create project: ${error}` })
    }
  }

  const handleToDetails = () => setActiveTabPath(TabPath.Details)
  const handleToAgentPrimer = () => setActiveTabPath(TabPath.AgentPrimer)

  if (!viewmodel) return null

  const tabs: Tab[] = [
    {
      content: <ProjectDetailsTab onNext={handleToAgentPrimer} />,
      icon: <Wrench />,
      label: 'Settings',
      path: TabPath.Details,
      title: 'Project Settings',
    },
    {
      content: <ProjectAgentPrimerTab onBack={handleToDetails} onNext={handleCreate} />,
      icon: <Brain />,
      label: 'Agent Primer',
      path: TabPath.AgentPrimer,
      title: 'Agent Primer',
    },
  ]

  const actions: Action[] = [
    {
      enabled: viewmodel.isValid,
      icon: <Save />,
      label: 'Create',
      onClick: handleCreate,
      variant: 'success',
    },
  ]

  return (
    <ProjectEditorViewModelContext.Provider value={viewmodel}>
      <Page
        actions={actions}
        activeTabPath={activeTabPath}
        subtitle="Create your project."
        tabs={tabs}
        title="New Project"
        onTabChange={setActiveTabPath}
      />
    </ProjectEditorViewModelContext.Provider>
  )
}

interface ProjectDetailsTabProps {
  onNext: () => void
}

export const ProjectDetailsTab = ({ onNext }: ProjectDetailsTabProps) => {
  const viewmodel = useProjectEditorViewModel()

  return (
    <>
      <ProjectDetailsV2 />
      <Section>
        <div className="flex justify-end w-full pt-4 gap-2">
          <Button
            data-tour="project-name-next"
            disabled={!viewmodel.name.trim().length}
            size="sm"
            variant="outline"
            onClick={onNext}
          >
            Next
          </Button>
        </div>
      </Section>
    </>
  )
}

interface ProjectAgentPrimerTabProps {
  onNext: () => void
  onBack: () => void
}

export const ProjectAgentPrimerTab = ({ onNext, onBack }: ProjectAgentPrimerTabProps) => {
  const viewmodel = useProjectEditorViewModel()

  return (
    <>
      <ProjectAgentPrimerV2 />
      <Section>
        <div className="flex justify-end w-full pt-4 gap-2">
          <Button size="sm" variant="outline-ghost" onClick={onBack}>
            Back
          </Button>
          <Button
            data-tour="project-agent-primer-next"
            disabled={!viewmodel.isValid}
            size="sm"
            variant="outline"
            onClick={onNext}
          >
            Create
          </Button>
        </div>
      </Section>
    </>
  )
}
