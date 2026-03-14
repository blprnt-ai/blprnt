import {
  commands,
  type McpServerConfig,
  type McpServerCreateParams,
  type McpServerPatch,
  type McpServerStatus,
  type McpToolDescriptor,
} from '@/bindings'
import { EventType, globalEventBus } from '@/lib/events'

class TauriMcpApi {
  public async listServers(): Promise<McpServerConfig[]> {
    const result = await commands.mcpServerList()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async getServer(serverId: string): Promise<McpServerConfig> {
    const result = await commands.mcpServerGet(serverId)
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async createServer(params: McpServerCreateParams): Promise<McpServerConfig> {
    const result = await commands.mcpServerCreate(params)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        serverId: result.data.id,
        type: 'mcp_server_added',
      },
    })

    return result.data
  }

  public async updateServer(serverId: string, patch: McpServerPatch): Promise<McpServerConfig> {
    const result = await commands.mcpServerUpdate(serverId, patch)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        serverId,
        type: 'mcp_server_updated',
      },
    })

    return result.data
  }

  public async deleteServer(serverId: string): Promise<void> {
    const result = await commands.mcpServerDelete(serverId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        serverId,
        type: 'mcp_server_removed',
      },
    })
  }

  public async testConnection(serverId: string): Promise<McpServerStatus> {
    const result = await commands.mcpServerTestConnection(serverId)
    if (result.status === 'error') throw result.error

    globalEventBus.emit(EventType.Internal, {
      event: {
        serverId,
        type: 'mcp_server_status_changed',
      },
    })

    return result.data
  }

  public async listStatuses(): Promise<McpServerStatus[]> {
    const result = await commands.mcpServerStatusList()
    if (result.status === 'error') throw result.error

    return result.data
  }

  public async listTools(): Promise<McpToolDescriptor[]> {
    const result = await commands.mcpServerToolsList()
    if (result.status === 'error') throw result.error

    return result.data
  }
}

export const tauriMcpApi = new TauriMcpApi()

export { TauriMcpApi }