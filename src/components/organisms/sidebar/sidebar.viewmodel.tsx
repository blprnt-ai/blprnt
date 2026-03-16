import type { DockviewApi, GroupviewPanelState, SerializedDockview } from 'dockview-react'
import { flow, makeAutoObservable, reaction, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import type { AppViewModel } from '@/app.viewmodel'
import type { ReportBugDialogViewModel } from '@/components/dialogs/report-bug-dialog.viewmodel'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import type { DockviewLayoutViewModel } from '@/components/dockview/dockview-layout.viewmodel'
import { projectNodeId, sessionNodeId, sessionsNodeId } from '@/components/organisms/trees/utils'
import type { SettingsTabs } from '@/components/views/settings/settings-page'
import { EventType, globalEventBus, type InternalEvent } from '@/lib/events/event-bus'
import { PanelModel, type PanelSnapshot, PanelType } from '@/lib/models/panel.model'

import { ProjectModel } from '@/lib/models/project.model'
import { SessionModel } from '@/lib/models/session.model'
import { newProjectId } from '@/lib/utils/default-models'
import { previewPanelId, projectPanelId, sessionPanelId } from '@/lib/utils/dockview-utils'

export class SidebarViewmodel {
  private dockviewDisposers: Array<() => void> = []
  private reactionDisposers: Array<() => void> = []
  private projectModels: ProjectModel[] = []
  private panelModels: PanelModel[] = []
  private activePanelId: string | null = null
  private sessionsByProjectId = new Map<string, SessionModel[]>()
  private sessionsLoadingProjects = new Set<string>()
  private unsubscribers: Array<() => void> = []

  constructor(
    readonly appStore: AppViewModel,
    readonly dockviewLayout: DockviewLayoutViewModel,
    readonly reportBugDialogViewmodel: ReportBugDialogViewModel,
  ) {
    makeAutoObservable(this, { appStore: false }, { autoBind: true })
  }

  init = () => {
    this.loadProjectsAndSessions()
    this.loadPanelsFromLocalStorage()
    this.attachContainerApi(this.dockviewLayout.containerApi)
    this.reactionDisposers.push(
      reaction(
        () => this.dockviewLayout.containerApi,
        (containerApi) => this.attachContainerApi(containerApi),
      ),
    )

    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.Internal, (event) => {
        this.handleInternalEvent(event.payload.event)
      }),
    )
  }

  destroy = () => {
    this.dockviewDisposers.forEach((dispose) => dispose())
    this.dockviewDisposers = []
    this.reactionDisposers.forEach((dispose) => dispose())
    this.reactionDisposers = []
    this.unsubscribers.forEach((unsubscribe) => unsubscribe())
    this.unsubscribers = []
  }

  private loadProjects = flow(function* (this: SidebarViewmodel) {
    const projectModels = yield ProjectModel.list()

    this.projectModels = projectModels
  })

  private handleInternalEvent = (event: InternalEvent) => {
    switch (event.type) {
      case 'project_added':
      case 'project_removed':
      case 'project_updated':
        void this.loadProjectsAndSessions()
        break
      case 'session_added':
        void this.loadSessionById(event.sessionId)
        break
    }
  }

  private loadProjectsAndSessions = flow(function* (this: SidebarViewmodel) {
    yield this.loadProjects()
    yield this.loadAllSessions()
  })

  private loadAllSessions = flow(function* (this: SidebarViewmodel) {
    yield Promise.all(this.projectModels.map((project) => this.ensureSessionsLoaded(project.id)))
  })

  private ensureSessionsLoaded = flow(function* (this: SidebarViewmodel, projectId: string) {
    if (this.sessionsByProjectId.has(projectId) || this.sessionsLoadingProjects.has(projectId)) return
    this.sessionsLoadingProjects.add(projectId)

    try {
      const sessions = yield SessionModel.list(projectId)
      runInAction(() => {
        this.sessionsByProjectId.set(projectId, sessions)
      })
      this.updateSessionPanelTitles(projectId)
    } catch (error) {
      console.error('Error loading sessions', { error, projectId })
    } finally {
      runInAction(() => {
        this.sessionsLoadingProjects.delete(projectId)
      })
    }
  })

  private updateSessionPanelTitles = (projectId: string) => {
    const sessions = this.sessionsByProjectId.get(projectId) ?? []
    if (!sessions.length) return

    sessions.forEach((session) => this.updateSessionPanelTitle(projectId, session))
  }

  private loadSessionById = flow(function* (this: SidebarViewmodel, sessionId: string) {
    const session = yield SessionModel.get(sessionId)
    if (!session?.projectId) return

    this.sessionsByProjectId.get(session.projectId)?.push(session)
    this.updateSessionPanelTitle(session.projectId, session)
  })

  private updateSessionPanelTitle = (projectId: string, session: SessionModel) => {
    const panelId = sessionPanelId(projectId, session.id)
    const title = session.name
    this.dockviewLayout.containerApi?.getPanel(panelId)?.setTitle(title)
    const panelModel = this.panelModels.find((panel) => panel.id === panelId)
    if (!panelModel) return
    panelModel.updateFrom({
      id: panelModel.id,
      isActive: panelModel.isActive,
      isVisible: panelModel.isVisible,
      params: panelModel.params,
      title,
      type: panelModel.type,
    })
  }

  private loadPanelsFromLocalStorage = () => {
    const layoutJson = localStorage.getItem('dockview-layout')
    if (!layoutJson) return

    try {
      const layout = JSON.parse(layoutJson) as SerializedDockview
      this.panelModels = this.buildPanelsFromLayout(layout)
    } catch (error) {
      console.error('Error parsing layout', error)
    }
  }

  private attachContainerApi = (containerApi: DockviewApi | null) => {
    if (!containerApi) return

    this.dockviewDisposers.forEach((dispose) => dispose())
    this.dockviewDisposers = []

    this.syncPanelsFromDockview(containerApi)

    const layoutDisposable = containerApi.onDidLayoutChange(() => {
      this.syncPanelsFromDockview(containerApi)
    })
    const activeDisposable = containerApi.onDidActivePanelChange((event) => this.setActivePanelId(event?.id ?? null))

    this.dockviewDisposers.push(() => layoutDisposable.dispose())
    this.dockviewDisposers.push(() => activeDisposable.dispose())
  }

  private syncPanelsFromDockview = (containerApi: DockviewApi) => {
    const layout = containerApi.toJSON()
    if (!layout) return

    this.panelModels = this.buildPanelsFromLayout(layout as SerializedDockview)
    this.setActivePanelId(containerApi.activePanel?.id ?? null)
  }

  private setActivePanelId = (panelId: string | null) => {
    this.activePanelId = panelId
    this.panelModels.forEach((panel) => panel.update({ isActive: panel.id === panelId }))
  }

  private buildPanelsFromLayout = (layout: SerializedDockview) => {
    const panels: PanelModel[] = []

    for (const panelState of Object.values(layout.panels)) {
      const snapshot = this.panelSnapshotFromState(panelState as GroupviewPanelState)
      if (!snapshot) continue
      panels.push(new PanelModel(snapshot))
    }

    return panels
  }

  private panelSnapshotFromState = (panelState: GroupviewPanelState): PanelSnapshot | null => {
    const panelType = this.getPanelTypeFromJsonPanel(panelState)
    if (!panelType) return null

    const params = (panelState.params ?? {}) as Record<string, unknown>
    const title = panelState.title ?? this.getDefaultPanelTitle(panelType, params)

    return {
      id: panelState.id as string,
      params,
      title,
      type: panelType,
    }
  }

  private getPanelTypeFromJsonPanel = (panelState: GroupviewPanelState) => {
    switch (panelState.contentComponent) {
      case DockviewContentComponent.Intro:
        return PanelType.Intro
      case DockviewContentComponent.Personality:
        return PanelType.Personality
      case DockviewContentComponent.Session:
        return PanelType.Session
      case DockviewContentComponent.Project:
        return PanelType.Project
      case DockviewContentComponent.Plan:
        return PanelType.Plan
      case DockviewContentComponent.UserAccount:
        return PanelType.UserAccount
      case DockviewContentComponent.Preview:
        return PanelType.Preview
      default:
        return null
    }
  }

  private getProjectById = (projectId: string) => {
    return this.projectModels.find((project) => project.id === projectId)
  }

  private getSessionName = (projectId: string, sessionId: string) => {
    if (!this.sessionsByProjectId.has(projectId)) {
      void this.ensureSessionsLoaded(projectId)
    }

    const sessions = this.sessionsByProjectId.get(projectId) ?? []
    const session = sessions.find((item) => item.id === sessionId)

    return session?.name ?? 'Loading...'
  }

  private getDefaultPanelTitle = (panelType: PanelType, params: Record<string, unknown>) => {
    const projectId = this.getParamString(params, 'projectId')
    const sessionId = this.getParamString(params, 'sessionId')

    const project = projectId ? this.getProjectById(projectId) : null
    const projectName = project?.name ?? 'New Project'

    switch (panelType) {
      case PanelType.Intro:
        return 'Intro'
      case PanelType.Personality:
        return 'Personalities'
      case PanelType.UserAccount:
        return 'Settings'
      case PanelType.Project:
        return projectId === newProjectId ? 'New Project' : projectName
      case PanelType.Session:
        return projectId && sessionId ? this.getSessionName(projectId, sessionId) : 'Loading 3...'
      case PanelType.Preview:
        return project ? `${projectName} - Preview` : 'Preview'
      case PanelType.Plan:
        return 'Plan'
      default:
        return 'Panel'
    }
  }

  get projects() {
    return this.projectModels.toSorted((a, b) => a.name.localeCompare(b.name))
  }

  get defaultExpandedIds() {
    const panelNodeIds = new Set(
      this.panelModels
        .flatMap((panel) => {
          const projectId = this.getPanelParamString(panel, 'projectId')
          const sessionId = this.getPanelParamString(panel, 'sessionId')

          switch (panel.type) {
            case PanelType.Session:
              if (!projectId || !sessionId) return null
              return [projectNodeId(projectId), sessionsNodeId(projectId), sessionNodeId(projectId, sessionId)]

            case PanelType.Preview:
              return projectId ? projectNodeId(projectId) : null
            default:
              return null
          }
        })
        .filter((id): id is string => id !== null),
    )

    return panelNodeIds.values().toArray()
  }

  openUserAccount = (tab: SettingsTabs) => {
    this.openPanel(
      this.buildPanelSnapshot(PanelType.UserAccount, 'user-account', { tab }),
      DockviewContentComponent.UserAccount,
    )
  }

  openReportBug = () => {
    globalEventBus.emit(EventType.ReportBugMenuClicked, null)
  }

  get isReportBugAvailable() {
    return true
  }

  openNewProject = () => this.openProject(newProjectId)
  openProject = (projectId: string) => {
    const panelId = projectPanelId(projectId)
    this.openPanel(this.buildPanelSnapshot(PanelType.Project, panelId, { projectId }), DockviewContentComponent.Project)
  }
  openPreview = (projectId: string) => {
    const panelId = previewPanelId(projectId)
    this.openPanel(this.buildPanelSnapshot(PanelType.Preview, panelId, { projectId }), DockviewContentComponent.Preview)
  }
  openSession = (projectId: string, sessionId: string) => {
    const panelId = sessionPanelId(projectId, sessionId)
    this.openPanel(
      this.buildPanelSnapshot(PanelType.Session, panelId, { projectId, sessionId }),
      DockviewContentComponent.Session,
    )
  }

  getProjectState = (projectId: string) => {
    const isProjectActive = this.isProjectPanelActive(projectId)
    const isPreviewActive = this.isProjectPreviewActive(projectId)
    const hasActiveSession = this.projectHasActiveSession(projectId)

    const isProjectOpen = this.isProjectPanelOpen(projectId)
    const isPreviewOpen = this.isProjectPreviewOpen(projectId)
    const hasOpenSession = this.projectHasOpenSession(projectId)

    return {
      hasActive: isProjectActive || isPreviewActive || hasActiveSession,
      hasOpen: isProjectOpen || isPreviewOpen || hasOpenSession,
    }
  }

  getSessionsState = (projectId: string) => {
    return {
      hasActiveSession: this.projectHasActiveSession(projectId),
      hasOpenSession: this.projectHasOpenSession(projectId),
    }
  }

  getSessionPanelState = (projectId: string, sessionId: string) => {
    const panelId = sessionPanelId(projectId, sessionId)

    return {
      closePanel: () => this.closePanel(panelId),
      isPanelActive: this.isSessionPanelActive(projectId, sessionId),
      isPanelOpen: this.isSessionPanelOpen(sessionId),
    }
  }

  private getPanelsByProjectId = (projectId: string) => {
    return this.panelModels.filter((panel) => this.getPanelParamString(panel, 'projectId') === projectId)
  }

  private isPanelActive = (panel: PanelModel) => {
    return panel.id === this.activePanelId || panel.isActive
  }

  private isProjectPanelActive = (projectId: string) => {
    return (
      this.getPanelsByProjectId(projectId).filter(
        (panel) => panel.type === PanelType.Project && this.isPanelActive(panel),
      ).length > 0
    )
  }

  private isProjectPanelOpen = (projectId: string) => {
    return this.getPanelsByProjectId(projectId).filter((panel) => panel.type === PanelType.Project).length > 0
  }

  private isProjectPreviewActive = (projectId: string) => {
    return this.activePanelId === previewPanelId(projectId)
  }

  private isProjectPreviewOpen = (projectId: string) => {
    return this.getPanelsByProjectId(projectId).filter((panel) => panel.type === PanelType.Preview).length > 0
  }

  private isSessionPanelActive = (projectId: string | undefined, sessionId: string) => {
    if (!projectId) return false

    return this.activePanelId === sessionPanelId(projectId, sessionId)
  }

  private isSessionPanelOpen = (sessionId: string) =>
    this.panelModels.filter((panel) => this.getPanelParamString(panel, 'sessionId') === sessionId).length > 0

  private projectHasActiveSession = (projectId: string) => {
    return (
      this.panelModels.filter((panel) => {
        if (panel.type !== PanelType.Session) return false

        const sessionId = this.getPanelParamString(panel, 'sessionId')
        if (!sessionId) return false

        return (
          this.activePanelId === sessionPanelId(projectId, sessionId) &&
          this.getPanelParamString(panel, 'projectId') === projectId
        )
      }).length > 0
    )
  }

  private projectHasOpenSession = (projectId: string) => {
    return (
      this.panelModels.filter(
        (panel) => panel.type === PanelType.Session && this.getPanelParamString(panel, 'projectId') === projectId,
      ).length > 0
    )
  }

  private openPanel = (snapshot: PanelSnapshot, component: DockviewContentComponent) => {
    this.dockviewLayout.openPanel({
      component,
      id: snapshot.id,
      params: snapshot.params,
      title: snapshot.title,
    })
    this.upsertPanelModel(snapshot)
  }

  private closePanel = (panelId: string) => {
    const containerApi = this.dockviewLayout.containerApi
    if (!containerApi) return

    containerApi.getPanel(panelId)?.api.close()
  }

  private upsertPanelModel = (snapshot: PanelSnapshot) => {
    const existing = this.panelModels.find((panel) => panel.id === snapshot.id)
    if (existing) {
      existing.updateFrom(snapshot)
    } else {
      this.panelModels = [...this.panelModels, new PanelModel(snapshot)]
    }

    this.setActivePanelId(snapshot.id)
  }

  private buildPanelSnapshot = (type: PanelType, panelId: string, params: Record<string, unknown>): PanelSnapshot => {
    return {
      id: panelId,
      params,
      title: this.getDefaultPanelTitle(type, params),
      type,
    }
  }

  private getPanelParamString = (panel: PanelModel, key: string) => {
    return this.getParamString(panel.params, key)
  }

  private getParamString = (params: Record<string, unknown>, key: string) => {
    const value = params[key]
    return typeof value === 'string' ? value : null
  }
}

export const SidebarViewmodelContext = createContext<SidebarViewmodel | null>(null)

export const useSidebarViewmodel = () => {
  const viewmodel = useContext(SidebarViewmodelContext)
  if (!viewmodel) throw new Error('useSidebarViewmodel must be used within SidebarProvider')

  return viewmodel
}
