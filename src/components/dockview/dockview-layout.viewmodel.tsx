import type {
  Direction,
  DockviewApi,
  DockviewReadyEvent,
  GroupviewPanelState,
  SerializedDockview,
} from 'dockview-react'
import { flow, makeAutoObservable } from 'mobx'
import { createContext, useContext } from 'react'
import { DockviewContentComponent } from '@/components/dockview/content-components'
import { SessionModel } from '@/lib/models/session.model'

export class DockviewLayoutViewModel {
  containerApi: DockviewApi | null = null
  activePanelId: string | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  setContainerApi = (api: DockviewApi) => {
    this.containerApi = api
  }

  setActivePanelId = (panelId: string | null) => {
    this.activePanelId = panelId
  }

  loadLayoutFromStorage = () => {
    const layoutJson = localStorage.getItem('dockview-layout')
    if (!layoutJson || !this.containerApi) return

    try {
      const layout = JSON.parse(layoutJson)
      this.containerApi.fromJSON(layout)
      this.setActivePanelId(this.containerApi.activePanel?.id ?? null)
    } catch (error) {
      console.error('Error parsing layout', error)
    }
  }

  saveLayoutToStorage = () => {
    if (!this.containerApi) return
    const layout = this.containerApi.toJSON()
    if (!layout) return
    localStorage.setItem('dockview-layout', JSON.stringify(layout))
  }

  openPanel = (params: {
    id: string
    title: string
    component: DockviewContentComponent
    params?: Record<string, unknown>
    direction?: 'self' | Direction
  }) => {
    if (!this.containerApi) return
    const panel = this.containerApi.getPanel(params.id)

    if (!panel) {
      if (params.direction && params.direction !== 'self') {
        const activeGroup = this.containerApi.activeGroup
        if (!activeGroup) return

        this.containerApi.addPanel({
          component: params.component,
          id: params.id,
          params: params.params,
          position: {
            direction: params.direction,
            referenceGroup: activeGroup,
          },
          title: params.title,
        })
      } else {
        this.containerApi.addPanel({
          component: params.component,
          id: params.id,
          params: params.params,
          title: params.title,
        })
      }
    } else if (params.component === DockviewContentComponent.UserAccount && panel.params?.tab !== params.params?.tab) {
      panel.api.close()
      this.containerApi.addPanel({
        component: params.component,
        id: params.id,
        params: params.params,
        title: params.title,
      })
    }

    this.containerApi.getPanel(params.id)?.focus()
  }

  closePanelsByPredicate = (predicate: (panel: { id: string; params: Record<string, unknown> }) => boolean) => {
    const layout = this.containerApi?.toJSON() as SerializedDockview | undefined
    if (!layout) return

    for (const panelState of Object.values(layout.panels ?? {})) {
      const state = panelState as GroupviewPanelState
      const id = state.id as string
      const params = (state.params ?? {}) as Record<string, unknown>
      if (!predicate({ id, params })) continue
      void this.closePanel(id)
    }
  }

  closePanel = flow(function* (this: DockviewLayoutViewModel, panelId: string) {
    const panel = this.containerApi?.getPanel(panelId)
    if (!panel) return

    const params = (panel.params ?? {}) as Record<string, unknown>
    const sessionId = typeof params.sessionId === 'string' ? params.sessionId : null

    if (panelId.startsWith('session-') && sessionId) {
      yield SessionModel.stopById(sessionId)
    }

    panel.api.close()
  })

  onReady = (event: DockviewReadyEvent) => {
    this.setContainerApi(event.api)
    this.loadLayoutFromStorage()
  }
}

export const DockviewLayoutViewModelContext = createContext<DockviewLayoutViewModel | null>(null)

export const useDockviewLayoutViewModel = () => {
  const viewmodel = useContext(DockviewLayoutViewModelContext)
  if (!viewmodel) throw new Error('useDockviewLayoutViewModel must be used within DockviewLayoutViewModelProvider')
  return viewmodel
}
