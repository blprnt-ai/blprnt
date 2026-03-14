import { makeAutoObservable } from 'mobx'
import type { SidebarViewmodel } from '@/components/organisms/sidebar/sidebar.viewmodel'
import type { SessionModel } from '@/lib/models/session.model'

export class SessionTreeViewmodel {
  constructor(
    private readonly sidebar: SidebarViewmodel,
    public readonly projectId: string,
    public readonly session: SessionModel,
  ) {
    makeAutoObservable<SessionTreeViewmodel, 'sidebar'>(this, { session: false, sidebar: false }, { autoBind: true })
  }

  get panelState() {
    return this.sidebar.getSessionPanelState(this.projectId, this.session.id)
  }

  get isRunning() {
    return this.session.isRunning
  }

  openSession = () => this.sidebar.openSession(this.projectId, this.session.id)
  closeSession = () => this.panelState.closePanel()
}
