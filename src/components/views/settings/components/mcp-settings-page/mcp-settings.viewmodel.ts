import { flow, makeAutoObservable } from 'mobx'
import type {
  McpAuthConfig,
  McpServerConfig,
  McpServerLifecycleState,
  McpServerStatus,
  McpTransportConfig,
} from '@/bindings'
import { basicToast } from '@/components/atoms/toaster'
import { EventType, globalEventBus } from '@/lib/events'
import { McpModel } from '@/lib/models/mcp.model'

type TransportType = 'stdio' | 'sse_http'
type AuthType = 'none' | 'bearer_token' | 'api_key' | 'basic' | 'headers'
const DEFAULT_STDIO_COMMAND = 'npx'
const DEFAULT_SSE_URL = 'http://localhost:3000/mcp'
const DEFAULT_API_KEY_HEADER = 'x-api-key'

export class McpSettingsViewModel {
  servers: McpServerConfig[] = []
  statuses = new Map<string, McpServerStatus>()

  isLoading = false
  isSaving = false
  isDeleting = false
  isTesting = false
  loadError: string | null = null
  submitError: string | null = null
  selectedServerId: string | null = null
  isCreatingNew = false
  fieldErrors = new Map<string, string>()

  name = ''
  enabled = true

  transportType: TransportType = 'stdio'
  stdioCommand = ''
  stdioArgs = ''
  stdioCwd = ''
  stdioEnv = ''
  sseUrl = ''
  sseHeaders = ''

  authType: AuthType = 'none'
  bearerToken = ''
  apiKeyHeader = 'x-api-key'
  apiKeyValue = ''
  basicUsername = ''
  basicPassword = ''
  authHeaders = ''

  private unsubscribeInternal: (() => void) | null = null
  private pollTimer: ReturnType<typeof setInterval> | null = null

  constructor() {
    makeAutoObservable(this, {}, { autoBind: true })
  }

  init = flow(function* (this: McpSettingsViewModel) {
    yield this.refreshAll()

    this.unsubscribeInternal = globalEventBus.subscribe(
      EventType.Internal,
      () => {
        void this.refreshAll()
      },
      (event) => {
        const type = event.payload.event.type
        return (
          type === 'mcp_server_added' ||
          type === 'mcp_server_updated' ||
          type === 'mcp_server_removed' ||
          type === 'mcp_server_status_changed'
        )
      },
    )

    this.pollTimer = setInterval(() => {
      void this.refreshStatuses()
    }, 5000)
  })

  destroy = () => {
    this.unsubscribeInternal?.()
    this.unsubscribeInternal = null
    if (this.pollTimer) {
      clearInterval(this.pollTimer)
      this.pollTimer = null
    }
  }

  refreshAll = flow(function* (this: McpSettingsViewModel) {
    this.isLoading = true
    this.loadError = null
    try {
      const [servers, statuses]: [McpServerConfig[], McpServerStatus[]] = yield Promise.all([
        McpModel.list(),
        McpModel.listStatuses(),
      ])

      this.servers = servers.toSorted((left, right) => left.name.localeCompare(right.name))
      this.statuses = new Map(statuses.map((status) => [status.server_id, status]))

      if (this.selectedServerId && !this.servers.some((server) => server.id === this.selectedServerId)) {
        this.clearForm()
      }

      if (!this.selectedServerId && this.servers.length > 0) {
        this.selectServer(this.servers[0].id)
      }
    } catch (error) {
      this.loadError = this.errorMessage(error)
    } finally {
      this.isLoading = false
    }
  })

  refreshStatuses = flow(function* (this: McpSettingsViewModel) {
    try {
      const statuses: McpServerStatus[] = yield McpModel.listStatuses()
      this.statuses = new Map(statuses.map((status) => [status.server_id, status]))
    } catch {
      // status polling should not block UI
    }
  })

  get selectedServer() {
    if (!this.selectedServerId) return null
    return this.servers.find((server) => server.id === this.selectedServerId) ?? null
  }

  get hasServers() {
    return this.servers.length > 0
  }

  statusFor = (serverId: string): McpServerStatus | null => {
    return this.statuses.get(serverId) ?? null
  }

  stateLabel = (state: McpServerLifecycleState) => {
    switch (state) {
      case 'configured':
        return 'Configured'
      case 'connecting':
        return 'Connecting'
      case 'connected':
        return 'Connected'
      case 'degraded':
        return 'Degraded'
      case 'disconnected':
        return 'Disconnected'
      case 'error':
        return 'Error'
    }
  }

  startCreate = () => {
    this.clearForm()
    this.applyCreateDefaults()
    this.isCreatingNew = true
    this.selectedServerId = null
  }

  selectServer = (serverId: string) => {
    const server = this.servers.find((item) => item.id === serverId)
    if (!server) return

    this.isCreatingNew = false
    this.selectedServerId = serverId
    this.populateForm(server)
    this.submitError = null
    this.fieldErrors.clear()
  }

  toggleEnabled = flow(function* (this: McpSettingsViewModel, serverId: string, enabled: boolean) {
    try {
      const updated: McpServerConfig = yield McpModel.update(serverId, { enabled })
      this.upsertServer(updated)
      if (this.selectedServerId === serverId) this.enabled = enabled
      yield this.refreshStatuses()
    } catch (error) {
      basicToast.error({ description: this.errorMessage(error), title: 'Failed to update server' })
    }
  })

  save = flow(function* (this: McpSettingsViewModel) {
    this.submitError = null
    this.fieldErrors = this.validate()
    if (this.fieldErrors.size > 0) return

    this.isSaving = true
    try {
      const payload = this.buildPayload()

      if (this.isCreatingNew || !this.selectedServerId) {
        const created: McpServerConfig = yield McpModel.create(payload)
        this.upsertServer(created)
        this.selectServer(created.id)
        basicToast.success({ title: 'MCP server created' })
      } else {
        const updated: McpServerConfig = yield McpModel.update(this.selectedServerId, {
          auth: payload.auth,
          enabled: payload.enabled,
          name: payload.name,
          transport: payload.transport,
        })
        this.upsertServer(updated)
        this.selectServer(updated.id)
        basicToast.success({ title: 'MCP server updated' })
      }

      yield this.refreshStatuses()
    } catch (error) {
      this.submitError = this.errorMessage(error)
    } finally {
      this.isSaving = false
    }
  })

  deleteSelected = flow(function* (this: McpSettingsViewModel) {
    if (!this.selectedServerId) return

    this.isDeleting = true
    this.submitError = null
    try {
      const deletedId = this.selectedServerId
      yield McpModel.delete(deletedId)
      this.servers = this.servers.filter((server) => server.id !== deletedId)
      this.statuses.delete(deletedId)
      this.clearForm()
      if (this.servers.length > 0) this.selectServer(this.servers[0].id)
      basicToast.success({ title: 'MCP server removed' })
    } catch (error) {
      this.submitError = this.errorMessage(error)
    } finally {
      this.isDeleting = false
    }
  })

  testSelectedConnection = flow(function* (this: McpSettingsViewModel) {
    if (!this.selectedServerId) return

    this.isTesting = true
    this.submitError = null
    try {
      const status: McpServerStatus = yield McpModel.testConnection(this.selectedServerId)
      this.statuses.set(status.server_id, status)
      basicToast.success({ title: 'MCP server connected successfully' })
    } catch (error) {
      const message = this.errorMessage(error)
      this.submitError = message
      basicToast.error({ description: message, title: 'MCP test connection failed' })
    } finally {
      this.isTesting = false
      yield this.refreshStatuses()
    }
  })

  setName = (value: string) => (this.name = value)
  setEnabled = (value: boolean) => (this.enabled = value)
  setTransportType = (value: TransportType) => (this.transportType = value)
  setStdioCommand = (value: string) => (this.stdioCommand = value)
  setStdioArgs = (value: string) => (this.stdioArgs = value)
  setStdioCwd = (value: string) => (this.stdioCwd = value)
  setStdioEnv = (value: string) => (this.stdioEnv = value)
  setSseUrl = (value: string) => (this.sseUrl = value)
  setSseHeaders = (value: string) => (this.sseHeaders = value)
  setAuthType = (value: AuthType) => (this.authType = value)
  setBearerToken = (value: string) => (this.bearerToken = value)
  setApiKeyHeader = (value: string) => (this.apiKeyHeader = value)
  setApiKeyValue = (value: string) => (this.apiKeyValue = value)
  setBasicUsername = (value: string) => (this.basicUsername = value)
  setBasicPassword = (value: string) => (this.basicPassword = value)
  setAuthHeaders = (value: string) => (this.authHeaders = value)

  private clearForm = () => {
    this.isCreatingNew = false
    this.selectedServerId = null
    this.submitError = null
    this.fieldErrors.clear()

    this.name = ''
    this.enabled = true
    this.transportType = 'stdio'
    this.stdioCommand = ''
    this.stdioArgs = ''
    this.stdioCwd = ''
    this.stdioEnv = ''
    this.sseUrl = ''
    this.sseHeaders = ''
    this.authType = 'none'
    this.bearerToken = ''
    this.apiKeyHeader = DEFAULT_API_KEY_HEADER
    this.apiKeyValue = ''
    this.basicUsername = ''
    this.basicPassword = ''
    this.authHeaders = ''
  }

  private populateForm = (server: McpServerConfig) => {
    this.name = server.name
    this.enabled = server.enabled

    if (server.transport.type === 'stdio') {
      this.transportType = 'stdio'
      this.stdioCommand = server.transport.command
      this.stdioArgs = server.transport.args.join(' ')
      this.stdioCwd = server.transport.cwd ?? ''
      this.stdioEnv = this.serializeMap(server.transport.env ?? null)
      this.sseUrl = ''
      this.sseHeaders = ''
    } else {
      this.transportType = 'sse_http'
      this.sseUrl = server.transport.url
      this.sseHeaders = this.serializeMap(server.transport.headers ?? null)
      this.stdioCommand = ''
      this.stdioArgs = ''
      this.stdioCwd = ''
      this.stdioEnv = ''
    }

    switch (server.auth.type) {
      case 'none':
        this.authType = 'none'
        this.bearerToken = ''
        this.apiKeyHeader = DEFAULT_API_KEY_HEADER
        this.apiKeyValue = ''
        this.basicUsername = ''
        this.basicPassword = ''
        this.authHeaders = ''
        break
      case 'bearer_token':
        this.authType = 'bearer_token'
        this.bearerToken = server.auth.token
        this.apiKeyHeader = DEFAULT_API_KEY_HEADER
        this.apiKeyValue = ''
        this.basicUsername = ''
        this.basicPassword = ''
        this.authHeaders = ''
        break
      case 'api_key':
        this.authType = 'api_key'
        this.apiKeyHeader = server.auth.header
        this.apiKeyValue = server.auth.key
        this.bearerToken = ''
        this.basicUsername = ''
        this.basicPassword = ''
        this.authHeaders = ''
        break
      case 'basic':
        this.authType = 'basic'
        this.basicUsername = server.auth.username
        this.basicPassword = server.auth.password
        this.bearerToken = ''
        this.apiKeyHeader = DEFAULT_API_KEY_HEADER
        this.apiKeyValue = ''
        this.authHeaders = ''
        break
      case 'headers':
        this.authType = 'headers'
        this.authHeaders = this.serializeMap(server.auth.headers)
        this.bearerToken = ''
        this.apiKeyHeader = DEFAULT_API_KEY_HEADER
        this.apiKeyValue = ''
        this.basicUsername = ''
        this.basicPassword = ''
        break
    }
  }

  private validate = () => {
    const errors = new Map<string, string>()

    if (!this.name.trim()) errors.set('name', 'Server name is required.')

    if (this.transportType === 'stdio') {
      if (!this.stdioCommand.trim()) errors.set('stdioCommand', 'Command is required for stdio transport.')
      const envResult = this.parseMap(this.stdioEnv)
      if (!envResult.ok) errors.set('stdioEnv', envResult.error)
    }

    if (this.transportType === 'sse_http') {
      if (!this.sseUrl.trim()) {
        errors.set('sseUrl', 'URL is required for SSE/HTTP transport.')
      } else {
        try {
          const parsed = new URL(this.sseUrl.trim())
          if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
            errors.set('sseUrl', 'URL must start with http:// or https://.')
          }
        } catch {
          errors.set('sseUrl', 'URL is invalid.')
        }
      }
      const headersResult = this.parseMap(this.sseHeaders)
      if (!headersResult.ok) errors.set('sseHeaders', headersResult.error)
    }

    switch (this.authType) {
      case 'none':
        break
      case 'bearer_token':
        if (!this.bearerToken.trim()) errors.set('bearerToken', 'Bearer token is required.')
        break
      case 'api_key':
        if (!this.apiKeyHeader.trim()) errors.set('apiKeyHeader', 'API key header is required.')
        if (!this.apiKeyValue.trim()) errors.set('apiKeyValue', 'API key value is required.')
        break
      case 'basic':
        if (!this.basicUsername.trim()) errors.set('basicUsername', 'Username is required.')
        if (!this.basicPassword.trim()) errors.set('basicPassword', 'Password is required.')
        break
      case 'headers': {
        const authHeadersResult = this.parseMap(this.authHeaders)
        if (!authHeadersResult.ok) {
          errors.set('authHeaders', authHeadersResult.error)
        } else if (Object.keys(authHeadersResult.value).length === 0) {
          errors.set('authHeaders', 'At least one header is required.')
        }
        break
      }
    }

    return errors
  }

  private buildPayload = () => {
    return {
      auth: this.buildAuthConfig(),
      enabled: this.enabled,
      name: this.name.trim(),
      transport: this.buildTransportConfig(),
    }
  }

  private buildTransportConfig = (): McpTransportConfig => {
    if (this.transportType === 'stdio') {
      const env = this.parseMap(this.stdioEnv)
      return {
        args: this.parseArgs(this.stdioArgs),
        command: this.stdioCommand.trim(),
        cwd: this.stdioCwd.trim() || null,
        env: env.ok && Object.keys(env.value).length > 0 ? env.value : null,
        type: 'stdio',
      }
    }

    const headers = this.parseMap(this.sseHeaders)
    return {
      headers: headers.ok && Object.keys(headers.value).length > 0 ? headers.value : null,
      type: 'sse_http',
      url: this.sseUrl.trim(),
    }
  }

  private buildAuthConfig = (): McpAuthConfig => {
    switch (this.authType) {
      case 'none':
        return { type: 'none' }
      case 'bearer_token':
        return { token: this.bearerToken.trim(), type: 'bearer_token' }
      case 'api_key':
        return { header: this.apiKeyHeader.trim(), key: this.apiKeyValue.trim(), type: 'api_key' }
      case 'basic':
        return { password: this.basicPassword.trim(), type: 'basic', username: this.basicUsername.trim() }
      case 'headers': {
        const parsed = this.parseMap(this.authHeaders)
        return { headers: parsed.ok ? parsed.value : {}, type: 'headers' }
      }
    }
  }

  private parseArgs = (value: string) => {
    const trimmed = value.trim()
    if (!trimmed) return []
    return trimmed.split(/\s+/).filter(Boolean)
  }

  private parseMap = (input: string): { ok: true; value: Record<string, string> } | { ok: false; error: string } => {
    const lines = input
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean)

    const result: Record<string, string> = {}
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index]
      const separator = line.indexOf('=')
      if (separator <= 0 || separator >= line.length - 1) {
        return { error: `Invalid key=value entry at line ${index + 1}.`, ok: false }
      }
      const key = line.slice(0, separator).trim()
      const value = line.slice(separator + 1).trim()
      if (!key || !value) {
        return { error: `Invalid key=value entry at line ${index + 1}.`, ok: false }
      }
      result[key] = value
    }

    return { ok: true, value: result }
  }

  private serializeMap = (data: Partial<Record<string, string>> | null) => {
    if (!data) return ''
    return Object.entries(data)
      .map(([key, value]) => `${key}=${value}`)
      .join('\n')
  }

  private upsertServer = (server: McpServerConfig) => {
    const index = this.servers.findIndex((item) => item.id === server.id)
    if (index === -1) {
      this.servers = [...this.servers, server].toSorted((left, right) => left.name.localeCompare(right.name))
      return
    }

    const next = [...this.servers]
    next[index] = server
    this.servers = next.toSorted((left, right) => left.name.localeCompare(right.name))
  }

  private errorMessage = (error: unknown) => {
    if (typeof error === 'string') return error
    if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string')
      return error.message
    return 'Unexpected error'
  }

  private applyCreateDefaults = () => {
    this.transportType = 'stdio'
    this.stdioCommand = DEFAULT_STDIO_COMMAND
    this.sseUrl = DEFAULT_SSE_URL
    this.authType = 'none'
    this.apiKeyHeader = DEFAULT_API_KEY_HEADER
  }
}
