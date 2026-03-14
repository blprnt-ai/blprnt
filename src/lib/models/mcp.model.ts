import type { McpServerConfig, McpServerCreateParams, McpServerPatch, McpServerStatus } from '@/bindings'
import { tauriMcpApi } from '@/lib/api/tauri/mcp.api'

// biome-ignore lint/complexity/noStaticOnlyClass: resonons
export class McpModel {
  static list = async (): Promise<McpServerConfig[]> => tauriMcpApi.listServers()

  static listStatuses = async (): Promise<McpServerStatus[]> => tauriMcpApi.listStatuses()

  static create = async (params: McpServerCreateParams): Promise<McpServerConfig> => tauriMcpApi.createServer(params)

  static update = async (serverId: string, patch: McpServerPatch): Promise<McpServerConfig> =>
    tauriMcpApi.updateServer(serverId, patch)

  static delete = async (serverId: string): Promise<void> => tauriMcpApi.deleteServer(serverId)

  static testConnection = async (serverId: string): Promise<McpServerStatus> => tauriMcpApi.testConnection(serverId)
}
