import { beforeEach, describe, expect, it, vi } from 'vitest'
import { EventType, globalEventBus } from '@/lib/events'
import { TauriMcpApi } from './mcp.api'

vi.mock('@/bindings', () => ({
  commands: {
    mcpServerCreate: vi.fn(),
    mcpServerDelete: vi.fn(),
    mcpServerGet: vi.fn(),
    mcpServerList: vi.fn(),
    mcpServerStatusList: vi.fn(),
    mcpServerTestConnection: vi.fn(),
    mcpServerToolsList: vi.fn(),
    mcpServerUpdate: vi.fn(),
  },
}))

describe('TauriMcpApi internal event payload hygiene', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    globalEventBus.clear()
  })

  it('emits server-only payloads without auth details for create/update/delete/test', async () => {
    const api = new TauriMcpApi()
    const captured: unknown[] = []
    globalEventBus.subscribe(EventType.Internal, (event) => captured.push(event.payload))

    await api.createServer({
      auth: { type: 'none' },
      enabled: true,
      name: 'Server',
      transport: { args: [], command: 'node', cwd: null, env: null, type: 'stdio' },
    })
    await api.updateServer('server-1', { name: 'Updated' })
    await api.deleteServer('server-1')
    await api.testConnection('server-1')

    expect(captured).toEqual([
      { event: { serverId: 'server-1', type: 'mcp_server_added' } },
      { event: { serverId: 'server-1', type: 'mcp_server_updated' } },
      { event: { serverId: 'server-1', type: 'mcp_server_removed' } },
      { event: { serverId: 'server-1', type: 'mcp_server_status_changed' } },
    ])

    const serialized = JSON.stringify(captured)
    expect(serialized).not.toContain('super-secret-token')
    expect(serialized).not.toContain('secret-password')
  })
})
