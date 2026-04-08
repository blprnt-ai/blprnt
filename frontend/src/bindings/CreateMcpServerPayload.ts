import type { McpServerAuthState } from './McpServerAuthState'

export type CreateMcpServerPayload = {
  project_id: string
  display_name: string
  description: string
  transport: string
  endpoint_url: string
  enabled?: boolean
  auth_state?: McpServerAuthState | null
  auth_summary?: string | null
}
