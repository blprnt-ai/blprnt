import type { CreateMcpServerPayload } from '@/bindings/CreateMcpServerPayload'
import type { McpOauthCompletePayload } from '@/bindings/McpOauthCompletePayload'
import type { McpOauthLaunchDto } from '@/bindings/McpOauthLaunchDto'
import type { McpOauthStatusDto } from '@/bindings/McpOauthStatusDto'
import type { McpServerDto } from '@/bindings/McpServerDto'
import type { McpServerPatchPayload } from '@/bindings/McpServerPatchPayload'
import { apiClient } from './fetch'

class McpServersApi {
  public async list(): Promise<McpServerDto[]> {
    return apiClient.get('/mcp-servers')
  }

  public async create(data: CreateMcpServerPayload): Promise<McpServerDto> {
    return apiClient.post('/mcp-servers', { body: JSON.stringify(data) })
  }

  public async update(serverId: string, data: McpServerPatchPayload): Promise<McpServerDto> {
    return apiClient.patch(`/mcp-servers/${serverId}`, { body: JSON.stringify(data) })
  }

  public async getOauthStatus(serverId: string): Promise<McpOauthStatusDto> {
    return apiClient.get(`/mcp-servers/${serverId}/oauth`)
  }

  public async launchOauth(serverId: string): Promise<McpOauthLaunchDto> {
    return apiClient.post(`/mcp-servers/${serverId}/oauth/launch`)
  }

  public async reconnectOauth(serverId: string): Promise<McpOauthLaunchDto> {
    return apiClient.post(`/mcp-servers/${serverId}/oauth/reconnect`)
  }

  public async completeOauth(serverId: string, data: McpOauthCompletePayload): Promise<McpOauthStatusDto> {
    return apiClient.post(`/mcp-servers/${serverId}/oauth/complete`, { body: JSON.stringify(data) })
  }
}

export const mcpServersApi = new McpServersApi()
