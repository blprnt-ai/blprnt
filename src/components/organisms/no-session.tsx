import { Rocket } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { BgGrid } from '@/components/atoms/bg-grid'
import { GlassCard } from '@/components/atoms/glass-card'
import { ShineBorder } from '@/components/atoms/shine-border'
import { NewSessionDialog } from '@/components/dialogs/session/new-session-dialog'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { useDockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { ProjectModel } from '@/lib/models/project.model'
import { SessionModel } from '@/lib/models/session.model'
import { newProjectId } from '@/lib/utils/default-models'
import { projectPanelId, sessionPanelId } from '@/lib/utils/dockview-utils'
import { SessionSelector } from './session-selector'

export const NoSession = () => {
  const dockviewLayout = useDockviewLayoutViewModel()
  const [projects, setProjects] = useState<ProjectModel[]>([])
  const [sessions, setSessions] = useState<SessionModel[]>([])
  const [isNewSessionOpen, setIsNewSessionOpen] = useState(false)
  const projectNames = useMemo(() => new Map(projects.map((project) => [project.id, project.name])), [projects])

  const handleNewSession = () => setIsNewSessionOpen(true)

  const handleOpenSession = (session: SessionModel) => {
    const panelId = sessionPanelId(session.projectId ?? 'unknown', session.id)
    dockviewLayout.openPanel({
      component: DockviewContentComponent.Session,
      id: panelId,
      params: { projectId: session.projectId, sessionId: session.id },
      title: session.name,
    })
  }

  const handleNewProject = () => {
    dockviewLayout.openPanel({
      component: DockviewContentComponent.Project,
      id: projectPanelId(newProjectId),
      params: { projectId: newProjectId },
      title: 'New Project',
    })
  }

  const handleOpenSessionById = (_projectId: string, sessionId: string) => {
    const session = sessions.find((item) => item.id === sessionId)
    if (!session) return

    handleOpenSession(session)
  }

  useEffect(() => {
    let isMounted = true
    ProjectModel.list()
      .then((list) => {
        if (!isMounted) return
        setProjects(list)
      })
      .catch((error) => {
        console.error('Error loading projects', error)
      })

    return () => {
      isMounted = false
    }
  }, [])

  useEffect(() => {
    if (!projects.length) return
    let isMounted = true

    const loadSessions = async () => {
      const sessionsByProject = await Promise.all(projects.map((project) => SessionModel.list(project.id)))
      const flattened = sessionsByProject
        .flat()
        .filter((session) => !session.parentId)
        .toSorted((a, b) => (a.updatedAt < b.updatedAt ? 1 : -1))
      if (!isMounted) return
      setSessions(flattened)
    }

    loadSessions().catch((error) => {
      console.error('Error loading sessions', error)
    })

    return () => {
      isMounted = false
    }
  }, [projects])

  return (
    <>
      <BgGrid className="opacity-50" />
      <div className="flex h-full items-center justify-center">
        <div className="w-2xl flex flex-col gap-4">
          <div className="text-center mb-4 relative">
            <p className="mb-4 text-4xl">Let's get started!</p>
            {projects.length > 0 ? (
              <>
                <p className="mb-4 text-lg text-muted-foreground">What do you want to do first?</p>

                <div className="flex gap-4 justify-center">
                  <GlassCard
                    className="relative rainbow-parent hover:border-transparent hover:[&>div]:opacity-100 hover:[&>div]:duration-600 cursor-pointer w-full h-full"
                    role="button"
                    onClick={handleNewSession}
                  >
                    <ShineBorder
                      className="opacity-0 hover:opacity-100 transition-opacity duration-600 size-[calc(100%+1px)]"
                      shineColor={['#00c378', '#7a9fdd', '#e08700']}
                    />
                    <div className="rainbow-child text-lg font-medium">Create a new session</div>
                    <div className="text-sm text-muted-foreground flex items-center justify-center gap-2 w-full">
                      <div>Configure and start a new blprnt session</div>
                      <Rocket className="size-4 text-primary" />
                    </div>
                  </GlassCard>
                </div>
              </>
            ) : (
              <div className="flex gap-4 justify-center">
                <GlassCard
                  className="relative rainbow-parent hover:border-transparent hover:[&>div]:opacity-100 hover:[&>div]:duration-600 cursor-pointer w-full h-full"
                  data-tour="no-project-create-new-project"
                  role="button"
                  onClick={handleNewProject}
                >
                  <ShineBorder
                    className="opacity-0 hover:opacity-100 transition-opacity duration-600 size-[calc(100%+1px)]"
                    shineColor={['#00c378', '#7a9fdd', '#e08700']}
                  />
                  <div className="rainbow-child text-lg font-medium">Create a new project</div>
                  <div className="text-sm text-muted-foreground flex items-center justify-center gap-2 w-full">
                    <div>Projects are the backbone of blprnt. Create one to get started.</div>
                    <Rocket className="size-4 text-primary" />
                  </div>
                </GlassCard>
              </div>
            )}
          </div>

          <SessionSelector
            handleOpenSession={handleOpenSessionById}
            projectNames={projectNames}
            sessions={sessions}
            title="or pick up where you left off"
          />
        </div>
      </div>

      {isNewSessionOpen && (
        <NewSessionDialog
          isOpen={isNewSessionOpen}
          onAfterCreate={handleOpenSession}
          onOpenChange={setIsNewSessionOpen}
        />
      )}
    </>
  )
}
