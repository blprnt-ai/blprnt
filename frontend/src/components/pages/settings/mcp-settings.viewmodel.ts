import { makeAutoObservable, runInAction } from 'mobx'
import { createContext, useContext } from 'react'
import { toast } from 'sonner'
import type { McpOauthLaunchDto } from '@/bindings/McpOauthLaunchDto'
import type { McpOauthStatusDto } from '@/bindings/McpOauthStatusDto'
import type { McpServerDto } from '@/bindings/McpServerDto'
import { McpServerSheetViewmodel } from '@/components/forms/mcp-server/mcp-server-sheet.viewmodel'
import { mcpServersApi } from '@/lib/api/mcp-servers'
import { AppModel } from '@/models/app.model'

type CompletionDraft = { code: string; state: string }

export class McpSettingsViewmodel {
  public errorMessage: string | null = null
  public isLoading = true
  public isRefreshingOauthId: string | null = null
  public oauthStatuses = new Map<string, McpOauthStatusDto>()
  public oauthLaunches = new Map<string, McpOauthLaunchDto>()
  public completionDrafts = new Map<string, CompletionDraft>()
  public selectedProjectId = AppModel.instance.projects[0]?.id ?? ''
  public servers: McpServerDto[] = []
  public readonly sheet: McpServerSheetViewmodel

  constructor() {
    this.sheet = new McpServerSheetViewmodel((server) => this.handleServerSaved(server))
    makeAutoObservable(this, {}, { autoBind: true })
  }

  public get projects() {
    return AppModel.instance.projects
  }

  public get hasProject() {
    return this.selectedProjectId.trim().length > 0
  }

  public async init() {
    if (!this.hasProject) {
      this.isLoading = false
      return
    }

    await this.loadServers()
  }

  public async setSelectedProject(projectId: string) {
    this.selectedProjectId = projectId
    this.oauthStatuses.clear()
    this.oauthLaunches.clear()
    this.completionDrafts.clear()
    await this.loadServers()
  }

  public openCreate() {
    if (!this.hasProject) return
    this.sheet.openForCreate(this.selectedProjectId)
  }

  public openEdit(server: McpServerDto) {
    this.sheet.openForEdit(server)
  }

  public setCompletionCode(serverId: string, code: string) {
    const draft = this.completionDrafts.get(serverId) ?? { code: '', state: '' }
    draft.code = code
    this.completionDrafts.set(serverId, draft)
  }

  public setCompletionState(serverId: string, state: string) {
    const draft = this.completionDrafts.get(serverId) ?? { code: '', state: '' }
    draft.state = state
    this.completionDrafts.set(serverId, draft)
  }

  public getCompletionDraft(serverId: string) {
    return this.completionDrafts.get(serverId) ?? { code: '', state: '' }
  }

  public getOauthStatus(serverId: string) {
    return this.oauthStatuses.get(serverId) ?? null
  }

  public getLaunch(serverId: string) {
    return this.oauthLaunches.get(serverId) ?? null
  }

  public async refreshOauth(serverId: string) {
    this.isRefreshingOauthId = serverId
    try {
      const status = await mcpServersApi.getOauthStatus(serverId)
      runInAction(() => {
        this.oauthStatuses.set(serverId, status)
      })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unable to load OAuth status.')
    } finally {
      runInAction(() => {
        this.isRefreshingOauthId = null
      })
    }
  }

  public async launchOauth(server: McpServerDto) {
    await this.startOauth(server, 'launch')
  }

  public async reconnectOauth(server: McpServerDto) {
    await this.startOauth(server, 'reconnect')
  }

  public async completeOauth(serverId: string) {
    const draft = this.getCompletionDraft(serverId)
    if (!draft.code.trim() || !draft.state.trim()) return

    this.isRefreshingOauthId = serverId
    try {
      const status = await mcpServersApi.completeOauth(serverId, { code: draft.code.trim(), state: draft.state.trim() })
      runInAction(() => {
        this.oauthStatuses.set(serverId, status)
        this.completionDrafts.set(serverId, { code: '', state: '' })
      })
      await this.loadServers(false)
      toast.success('OAuth connection updated.')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unable to complete OAuth.')
    } finally {
      runInAction(() => {
        this.isRefreshingOauthId = null
      })
    }
  }

  private async loadServers(setLoading = true) {
    if (!this.hasProject) return

    runInAction(() => {
      if (setLoading) this.isLoading = true
      this.errorMessage = null
    })

    try {
      const servers = await mcpServersApi.list(this.selectedProjectId)
      const oauthEntries = await Promise.all(
        servers.map(async (server) => {
          try {
            return [server.id, await mcpServersApi.getOauthStatus(server.id)] as const
          } catch {
            return null
          }
        }),
      )

      runInAction(() => {
        this.servers = [...servers].sort((left, right) => left.display_name.localeCompare(right.display_name))
        this.oauthStatuses = new Map(oauthEntries.flatMap((entry) => (entry ? [entry] : [])))
      })
    } catch (error) {
      runInAction(() => {
        this.errorMessage = error instanceof Error ? error.message : 'Unable to load MCP servers.'
      })
    } finally {
      runInAction(() => {
        this.isLoading = false
      })
    }
  }

  private async handleServerSaved(server: McpServerDto) {
    const index = this.servers.findIndex((candidate) => candidate.id === server.id)
    this.servers =
      index === -1
        ? [...this.servers, server].sort((left, right) => left.display_name.localeCompare(right.display_name))
        : this.servers.map((candidate) => (candidate.id === server.id ? server : candidate))

    await this.refreshOauth(server.id)
  }

  private async startOauth(server: McpServerDto, mode: 'launch' | 'reconnect') {
    this.isRefreshingOauthId = server.id
    try {
      const launch =
        mode === 'launch' ? await mcpServersApi.launchOauth(server.id) : await mcpServersApi.reconnectOauth(server.id)

      runInAction(() => {
        this.oauthLaunches.set(server.id, launch)
      })

      window.open(launch.authorization_url, '_blank', 'noopener,noreferrer')
      await this.refreshOauth(server.id)
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Unable to start OAuth flow.')
      runInAction(() => {
        this.isRefreshingOauthId = null
      })
    }
  }
}

export const McpSettingsViewmodelContext = createContext<McpSettingsViewmodel | null>(null)

export const useMcpSettingsViewmodel = () => {
  const viewmodel = useContext(McpSettingsViewmodelContext)
  if (!viewmodel) throw new Error('McpSettingsViewmodel not found')
  return viewmodel
}
