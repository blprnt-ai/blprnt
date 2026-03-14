import dayjs from 'dayjs'
import { flow, makeAutoObservable, onBecomeObserved } from 'mobx'
import type { SidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import { EventType, globalEventBus, type InternalEvent } from '@/lib/events/event-bus'
import { SessionModel } from '@/lib/models/session.model'

export class SessionsTreeViewmodel {
  public sessions: SessionModel[] = []
  private hasLoaded = false
  public isLoading = false

  private unsubscribers: Array<() => void> = []

  public sortColumn: 'name' | 'createdAt' = 'name'

  constructor(
    private readonly sidebar: SidebarViewmodel,
    public readonly projectId: string,
  ) {
    makeAutoObservable<SessionsTreeViewmodel, 'sidebar'>(this, { sidebar: false }, { autoBind: true })
    onBecomeObserved(this, 'sessions', this.ensureSessionsLoaded)
  }

  init = () => {
    this.unsubscribers.push(
      globalEventBus.subscribe(EventType.Internal, (event) => {
        this.handleSessionEvent(event.payload.event)
      }),
    )
  }

  destroy = () => {
    this.unsubscribers.forEach((unsubscribe) => unsubscribe())
    this.unsubscribers = []
  }

  sortSessions = () => {
    this.sortColumn = this.sortColumn === 'name' ? 'createdAt' : 'name'
  }

  get state() {
    return this.sidebar.getSessionsState(this.projectId)
  }

  get visibleSessions() {
    return this.sessions
      .filter((session) => session.parentId === null)
      .toSorted((a, b) => {
        if (this.sortColumn === 'createdAt') return dayjs(b.createdAt).diff(dayjs(a.createdAt))

        return a.name.localeCompare(b.name)
      })
  }

  openSession = (sessionId: string) => this.sidebar.openSession(this.projectId, sessionId)

  private ensureSessionsLoaded = () => {
    if (this.hasLoaded || this.isLoading) return
    this.loadSessions()
  }

  private loadSessions = flow(function* (this: SessionsTreeViewmodel) {
    this.isLoading = true
    try {
      const items = yield SessionModel.list(this.projectId)

      this.sessions = items
      this.hasLoaded = true
      this.isLoading = false
    } catch {
      this.isLoading = false
    }
  })

  private handleSessionEvent = (event: InternalEvent) => {
    switch (event.type) {
      case 'session_added':
      case 'session_removed':
      case 'session_updated':
        this.loadSessions()
        break
    }
  }
}
